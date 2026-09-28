use crate::authority::{AttackOperation, CampaignController, WorldIdentity};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Catalog {
    pub services: Vec<ServiceSpec>,
    pub attacks: Vec<AttackSpec>,
}
impl Default for Catalog {
    fn default() -> Self {
        serde_json::from_str(include_str!("../data/catalog.json")).expect("valid bundled catalog")
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceSpec {
    pub kind: String,
    pub name: String,
    pub prefix: String,
    pub zone: String,
    pub price: i64,
    pub revenue_per_min: i64,
    pub cost_per_min: i64,
    pub port: u16,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttackSpec {
    pub kind: String,
    pub name: String,
    pub attacker_id: String,
    pub stage_seconds: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Backup {
    pub tick: u64,
    pub immutable: bool,
    pub clean: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Server {
    pub id: String,
    pub kind: String,
    pub zone: String,
    pub deployed: bool,
    pub isolated: bool,
    pub patched: bool,
    pub maintenance_until: u64,
    pub maintenance_kind: Option<String>,
    pub backup: Backup,
    pub traffic: u64,
    pub downtime: u64,
    pub lost_revenue: i64,
}
impl Server {
    pub fn online(&self, tick: u64) -> bool {
        self.deployed && !self.isolated && self.maintenance_until <= tick
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Credential {
    pub version: u64,
    pub weak: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub enabled: bool,
    pub allowed_hosts: Vec<String>,
    pub credential: Credential,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FirewallRule {
    pub source: String,
    pub destination: String,
    pub service: String,
    pub action: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Network {
    pub segmented: bool,
    pub rules: Vec<FirewallRule>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Attacker {
    pub id: String,
    pub name: String,
    pub archetype: String,
    pub skill: u8,
    pub stealth: u8,
    pub attempts: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttackStage {
    Reconnaissance,
    InitialAccess,
    Execution,
    Persistence,
    Discovery,
    LateralMovement,
    Collection,
    Exfiltration,
    Complete,
    Contained,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Access {
    pub host: String,
    pub privilege: String,
    pub session: bool,
    pub persistence: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttackCampaign {
    #[serde(default)]
    pub controller: CampaignController,
    #[serde(default)]
    pub pending_operation: Option<AttackOperation>,
    pub id: u64,
    pub attacker: String,
    pub target: String,
    pub source_ip: String,
    pub objective: String,
    pub kind: String,
    pub stage: AttackStage,
    pub stage_since: u64,
    pub stage_seconds: u64,
    pub stealth_level: u8,
    pub skill_level: u8,
    pub current_access: Vec<Access>,
    pub discovered_credentials: Vec<u64>,
    pub known_hosts: Vec<String>,
    pub persistence: bool,
    pub detected: bool,
    pub started: u64,
    pub detected_at: Option<u64>,
    pub affected_hosts: Vec<String>,
    pub data_lost_mb: u64,
    pub timeline: Vec<SecurityEvent>,
    pub lost_revenue_at_start: i64,
    pub downtime_at_start: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventKind {
    AuthLoginFailed,
    AuthLoginSuccess,
    HttpRequest,
    ProcessStarted,
    FileCreated,
    NetworkConnection,
    FirewallBlock,
    DataTransfer,
    ServiceStarted,
    ServiceStopped,
    HostIsolated,
    SourceBlocked,
    CredentialRotated,
    UserDisabled,
    ServicePatched,
    BackupRestored,
    NetworkSegmented,
    CampaignEnded,
}
impl EventKind {
    pub fn label(&self) -> String {
        serde_json::to_value(self)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string()
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub id: u64,
    pub tick: u64,
    pub event: EventKind,
    pub host: String,
    pub source_ip: String,
    pub destination: String,
    pub user: String,
    pub message: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DetectionRule {
    pub id: String,
    pub threshold: usize,
    pub window_seconds: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Alert {
    pub id: u64,
    pub tick: u64,
    pub host: String,
    pub source_ip: String,
    pub title: String,
    pub detail: String,
    pub severity: String,
    pub resolved: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IncidentReport {
    pub id: u64,
    pub closed_tick: u64,
    pub outcome: String,
    pub root_cause: String,
    pub attacker: String,
    pub time_to_detect: Option<u64>,
    pub time_to_contain: Option<u64>,
    pub affected_hosts: Vec<String>,
    pub data_lost_mb: u64,
    pub downtime_seconds: u64,
    pub lost_revenue: i64,
    pub recommendations: Vec<String>,
    pub timeline: Vec<SecurityEvent>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    Buy { kind: String },
    Deploy { host: String },
    BlockSource { source_ip: String },
    Isolate { host: String },
    Resume { host: String },
    RotateCredentials,
    DisableUser,
    EnableUser,
    Patch { host: String },
    Restore { host: String },
    Segment,
    Pause,
    Practice { kind: String },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct World {
    #[serde(default)]
    pub identity: WorldIdentity,
    pub format_version: u32,
    pub tick: u64,
    pub cash: i64,
    pub money_remainder: i64,
    pub paused: bool,
    pub seed: u64,
    pub next_event: u64,
    pub next_campaign: u64,
    pub next_attack_tick: u64,
    pub servers: Vec<Server>,
    pub network: Network,
    pub user: User,
    pub attackers: Vec<Attacker>,
    pub campaign: Option<AttackCampaign>,
    pub footholds: Vec<Access>,
    pub events: Vec<SecurityEvent>,
    pub alerts: Vec<Alert>,
    pub reports: Vec<IncidentReport>,
    pub detection: DetectionRule,
}
#[derive(Serialize)]
pub struct WorldView {
    pub tick: u64,
    pub cash: i64,
    pub net_per_min: i64,
    pub paused: bool,
    pub servers: Vec<Server>,
    pub network: Network,
    pub user: User,
    pub events: Vec<SecurityEvent>,
    pub alerts: Vec<Alert>,
    pub reports: Vec<IncidentReport>,
    pub catalog: Catalog,
    pub detection: DetectionRule,
}
