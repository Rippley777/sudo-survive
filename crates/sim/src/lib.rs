pub mod model;
pub mod query;
pub mod storage;
pub use model::*;

pub const DOLLAR: i64 = 1_000_000;

impl Default for World {
    fn default() -> Self {
        let mut world = Self {
            format_version: 1,
            tick: 0,
            cash: 160 * DOLLAR,
            money_remainder: 0,
            paused: false,
            seed: 42,
            next_event: 1,
            next_campaign: 1,
            next_attack_tick: 45,
            servers: vec![],
            network: Network {
                segmented: false,
                rules: vec![],
            },
            user: User {
                name: "deploy".into(),
                enabled: true,
                allowed_hosts: vec![],
                credential: Credential {
                    version: 1,
                    weak: true,
                },
            },
            attackers: vec![
                Attacker {
                    id: "packet-rat".into(),
                    name: "Packet Rat".into(),
                    archetype: "Opportunist".into(),
                    skill: 2,
                    stealth: 1,
                    attempts: 0,
                },
                Attacker {
                    id: "grey-market".into(),
                    name: "Grey Market".into(),
                    archetype: "Credential broker".into(),
                    skill: 5,
                    stealth: 5,
                    attempts: 0,
                },
                Attacker {
                    id: "night-shift".into(),
                    name: "Night Shift".into(),
                    archetype: "Patient operator".into(),
                    skill: 7,
                    stealth: 8,
                    attempts: 0,
                },
            ],
            campaign: None,
            footholds: vec![],
            events: vec![],
            alerts: vec![],
            reports: vec![],
            detection: DetectionRule {
                id: "auth-burst".into(),
                threshold: 5,
                window_seconds: 60,
            },
        };
        world.add_server("web");
        world.servers[0].deployed = true;
        world.emit(
            EventKind::ServiceStarted,
            "WEB-01",
            "127.0.0.1",
            "",
            "system",
            "Web hosting deployed. Clean immutable recovery point created.",
        );
        world
    }
}

impl World {
    fn spec(kind: &str) -> Option<ServiceSpec> {
        Catalog::default()
            .services
            .into_iter()
            .find(|s| s.kind == kind)
    }
    fn add_server(&mut self, kind: &str) {
        let spec = Self::spec(kind).expect("validated service");
        let count = self.servers.iter().filter(|s| s.kind == kind).count() + 1;
        let id = format!("{}-{:02}", spec.prefix, count);
        self.user.allowed_hosts.push(id.clone());
        self.servers.push(Server {
            id,
            kind: kind.into(),
            zone: spec.zone,
            deployed: false,
            isolated: false,
            patched: false,
            maintenance_until: 0,
            maintenance_kind: None,
            backup: Backup {
                tick: self.tick,
                immutable: true,
                clean: true,
            },
            traffic: 0,
            downtime: 0,
            lost_revenue: 0,
        });
    }
    fn server_index(&self, host: &str) -> Result<usize, String> {
        self.servers
            .iter()
            .position(|s| s.id == host)
            .ok_or_else(|| "Unknown host".into())
    }
    fn spend(&mut self, dollars: i64) -> Result<(), String> {
        if self.cash < dollars * DOLLAR {
            return Err(format!("Requires ${dollars}"));
        }
        self.cash -= dollars * DOLLAR;
        Ok(())
    }
    fn random(&mut self) -> u64 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        self.seed
    }
    pub fn net_per_min(&self) -> i64 {
        self.servers
            .iter()
            .map(|s| {
                let spec = Self::spec(&s.kind).unwrap();
                (if s.online(self.tick) {
                    spec.revenue_per_min
                } else {
                    0
                }) - spec.cost_per_min
            })
            .sum()
    }
    pub fn view(&self) -> WorldView {
        WorldView {
            tick: self.tick,
            cash: self.cash,
            net_per_min: self.net_per_min(),
            paused: self.paused,
            servers: self.servers.clone(),
            network: self.network.clone(),
            user: self.user.clone(),
            events: self.events.clone(),
            alerts: self.alerts.clone(),
            reports: self.reports.clone(),
            catalog: Catalog::default(),
            detection: self.detection.clone(),
        }
    }
    fn emit(
        &mut self,
        event: EventKind,
        host: &str,
        source: &str,
        destination: &str,
        user: &str,
        message: &str,
    ) {
        self.events.push(SecurityEvent {
            id: self.next_event,
            tick: self.tick,
            event,
            host: host.into(),
            source_ip: source.into(),
            destination: destination.into(),
            user: user.into(),
            message: message.into(),
        });
        self.next_event += 1;
        if self.events.len() > 600 {
            self.events.remove(0);
        }
    }
    /// Deny rules precede the default route. Isolation applies in both directions.
    pub fn can_connect(&self, source: &str, destination: &str, service: &str) -> bool {
        let from = self.servers.iter().find(|s| s.id == source);
        let to = self.servers.iter().find(|s| s.id == destination);
        if from.is_some_and(|s| !s.online(self.tick)) || to.is_some_and(|s| !s.online(self.tick)) {
            return false;
        }
        if self.network.rules.iter().any(|r| {
            r.action == "deny"
                && (r.source == "*" || r.source == source)
                && (r.destination == "*" || r.destination == destination)
                && (r.service == "*" || r.service == service)
        }) {
            return false;
        }
        if let (Some(a), Some(b)) = (from, to) {
            if self.network.segmented && a.zone == "dmz" && b.zone == "internal" && service == "ssh"
            {
                return false;
            }
        }
        // Only DMZ services have public ingress. Outbound routes are otherwise allowed.
        if from.is_none() && to.is_some_and(|s| s.zone != "dmz") {
            return false;
        }
        true
    }
    pub fn tick(&mut self) {
        if self.paused {
            return;
        }
        self.tick += 1;
        let mut completed = vec![];
        for s in &mut self.servers {
            if s.maintenance_kind.is_some() && s.maintenance_until <= self.tick {
                let kind = s.maintenance_kind.take().unwrap();
                if kind == "patch" {
                    s.patched = true;
                }
                completed.push((s.id.clone(), kind));
            }
            let spec = Self::spec(&s.kind).unwrap();
            if s.online(self.tick) {
                s.traffic = 12 + ((self.tick * 7 + s.id.len() as u64) % 31);
            } else {
                s.traffic = 0;
                if s.deployed {
                    s.downtime += 1;
                    s.lost_revenue += spec.revenue_per_min * DOLLAR / 60;
                }
            }
        }
        for (host, kind) in completed {
            if kind == "restore" {
                self.footholds.retain(|a| a.host != host);
                if let Some(c) = &mut self.campaign {
                    c.current_access.retain(|a| a.host != host);
                }
                self.emit(EventKind::BackupRestored, &host, "127.0.0.1", "", "operator", "Clean backup restored. Sessions and endpoint persistence removed. Host remains isolated.");
            } else {
                self.emit(
                    EventKind::ServicePatched,
                    &host,
                    "127.0.0.1",
                    "",
                    "operator",
                    "Patch complete. Web weakness removed; existing access is unaffected.",
                );
            }
        }
        let total = self.net_per_min() * DOLLAR + self.money_remainder;
        self.cash += total / 60;
        self.money_remainder = total % 60;
        if self.tick.is_multiple_of(6) {
            let hosts: Vec<_> = self
                .servers
                .iter()
                .filter(|s| s.online(self.tick))
                .map(|s| s.id.clone())
                .collect();
            for host in hosts {
                self.emit(
                    EventKind::HttpRequest,
                    &host,
                    "198.51.100.12",
                    &host,
                    "visitor",
                    "GET /health 200 · routine synthetic service traffic",
                );
            }
        }
        if self.campaign.is_none() && self.tick >= self.next_attack_tick {
            let kind = if self.next_campaign == 1 {
                "brute_force"
            } else {
                ["brute_force", "stolen_credential", "web_compromise"][(self.random() % 3) as usize]
            };
            if self.start_campaign(kind).is_err() {
                self.next_attack_tick = self.tick + 60;
            }
        }
        self.advance_campaign();
        self.detect();
        self.capture_timeline();
        self.finish_campaign();
    }
    fn start_campaign(&mut self, kind: &str) -> Result<(), String> {
        if self.campaign.is_some() {
            return Err("An exercise is already running. Investigate and contain it first.".into());
        }
        let spec = Catalog::default()
            .attacks
            .into_iter()
            .find(|a| a.kind == kind)
            .ok_or("Unknown exercise")?;
        let targets: Vec<_> = self
            .servers
            .iter()
            .filter(|s| s.zone == "dmz" && s.online(self.tick))
            .map(|s| s.id.clone())
            .collect();
        if targets.is_empty() {
            return Err("Deploy and resume a web server first.".into());
        }
        let target = targets[(self.random() as usize) % targets.len()].clone();
        let actor = self
            .attackers
            .iter_mut()
            .find(|a| a.id == spec.attacker_id)
            .unwrap();
        actor.attempts += 1;
        self.campaign = Some(AttackCampaign {
            id: self.next_campaign,
            attacker: actor.id.clone(),
            target: target.clone(),
            source_ip: format!("192.0.2.{}", 40 + self.next_campaign % 180),
            objective: "Collect and transfer hosted data".into(),
            kind: kind.into(),
            stage: AttackStage::Reconnaissance,
            stage_since: self.tick,
            stage_seconds: spec.stage_seconds,
            stealth_level: actor.stealth,
            skill_level: actor.skill,
            current_access: vec![],
            discovered_credentials: vec![self.user.credential.version],
            known_hosts: vec![target],
            persistence: false,
            detected: false,
            started: self.tick,
            detected_at: None,
            affected_hosts: vec![],
            data_lost_mb: 0,
            timeline: vec![],
            lost_revenue_at_start: self.servers.iter().map(|s| s.lost_revenue).sum(),
            downtime_at_start: self.servers.iter().map(|s| s.downtime).sum(),
        });
        self.next_campaign += 1;
        Ok(())
    }
    fn viable_access(&self, access: &Access) -> bool {
        self.servers
            .iter()
            .any(|s| s.id == access.host && s.online(self.tick))
            && (access.persistence || (access.session && self.user.enabled))
    }
    fn advance_campaign(&mut self) {
        let Some(mut c) = self.campaign.clone() else {
            return;
        };
        if !c.affected_hosts.is_empty() && !c.current_access.iter().any(|a| self.viable_access(a)) {
            // Maintenance suspends activity; it does not falsely eradicate the attacker.
            let temporarily_down = c.current_access.iter().any(|a| {
                self.servers
                    .iter()
                    .any(|s| s.id == a.host && !s.isolated && s.maintenance_until > self.tick)
                    && (a.persistence || (a.session && self.user.enabled))
            });
            if !temporarily_down {
                c.stage = AttackStage::Contained;
            }
            self.campaign = Some(c);
            return;
        }
        let elapsed = self.tick - c.stage_since;
        let source = c.source_ip.clone();
        let target = c.target.clone();
        if matches!(
            c.stage,
            AttackStage::Reconnaissance | AttackStage::InitialAccess
        ) && !self.can_connect(
            &source,
            &target,
            if c.kind == "web_compromise" {
                "https"
            } else {
                "ssh"
            },
        ) {
            self.emit(
                EventKind::FirewallBlock,
                &target,
                &source,
                &target,
                "",
                "Perimeter denied connection; no initial access established.",
            );
            c.stage = AttackStage::Contained;
            self.campaign = Some(c);
            return;
        }
        if c.stage == AttackStage::InitialAccess
            && c.kind == "brute_force"
            && elapsed.is_multiple_of(2)
            && elapsed <= 16
        {
            self.emit(
                EventKind::AuthLoginFailed,
                &target,
                &source,
                &target,
                "deploy",
                "SSH authentication rejected · invalid deployment credential",
            );
        }
        let wait = if c.stage == AttackStage::InitialAccess && c.kind == "brute_force" {
            20
        } else {
            c.stage_seconds
        };
        if elapsed < wait {
            return;
        }
        match c.stage {
            AttackStage::Reconnaissance => {
                self.emit(
                    EventKind::NetworkConnection,
                    &target,
                    &source,
                    &target,
                    "",
                    "Inbound connection to public service",
                );
                c.stage = AttackStage::InitialAccess;
            }
            AttackStage::InitialAccess => {
                let valid_account = self.user.enabled
                    && self.user.allowed_hosts.contains(&target)
                    && c.discovered_credentials
                        .contains(&self.user.credential.version);
                let valid = if c.kind == "web_compromise" {
                    !self
                        .servers
                        .iter()
                        .find(|s| s.id == target)
                        .unwrap()
                        .patched
                } else {
                    valid_account && (c.kind != "brute_force" || self.user.credential.weak)
                };
                if valid {
                    let event = if c.kind == "web_compromise" {
                        EventKind::HttpRequest
                    } else {
                        EventKind::AuthLoginSuccess
                    };
                    self.emit(
                        event,
                        &target,
                        &source,
                        &target,
                        if c.kind == "web_compromise" {
                            "web-service"
                        } else {
                            "deploy"
                        },
                        if c.kind == "web_compromise" {
                            "Unexpected admin handler response · HTTP 200"
                        } else {
                            "SSH session opened · unfamiliar source network"
                        },
                    );
                    let access = Access {
                        host: target.clone(),
                        privilege: "service".into(),
                        session: true,
                        persistence: false,
                    };
                    c.current_access.push(access.clone());
                    self.footholds.push(access);
                    c.affected_hosts.push(target.clone());
                    c.stage = AttackStage::Execution;
                } else {
                    self.emit(
                        EventKind::FirewallBlock,
                        &target,
                        &source,
                        &target,
                        "deploy",
                        "Initial access failed: service or identity control rejected attempt.",
                    );
                    c.stage = AttackStage::Contained;
                }
            }
            AttackStage::Execution => {
                self.emit(
                    EventKind::ProcessStarted,
                    &target,
                    &source,
                    "",
                    "deploy",
                    "Unexpected child process: service-worker → task-runner · unsigned executable",
                );
                c.stage = AttackStage::Persistence;
            }
            AttackStage::Persistence => {
                for a in c
                    .current_access
                    .iter_mut()
                    .chain(self.footholds.iter_mut())
                    .filter(|a| a.host == target)
                {
                    a.persistence = true;
                }
                c.persistence = true;
                self.emit(
                    EventKind::FileCreated,
                    &target,
                    &source,
                    "",
                    "deploy",
                    "New recurring startup task registered: telemetry-sync",
                );
                c.stage = AttackStage::Discovery;
            }
            AttackStage::Discovery => {
                c.known_hosts = self.servers.iter().map(|s| s.id.clone()).collect();
                if self.user.enabled {
                    c.discovered_credentials.push(self.user.credential.version);
                }
                self.emit(
                    EventKind::NetworkConnection,
                    &target,
                    &target,
                    "internal",
                    "deploy",
                    "Burst of internal service discovery connections",
                );
                c.stage = AttackStage::LateralMovement;
            }
            AttackStage::LateralMovement => {
                let destinations: Vec<_> = self
                    .servers
                    .iter()
                    .filter(|s| s.id != target && c.known_hosts.contains(&s.id))
                    .map(|s| s.id.clone())
                    .collect();
                for dest in destinations {
                    let route = self.can_connect(&target, &dest, "ssh");
                    let credential = self.user.enabled
                        && c.discovered_credentials
                            .contains(&self.user.credential.version)
                        && self.user.allowed_hosts.contains(&dest);
                    if route && credential {
                        self.emit(
                            EventKind::AuthLoginSuccess,
                            &dest,
                            &target,
                            &dest,
                            "deploy",
                            "Internal SSH session opened using shared deployment identity",
                        );
                        let a = Access {
                            host: dest.clone(),
                            privilege: "service".into(),
                            session: true,
                            persistence: false,
                        };
                        c.current_access.push(a.clone());
                        self.footholds.push(a);
                        c.affected_hosts.push(dest);
                    } else {
                        self.emit(if route { EventKind::AuthLoginFailed } else { EventKind::FirewallBlock }, &dest, &target, &dest, "deploy", if route { "Lateral authentication rejected: credential or account scope invalid" } else { "Lateral administration denied by network policy or host isolation" });
                    }
                }
                c.stage = AttackStage::Collection;
            }
            AttackStage::Collection => {
                for a in &c.current_access {
                    if self.viable_access(a) {
                        self.emit(
                            EventKind::FileCreated,
                            &a.host,
                            &a.host,
                            "",
                            "deploy",
                            "Bulk service records staged in a temporary archive",
                        );
                    }
                }
                c.stage = AttackStage::Exfiltration;
            }
            AttackStage::Exfiltration => {
                for a in &c.current_access {
                    if self.viable_access(a) && self.can_connect(&a.host, "203.0.113.88", "https") {
                        c.data_lost_mb += 24;
                        self.emit(
                            EventKind::DataTransfer,
                            &a.host,
                            &a.host,
                            "203.0.113.88",
                            "deploy",
                            "24 MB outbound transfer to unfamiliar external destination",
                        );
                    }
                }
                c.stage = AttackStage::Complete;
            }
            AttackStage::Complete | AttackStage::Contained => {}
        }
        c.stage_since = self.tick;
        self.campaign = Some(c);
    }
    fn detect(&mut self) {
        let recent: Vec<_> = self
            .events
            .iter()
            .filter(|e| e.tick == self.tick)
            .cloned()
            .collect();
        for e in recent {
            let (title, detail, severity) = match e.event {
                EventKind::AuthLoginFailed => {
                    let n = self.events.iter().filter(|other| other.event == EventKind::AuthLoginFailed && other.host == e.host && other.source_ip == e.source_ip && self.tick - other.tick <= self.detection.window_seconds).count();
                    if n < self.detection.threshold { continue; }
                    ("Authentication anomaly", format!("{n} failed logins within 60s. Review successful authentications before choosing a response."), "warning")
                }
                EventKind::AuthLoginSuccess if e.source_ip.starts_with("192.0.2.") => ("Unfamiliar authentication source", "Deployment identity authenticated from outside its expected network. Verify the session.".into(), "warning"),
                EventKind::ProcessStarted => ("Unexpected service child process", "A service spawned an unsigned task runner. Correlate process and access logs.".into(), "warning"),
                EventKind::DataTransfer => ("Unusual outbound transfer", "Service records transferred to an unfamiliar destination. Investigate the originating host.".into(), "critical"),
                _ => continue,
            };
            if self.alerts.iter().any(|a| {
                !a.resolved && a.host == e.host && a.title == title && a.source_ip == e.source_ip
            }) {
                continue;
            }
            self.alerts.push(Alert {
                id: e.id,
                tick: self.tick,
                host: e.host,
                source_ip: e.source_ip,
                title: title.into(),
                detail,
                severity: severity.into(),
                resolved: false,
            });
            if let Some(c) = &mut self.campaign {
                c.detected = true;
                c.detected_at.get_or_insert(self.tick);
            }
        }
        while self.alerts.len() > 60 {
            self.alerts.remove(0);
        }
    }
    fn capture_timeline(&mut self) {
        if let Some(c) = &mut self.campaign {
            let last = c.timeline.last().map_or(0, |e| e.id);
            c.timeline.extend(
                self.events
                    .iter()
                    .filter(|e| e.id > last && e.tick >= c.started && e.user != "visitor")
                    .cloned(),
            );
            if c.timeline.len() > 256 {
                c.timeline.drain(..c.timeline.len() - 256);
            }
        }
    }
    fn finish_campaign(&mut self) {
        let Some(c) = self.campaign.as_ref() else {
            return;
        };
        if !matches!(c.stage, AttackStage::Contained | AttackStage::Complete) {
            return;
        }
        let contained = c.stage == AttackStage::Contained;
        if !contained && self.reports.iter().any(|r| r.id == c.id) {
            return;
        }
        let target = c.target.clone();
        let source = c.source_ip.clone();
        self.emit(EventKind::CampaignEnded, &target, &source, "", "system", if contained { "Campaign contained. Inspect recovery requirements before resuming hosts." } else { "Observed transfer sequence ended. Affected hosts still require containment and clean recovery." });
        self.capture_timeline();
        let c = self.campaign.clone().unwrap();
        if contained {
            self.campaign = None;
        }
        if contained {
            for a in &mut self.alerts {
                a.resolved = true;
            }
        }
        let root = match c.kind.as_str() {
            "brute_force" => "Weak, reused deployment credential on an exposed SSH service",
            "stolen_credential" => "Stolen deployment credential with broad host authorization",
            _ => "Unpatched public web service",
        };
        self.reports.retain(|r| r.id != c.id);
        self.reports.push(IncidentReport {
            id: c.id,
            closed_tick: self.tick,
            outcome: if contained {
                "Contained"
            } else {
                "Data transferred"
            }
            .into(),
            root_cause: if c.affected_hosts.is_empty() {
                format!("Attempted entry: {root}. No access established.")
            } else {
                root.into()
            },
            attacker: self
                .attackers
                .iter()
                .find(|a| a.id == c.attacker)
                .unwrap()
                .name
                .clone(),
            time_to_detect: c.detected_at.map(|t| t - c.started),
            time_to_contain: contained.then_some(self.tick - c.started),
            affected_hosts: c.affected_hosts,
            data_lost_mb: c.data_lost_mb,
            downtime_seconds: self.servers.iter().map(|s| s.downtime).sum::<u64>()
                - c.downtime_at_start,
            lost_revenue: self.servers.iter().map(|s| s.lost_revenue).sum::<i64>()
                - c.lost_revenue_at_start,
            recommendations: vec![
                "Rotate the shared deployment credential; an IP block does not revoke a session."
                    .into(),
                "Isolate and restore affected hosts to remove endpoint persistence.".into(),
                "Segment DMZ from internal administration to limit lateral movement.".into(),
                "Patch exposed web services during planned maintenance.".into(),
            ],
            timeline: c.timeline,
        });
        if self.reports.len() > 20 {
            self.reports.remove(0);
        }
        self.next_attack_tick = self.tick + 240;
    }
    pub fn act(&mut self, action: Action) -> Result<(), String> {
        let restoring = matches!(&action, Action::Restore { .. });
        match action {
            Action::Buy { kind } => {
                let spec = Self::spec(&kind).ok_or("Unknown server type")?;
                if self.servers.len() >= 8 {
                    return Err("Rack capacity reached (8 servers).".into());
                }
                self.spend(spec.price)?;
                self.add_server(&kind);
            }
            Action::Deploy { host } => {
                let i = self.server_index(&host)?;
                if self.servers[i].deployed {
                    return Err("Service already deployed".into());
                }
                self.servers[i].deployed = true;
                self.emit(
                    EventKind::ServiceStarted,
                    &host,
                    "127.0.0.1",
                    "",
                    "operator",
                    "Service deployed; public or internal route follows its network zone.",
                );
            }
            Action::BlockSource { source_ip } => {
                let ip: std::net::Ipv4Addr = source_ip
                    .parse()
                    .map_err(|_| "Enter a valid source IPv4 address")?;
                let source = ip.to_string();
                if self.network.rules.iter().any(|r| r.source == source) {
                    return Err("Source already blocked".into());
                }
                self.network.rules.push(FirewallRule {
                    source: source.clone(),
                    destination: "*".into(),
                    service: "*".into(),
                    action: "deny".into(),
                });
                self.emit(EventKind::SourceBlocked, "FIREWALL", &source, "*", "operator", "Ingress source blocked. Existing host sessions and persistence are unaffected.");
            }
            Action::Isolate { host } => {
                let i = self.server_index(&host)?;
                if self.servers[i].isolated {
                    return Err("Host already isolated".into());
                }
                self.servers[i].isolated = true;
                self.emit(EventKind::HostIsolated, &host, "127.0.0.1", "", "operator", "All host network paths denied. Service revenue suspended; operating costs continue.");
            }
            Action::Resume { host } => {
                let i = self.server_index(&host)?;
                if self.servers[i].maintenance_until > self.tick {
                    return Err("Wait for maintenance to finish.".into());
                }
                if self.footholds.iter().any(|a| a.host == host) {
                    return Err("Recovery check failed: restore a clean backup before reconnecting this host.".into());
                }
                self.servers[i].isolated = false;
                for a in &mut self.alerts {
                    if a.host == host {
                        a.resolved = true;
                    }
                }
                self.emit(
                    EventKind::ServiceStarted,
                    &host,
                    "127.0.0.1",
                    "",
                    "operator",
                    "Host reconnected after recovery checks. Service revenue resumed.",
                );
            }
            Action::RotateCredentials => {
                self.user.credential.version += 1;
                self.user.credential.weak = false;
                self.emit(EventKind::CredentialRotated, "IDENTITY", "127.0.0.1", "", "operator", "Deployment credential rotated. Previously stolen versions invalid; existing sessions remain.");
            }
            Action::DisableUser => {
                self.user.enabled = false;
                for a in &mut self.footholds {
                    a.session = false;
                }
                if let Some(c) = &mut self.campaign {
                    for a in &mut c.current_access {
                        a.session = false;
                    }
                }
                self.emit(EventKind::UserDisabled, "IDENTITY", "127.0.0.1", "", "operator", "deploy disabled; active identity sessions revoked. Startup persistence is unaffected.");
            }
            Action::EnableUser => {
                self.user.enabled = true;
            }
            Action::Patch { host } | Action::Restore { host } => {
                let restore = restoring;
                let i = self.server_index(&host)?;
                if self.servers[i].maintenance_until > self.tick {
                    return Err("Maintenance already running".into());
                }
                if restore {
                    if !self.servers[i].isolated {
                        return Err("Isolate the host before restoring.".into());
                    }
                    if !self.servers[i].backup.clean {
                        return Err("No clean recovery point available".into());
                    }
                    self.spend(20)?;
                } else if self.servers[i].patched {
                    return Err("Service is already patched".into());
                }
                self.servers[i].maintenance_until = self.tick + if restore { 15 } else { 12 };
                self.servers[i].maintenance_kind =
                    Some(if restore { "restore" } else { "patch" }.into());
                self.emit(
                    EventKind::ServiceStopped,
                    &host,
                    "127.0.0.1",
                    "",
                    "operator",
                    if restore {
                        "Clean restore started · 15 seconds · $20"
                    } else {
                        "Patch started · 12 seconds service downtime"
                    },
                );
            }
            Action::Segment => {
                if self.network.segmented {
                    return Err("Network already segmented".into());
                }
                self.spend(40)?;
                self.network.segmented = true;
                self.emit(
                    EventKind::NetworkSegmented,
                    "FIREWALL",
                    "dmz",
                    "internal",
                    "operator",
                    "DMZ → internal SSH denied. Application service routes remain available.",
                );
            }
            Action::Pause => {
                self.paused = !self.paused;
            }
            Action::Practice { kind } => {
                self.start_campaign(&kind)?;
            }
        }
        self.capture_timeline();
        // Responses stop a campaign immediately, without advancing simulated time.
        if let Some(c) = &self.campaign {
            let no_access = !c.affected_hosts.is_empty()
                && !c.current_access.iter().any(|a| self.viable_access(a));
            let no_ingress = c.affected_hosts.is_empty()
                && !self.can_connect(
                    &c.source_ip,
                    &c.target,
                    if c.kind == "web_compromise" {
                        "https"
                    } else {
                        "ssh"
                    },
                );
            let maintenance = c.current_access.iter().any(|a| {
                self.servers
                    .iter()
                    .any(|s| s.id == a.host && !s.isolated && s.maintenance_until > self.tick)
            });
            if no_ingress || (no_access && !maintenance) {
                self.campaign.as_mut().unwrap().stage = AttackStage::Contained;
            }
        }
        self.finish_campaign();
        Ok(())
    }
}

#[cfg(test)]
mod tests;
