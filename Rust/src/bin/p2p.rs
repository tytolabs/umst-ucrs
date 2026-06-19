//! P2P gossip daemon: libp2p noise/yamux/mDNS/GossipSub mesh for UCRS clock sync.
//!
//! Publishes gate-guarded JSON [`umst_ucrs::wire::ClockTick`] on gossipsub topic
//! `umst-ucrs/v1/clocksync`. Inbound messages use [`umst_ucrs::p2p::apply_gated_inbound`].

use std::error::Error;
use std::time::Duration;

use clap::Parser;
use futures::StreamExt;
use libp2p::core::upgrade;
use libp2p::gossipsub::{
    AllowAllSubscriptionFilter, Behaviour as GossipsubBehaviour, ConfigBuilder,
    Event as GossipsubEvent, IdentTopic, IdentityTransform, MessageAuthenticity,
};
use libp2p::mdns;
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{identity, noise, tcp, yamux, Multiaddr, SwarmBuilder, Transport};
use tokio::time::interval;
use tracing::{info, warn};

use umst_ucrs::clock::LocalClock;
use umst_ucrs::credit::CreditLedger;
use umst_ucrs::p2p::{apply_gated_inbound, outbound_tick_if_admitted, GatedSyncOutcome};
use umst_ucrs::wire::ClockTick;
use umst_ucrs::{agent_tick, AgentConfig};

const TOPIC: &str = "umst-ucrs/v1/clocksync";

#[derive(NetworkBehaviour)]
struct Behaviour {
    gossipsub: GossipsubBehaviour<IdentityTransform, AllowAllSubscriptionFilter>,
    mdns: mdns::tokio::Behaviour,
}

/// UMST-UCRS P2P gossip daemon.
#[derive(Debug, Parser)]
#[command(name = "umst-ucrs-p2p", version, about = "UCRS libp2p gossip mesh")]
struct Cli {
    /// Local credit-ledger peer id.
    #[arg(long, default_value_t = 1)]
    peer_id: u64,
    /// TCP listen port (0 = ephemeral).
    #[arg(long, default_value_t = 4001)]
    port: u16,
    /// Bootstrap multiaddrs (repeatable).
    #[arg(long = "bootstrap")]
    bootstrap: Vec<String>,
    /// Landauer sync budget per window (bits).
    #[arg(long, default_value_t = 20.0)]
    budget_bits: f64,
    /// Agent tick / gossip publish interval (seconds).
    #[arg(long, default_value_t = 5)]
    tick_secs: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    let psk = std::env::var("UMST_UCRS_PSK").unwrap_or_else(|_| "umst-ucrs-dev-insecure".into());
    let secret = psk.as_bytes();

    let config = AgentConfig {
        peer_id: cli.peer_id,
        budget_bits: cli.budget_bits,
        sync_interval_sec: cli.tick_secs as f64,
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

    let gossipsub = GossipsubBehaviour::<IdentityTransform, AllowAllSubscriptionFilter>::new(
        MessageAuthenticity::Signed(local_key.clone()),
        gossipsub_config,
    )
    .map_err(|e| format!("gossipsub: {e}"))?;

    let mdns_behaviour = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer)?;

    let mut swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_other_transport(|_| transport)?
        .with_behaviour(|_| Behaviour {
            gossipsub,
            mdns: mdns_behaviour,
        })?
        .build();

    let listen: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", cli.port).parse()?;
    swarm.listen_on(listen.clone())?;

    for addr in &cli.bootstrap {
        if let Ok(m) = addr.parse::<Multiaddr>() {
            if let Err(e) = swarm.dial(m) {
                warn!(%addr, ?e, "bootstrap dial failed");
            }
        }
    }

    let topic = IdentTopic::new(TOPIC);
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    info!(%listen, %local_peer, peer_id = cli.peer_id, "umst-ucrs-p2p listening");

    let mut tick_timer = interval(Duration::from_secs(cli.tick_secs));

    loop {
        tokio::select! {
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                        for (peer, addr) in list {
                            info!(%peer, %addr, "mdns discovered");
                            swarm.dial(addr)?;
                        }
                    }
                    SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(GossipsubEvent::Message { message, .. })) => {
                        if let Ok(t) = serde_json::from_slice::<ClockTick>(&message.data) {
                            match apply_gated_inbound(&mut clock, &mut ledger, &config, &t, secret) {
                                GatedSyncOutcome::Admitted(_) => info!(agent = t.agent_id, "applied remote tick"),
                                GatedSyncOutcome::RejectedByGate => warn!("rejected tick: gate"),
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ = tick_timer.tick() => {
                clock.update_uncertainty();
                let _ = agent_tick(&mut clock, &mut ledger, &config);
                if let Some(ct) = outbound_tick_if_admitted(&clock, &config, secret) {
                    let data = serde_json::to_vec(&ct)?;
                    if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), data) {
                        warn!(?e, "gossip publish skipped (no peers yet)");
                    }
                }
            }
        }
    }
}
