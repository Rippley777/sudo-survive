use super::*;

fn advance(w: &mut World, ticks: u64) {
    for _ in 0..ticks {
        w.tick();
    }
}
fn exercise() -> World {
    let mut w = World::default();
    w.act(Action::Practice {
        kind: "brute_force".into(),
    })
    .unwrap();
    w
}
fn lateral_world(segmented: bool) -> World {
    let mut w = World {
        cash: 1000 * DOLLAR,
        ..World::default()
    };
    w.act(Action::Buy {
        kind: "database".into(),
    })
    .unwrap();
    w.act(Action::Deploy {
        host: "DB-01".into(),
    })
    .unwrap();
    if segmented {
        w.act(Action::Segment).unwrap();
    }
    w.act(Action::Practice {
        kind: "brute_force".into(),
    })
    .unwrap();
    advance(&mut w, 70);
    w
}
#[test]
fn replay_is_deterministic() {
    let mut a = World::default();
    let mut b = a.clone();
    advance(&mut a, 350);
    advance(&mut b, 350);
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}
#[test]
fn income_has_no_rounding_drift() {
    let mut w = World {
        next_attack_tick: 10000,
        ..World::default()
    };
    advance(&mut w, 60);
    assert_eq!(w.cash, 170 * DOLLAR);
}
#[test]
fn purchase_requires_money_and_deployment() {
    let mut w = World::default();
    w.act(Action::Buy {
        kind: "database".into(),
    })
    .unwrap();
    assert!(!w.servers[1].deployed);
    assert_eq!(w.net_per_min(), 4);
    assert!(w.act(Action::Buy { kind: "web".into() }).is_err());
    w.act(Action::Deploy {
        host: "DB-01".into(),
    })
    .unwrap();
    assert_eq!(w.net_per_min(), 28);
}
#[test]
fn alerts_come_from_events_before_access() {
    let mut w = exercise();
    advance(&mut w, 18);
    assert_eq!(
        w.events
            .iter()
            .filter(|e| e.event == EventKind::AuthLoginFailed)
            .count(),
        5
    );
    assert_eq!(w.alerts.len(), 1);
    assert!(w.campaign.as_ref().unwrap().current_access.is_empty());
}
#[test]
fn early_source_block_stops_initial_access() {
    let mut w = exercise();
    advance(&mut w, 18);
    let ip = w.alerts[0].source_ip.clone();
    w.act(Action::BlockSource { source_ip: ip }).unwrap();
    assert!(w.campaign.is_none());
    assert_eq!(w.reports[0].outcome, "Contained");
    assert!(w.reports[0].affected_hosts.is_empty());
}
#[test]
fn late_source_block_does_not_remove_session() {
    let mut w = exercise();
    advance(&mut w, 29);
    let ip = w.campaign.as_ref().unwrap().source_ip.clone();
    w.act(Action::BlockSource { source_ip: ip }).unwrap();
    assert!(w.campaign.is_some());
    advance(&mut w, 60);
    assert!(w.reports[0].data_lost_mb > 0);
}
#[test]
fn isolation_contains_but_does_not_eradicate() {
    let mut w = exercise();
    advance(&mut w, 45);
    w.act(Action::Isolate {
        host: "WEB-01".into(),
    })
    .unwrap();
    assert!(w.campaign.is_none());
    assert_eq!(w.reports[0].data_lost_mb, 0);
    assert!(w
        .act(Action::Resume {
            host: "WEB-01".into()
        })
        .is_err());
    assert_eq!(w.net_per_min(), -2);
}
#[test]
fn clean_restore_allows_safe_recovery() {
    let mut w = exercise();
    advance(&mut w, 45);
    w.act(Action::Isolate {
        host: "WEB-01".into(),
    })
    .unwrap();
    w.act(Action::Restore {
        host: "WEB-01".into(),
    })
    .unwrap();
    assert!(w
        .act(Action::Resume {
            host: "WEB-01".into()
        })
        .is_err());
    advance(&mut w, 15);
    w.act(Action::Resume {
        host: "WEB-01".into(),
    })
    .unwrap();
    assert_eq!(w.net_per_min(), 10);
    assert!(w.footholds.is_empty());
}
#[test]
fn segmentation_prevents_lateral_movement() {
    let flat = lateral_world(false);
    let segmented = lateral_world(true);
    assert!(flat
        .campaign
        .as_ref()
        .unwrap()
        .affected_hosts
        .contains(&"DB-01".into()));
    assert!(!segmented
        .campaign
        .as_ref()
        .unwrap()
        .affected_hosts
        .contains(&"DB-01".into()));
    assert!(segmented
        .events
        .iter()
        .any(|e| e.event == EventKind::FirewallBlock && e.host == "DB-01"));
}
#[test]
fn stolen_credentials_fail_after_rotation() {
    let mut w = World::default();
    w.act(Action::Practice {
        kind: "stolen_credential".into(),
    })
    .unwrap();
    advance(&mut w, 10);
    w.act(Action::RotateCredentials).unwrap();
    advance(&mut w, 10);
    assert!(w.reports[0].affected_hosts.is_empty());
}
#[test]
fn lateral_requires_valid_credential_and_scope() {
    let mut w = World {
        cash: 500 * DOLLAR,
        ..World::default()
    };
    w.act(Action::Buy {
        kind: "database".into(),
    })
    .unwrap();
    w.act(Action::Deploy {
        host: "DB-01".into(),
    })
    .unwrap();
    w.act(Action::Practice {
        kind: "brute_force".into(),
    })
    .unwrap();
    advance(&mut w, 53);
    w.act(Action::RotateCredentials).unwrap();
    advance(&mut w, 8);
    assert!(!w
        .campaign
        .as_ref()
        .unwrap()
        .affected_hosts
        .contains(&"DB-01".into()));
    assert!(w
        .events
        .iter()
        .any(|e| e.event == EventKind::AuthLoginFailed && e.host == "DB-01"));
}
#[test]
fn disabled_account_revokes_sessions_but_persistence_survives() {
    let mut early = exercise();
    advance(&mut early, 29);
    early.act(Action::DisableUser).unwrap();
    assert!(early.campaign.is_none());
    let mut late = exercise();
    advance(&mut late, 45);
    late.act(Action::DisableUser).unwrap();
    assert!(late.campaign.is_some());
}
#[test]
fn patched_web_resists_web_entry() {
    let mut w = World::default();
    w.act(Action::Patch {
        host: "WEB-01".into(),
    })
    .unwrap();
    advance(&mut w, 12);
    w.act(Action::Practice {
        kind: "web_compromise".into(),
    })
    .unwrap();
    advance(&mut w, 24);
    assert!(w.reports[0].affected_hosts.is_empty());
}
#[test]
fn patching_a_compromised_host_does_not_end_campaign() {
    let mut w = exercise();
    advance(&mut w, 29);
    w.act(Action::Patch {
        host: "WEB-01".into(),
    })
    .unwrap();
    advance(&mut w, 5);
    assert!(w.campaign.is_some());
    assert!(!w.footholds.is_empty());
}
#[test]
#[cfg(feature = "local-store")]
fn sqlite_round_trip_preserves_campaign_and_next_tick() {
    let mut store = storage::Store::open(":memory:").unwrap();
    let mut original = exercise();
    advance(&mut original, 47);
    store.save(&original).unwrap();
    let mut loaded = store.load().unwrap();
    original.tick();
    loaded.tick();
    assert_eq!(
        serde_json::to_string(&original).unwrap(),
        serde_json::to_string(&loaded).unwrap()
    );
}
#[test]
fn query_is_conjunctive_and_validated() {
    let mut w = exercise();
    advance(&mut w, 30);
    let matches =
        query::search(&w.events, "event = AUTH_LOGIN_FAILED AND user = \"deploy\"").unwrap();
    assert_eq!(matches.len(), 8);
    assert!(query::search(&w.events, "invalid = yes").is_err());
    assert!(query::search(&w.events, "host").is_err());
    assert!(query::search(&w.events, "host =").is_err());
}
#[test]
fn pause_freezes_simulation() {
    let mut w = exercise();
    w.act(Action::Pause).unwrap();
    let before = serde_json::to_string(&w).unwrap();
    advance(&mut w, 99);
    assert_eq!(before, serde_json::to_string(&w).unwrap());
}
#[test]
fn view_does_not_disclose_campaign_truth() {
    let w = exercise();
    let json = serde_json::to_value(w.view()).unwrap();
    assert!(json.get("campaign").is_none());
    assert!(json.get("footholds").is_none());
}
#[test]
fn containment_after_transfer_updates_the_same_incident() {
    let mut w = exercise();
    advance(&mut w, 85);
    assert_eq!(w.reports.len(), 1);
    assert_eq!(w.reports[0].outcome, "Data transferred");
    w.act(Action::Isolate {
        host: "WEB-01".into(),
    })
    .unwrap();
    assert!(w.campaign.is_none());
    assert_eq!(w.reports.len(), 1);
    assert_eq!(w.reports[0].outcome, "Contained");
    assert!(w.reports[0].data_lost_mb > 0);
    assert!(w.reports[0]
        .timeline
        .iter()
        .any(|e| e.event == EventKind::HostIsolated));
}
#[test]
fn isolation_of_one_host_does_not_stop_other_footholds() {
    let mut w = lateral_world(false);
    w.act(Action::Isolate {
        host: "WEB-01".into(),
    })
    .unwrap();
    assert!(w.campaign.is_some());
    w.act(Action::Isolate {
        host: "DB-01".into(),
    })
    .unwrap();
    assert!(w.campaign.is_none());
}
