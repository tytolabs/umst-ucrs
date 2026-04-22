//! P2P gossip daemon: publishes JSON [`umst_ucrs::wire::ClockTick`] on gossipsub topic `umst-ucrs/v1/clocksync`.
//! Inbound messages are verified and applied via [`umst_ucrs::wire::apply_inbound_clock_tick`] after `gate_check`.
//! Env: `UMST_UCRS_PSK` (shared secret for tick signatures), `UMST_UCRS_LISTEN` (multiaddr, default `/ip4/0.0.0.0/tcp/0`).

use std::error::Error;
use std::time::Duration;

use futures::StreamExt;
use libp2p::core::upgrade;
use libp2p::gossipsub::{
    AllowAllSubscriptionFilter, Behaviour as GossipsubBehaviour, ConfigBuilder,
    Event as GossipsubEvent, IdentTopic, IdentityTransform, MessageAuthenticity,
};
use libp2p::swarm::SwarmEvent;
use libp2p::{identity, noise, tcp, yamux, Multiaddr, SwarmBuilder, Transport};
use tokio::time::interval;
use tracing::{info, warn};

use umst_ucrs::clock::LocalClock;
use umst_ucrs::credit::CreditLedger;
use umst_ucrs::wire::{apply_inbound_clock_tick, sign_tick, ClockTick};
use umst_ucrs::{agent_tick, AgentConfig};

const TOPIC: &str = "umst-ucrs/v1/clocksync";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let _ = tracing_subscriber::fmt::try_init();

    let psk = std::env::var("UMST_UCRS_PSK").unwrap_or_else(|_| "umst-ucrs-dev-insecure".into());
    let secret = psk.as_bytes();

    let peer_id_cli = std::env::var("UMST_UCRS_PEER_ID")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(1);
    let drift_ppb = std::env::var("UMST_UCRS_DRIFT_PPB")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10.0);
    let temp_k = std::env::var("UMST_UCRS_TEMP_K")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(300.0);

    let config = AgentConfig {
        peer_id: peer_id_cli,
        drift_ppb,
        temperature_k: temp_k,
        ..AgentConfig::default()
    };

    let mut clock = LocalClock::new(config.drift_ppb, config.temperature_k);
    let mut ledger = CreditLedger::new(config.peer_id, config.temperature_k);

    let local_key = identity::Keypair::generate_ed25519();
    let local_peer = libp2p::PeerId::from(local_key.public());

    let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(upgrade::Version::V1Lazy)
        .authenticate(noise::Config::new(&local_key)?)
        .multiplex(yamux::Config::default())
        .boxed();

    let gossipsub_config = ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10))
        .validation_mode(libp2p::gossipsub::ValidationMode::Permissive)
        .build()
        .map_err(|e| format!("gossipsub config: {e}"))?;

    let mut gossipsub = GossipsubBehaviour::<IdentityTransform, AllowAllSubscriptionFilter>::new(
        MessageAuthenticity::Signed(local_key.clone()),
        gossipsub_config,
    )
    .map_err(|e| format!("gossipsub: {e}"))?;

    let topic = IdentTopic::new(TOPIC);
    gossipsub.subscribe(&topic)?;

    let mut swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_other_transport(|_| transport)?
        .with_behaviour(|_| gossipsub)?
        .build();

    let listen: Multiaddr = std::env::var("UMST_UCRS_LISTEN")
        .unwrap_or_else(|_| "/ip4/0.0.0.0/tcp/0".into())
        .parse()?;

    swarm.listen_on(listen.clone())?;
    info!(%listen, %local_peer, "umst-ucrs-p2p listening");

    let mut tick_timer = interval(Duration::from_secs(
        std::env::var("UMST_UCRS_PUBLISH_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5),
    ));

    loop {
        tokio::select! {
            event = swarm.select_next_some() => {
                if let SwarmEvent::Behaviour(GossipsubEvent::Message { message, .. }) = event {
                    if let Ok(t) = serde_json::from_slice::<ClockTick>(&message.data) {
                        match apply_inbound_clock_tick(&mut clock, &mut ledger, &config, &t, secret) {
                            umst_ucrs::wire::MergeOutcome::Accepted => {
                                info!(agent = t.agent_id, "applied remote tick")
                            }
                            umst_ucrs::wire::MergeOutcome::RejectedBadSig => {
                                warn!("rejected tick: bad sig")
                            }
                            umst_ucrs::wire::MergeOutcome::RejectedGate => warn!("rejected tick: gate"),
                            umst_ucrs::wire::MergeOutcome::RejectedSelf => {}
                        }
                    }
                }
            }
            _ = tick_timer.tick() => {
                clock.update_uncertainty();
                let _ = agent_tick(&mut clock, &mut ledger, &config);
                let mut ct = ClockTick {
                    agent_id: config.peer_id,
                    phase_entropy_bits: clock.phase_entropy_bits(),
                    landauer_cost_j: clock.desync_energy_joules(),
                    accuracy_score: 1.0,
                    sig: [0; 32],
                };
                sign_tick(secret, &mut ct);
                let data = serde_json::to_vec(&ct)?;
                if let Err(e) = swarm.behaviour_mut().publish(topic.clone(), data) {
                    warn!(?e, "gossip publish skipped (no peers yet)");
                }
            }
        }
    }
}
