//! Transport-independent command and view boundary. This is authorization, not
//! authentication: a trusted adapter must establish the principal beforehand.
use crate::{query, Action, AttackStage, SecurityEvent, World, WorldView};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PlayerId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NetworkId(pub String);

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorldMode {
    #[default]
    LocalTraining,
    Hosted,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldIdentity {
    pub network_id: NetworkId,
    pub owner: PlayerId,
    pub mode: WorldMode,
}
impl Default for WorldIdentity {
    fn default() -> Self {
        Self {
            network_id: NetworkId("local-network".into()),
            owner: PlayerId("local-player".into()),
            mode: WorldMode::LocalTraining,
        }
    }
}

/// Intentionally not deserializable. Never construct this from a request's
/// claimed player ID. A future gateway derives it from a verified session.
pub struct Principal {
    player: PlayerId,
}
impl Principal {
    pub fn from_verified_session(player: PlayerId) -> Self {
        Self { player }
    }
    pub fn local_operator() -> Self {
        Self {
            player: WorldIdentity::default().owner,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CampaignController {
    #[default]
    Npc,
    Player {
        player_id: PlayerId,
    },
}

/// Game-native decisions, never shell commands, payloads, or real endpoints.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttackOperation {
    EnumeratePublicService,
    AttemptInitialAccess,
    ExecuteSimulatedTask,
    EstablishPersistence,
    DiscoverReachableHosts,
    AttemptLateralMovement,
    CollectRecords,
    TransferRecords,
}
impl AttackOperation {
    pub fn for_stage(stage: &AttackStage) -> Option<Self> {
        Some(match stage {
            AttackStage::Reconnaissance => Self::EnumeratePublicService,
            AttackStage::InitialAccess => Self::AttemptInitialAccess,
            AttackStage::Execution => Self::ExecuteSimulatedTask,
            AttackStage::Persistence => Self::EstablishPersistence,
            AttackStage::Discovery => Self::DiscoverReachableHosts,
            AttackStage::LateralMovement => Self::AttemptLateralMovement,
            AttackStage::Collection => Self::CollectRecords,
            AttackStage::Exfiltration => Self::TransferRecords,
            AttackStage::Complete | AttackStage::Contained => return None,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(
    tag = "role",
    content = "decision",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Decision {
    Defender(Action),
    Attacker {
        campaign_id: u64,
        operation: AttackOperation,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub network_id: NetworkId,
    pub decision: Decision,
}
impl Command {
    pub fn defend(network_id: NetworkId, action: Action) -> Self {
        Self {
            network_id,
            decision: Decision::Defender(action),
        }
    }
}

/// Only this attacker's observations. Never embeds the defender's WorldView,
/// logs, controls, alert state, campaign truth, or other players' identifiers.
#[derive(Debug, Serialize)]
pub struct AttackerView {
    pub campaign_id: u64,
    pub target: String,
    pub known_hosts: Vec<String>,
    pub pending_operation: Option<AttackOperation>,
    pub next_operation: Option<AttackOperation>,
    pub observed_access: Vec<String>,
}

const UNAVAILABLE: &str = "Network or campaign unavailable";

impl World {
    fn authorize_owner(&self, principal: &Principal, network: &NetworkId) -> Result<(), String> {
        if *network != self.identity.network_id || principal.player != self.identity.owner {
            return Err(UNAVAILABLE.into());
        }
        Ok(())
    }

    pub fn apply(&mut self, principal: &Principal, command: Command) -> Result<(), String> {
        if command.network_id != self.identity.network_id {
            return Err(UNAVAILABLE.into());
        }
        match command.decision {
            Decision::Defender(action) => {
                self.authorize_owner(principal, &command.network_id)?;
                if self.identity.mode == WorldMode::Hosted
                    && matches!(action, Action::Pause | Action::Practice { .. })
                {
                    return Err("Pause and practice commands are local-training only".into());
                }
                self.act(action)
            }
            Decision::Attacker {
                campaign_id,
                operation,
            } => {
                let c = self.campaign.as_mut().ok_or(UNAVAILABLE)?;
                if c.id != campaign_id
                    || !matches!(&c.controller, CampaignController::Player { player_id } if *player_id == principal.player)
                {
                    return Err(UNAVAILABLE.into());
                }
                if c.pending_operation.is_some() {
                    return Err("An operation is already queued".into());
                }
                if AttackOperation::for_stage(&c.stage).as_ref() != Some(&operation) {
                    return Err("Operation is not available at this stage".into());
                }
                c.pending_operation = Some(operation);
                c.stage_since = self.tick;
                Ok(())
            }
        }
    }

    pub fn defender_view(
        &self,
        principal: &Principal,
        network: &NetworkId,
    ) -> Result<WorldView, String> {
        self.authorize_owner(principal, network)?;
        let mut view = self.view();
        if self.identity.mode == WorldMode::Hosted {
            view.paused = false;
            // Training debriefs reveal ground truth. Hosted play must not. Rich
            // evidence-based attribution/report projections are a later milestone.
            view.reports.clear();
            view.events
                .retain(|e| e.event != crate::EventKind::CampaignEnded);
        }
        Ok(view)
    }

    pub fn defender_logs(
        &self,
        principal: &Principal,
        network: &NetworkId,
        filter: &str,
    ) -> Result<Vec<SecurityEvent>, String> {
        query::search(&self.defender_view(principal, network)?.events, filter)
    }

    pub fn attacker_view(
        &self,
        principal: &Principal,
        network: &NetworkId,
        campaign_id: u64,
    ) -> Result<AttackerView, String> {
        if *network != self.identity.network_id {
            return Err(UNAVAILABLE.into());
        }
        let c = self.campaign.as_ref().ok_or(UNAVAILABLE)?;
        if c.id != campaign_id
            || !matches!(&c.controller, CampaignController::Player { player_id } if *player_id == principal.player)
        {
            return Err(UNAVAILABLE.into());
        }
        Ok(AttackerView {
            campaign_id: c.id,
            target: c.target.clone(),
            known_hosts: c.known_hosts.clone(),
            pending_operation: c.pending_operation.clone(),
            next_operation: AttackOperation::for_stage(&c.stage),
            observed_access: c.current_access.iter().map(|a| a.host.clone()).collect(),
        })
    }
}
