// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Consumer contract — observation, accept, wire, crypto, landauer, cast spine.
//! No daemon, no p2p, no rapl, no agent_tick.

pub mod observation {
    pub use crate::observation::*;
}
pub mod accept {
    pub use crate::accept::*;
}
pub mod wire {
    pub use crate::wire::*;
}
pub mod crypto {
    pub use crate::crypto::*;
}
pub mod landauer {
    pub use crate::landauer::*;
}
pub mod frame_spine {
    pub use crate::frame_spine::*;
}
pub mod design_sheaf {
    pub use crate::design_sheaf::*;
}
pub mod decision_tree {
    pub use crate::decision_tree::{SteerDecision, SteerDecisionTrace, SteerKnobs, SteerPolicy};
}

#[cfg(test)]
mod consumer_fence_tests {
    use super::observation::StampTier;

    #[test]
    fn shared_types_stamp_lane_reexports() {
        assert_eq!(StampTier::UcrsTier2.as_wire_str(), "UcrsTier2");
        // Fence: module docs forbid daemon/p2p/rapl — stamp-only consumer surface.
    }
}
