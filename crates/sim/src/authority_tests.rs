use crate::{authority::*, *};

fn principal(id: &str) -> Principal {
    Principal::from_verified_session(PlayerId(id.into()))
}

fn player_campaign() -> World {
    // Trusted harness setup only. No command/transport accepts controller assignment.
    // Future hosted admission must validate recon grants, eligibility and costs first.
    let mut w = World::default();
    w.start_campaign("brute_force").unwrap();
    w.campaign.as_mut().unwrap().controller = CampaignController::Player {
        player_id: PlayerId("red-player".into()),
    };
    w
}

fn queue(w: &mut World, operation: AttackOperation) -> Result<(), String> {
    let id = w.campaign.as_ref().unwrap().id;
    w.apply(
        &principal("red-player"),
        Command {
            network_id: w.identity.network_id.clone(),
            decision: Decision::Attacker {
                campaign_id: id,
                operation,
            },
        },
    )
}

fn run_player(w: &mut World, ticks: u64) {
    for _ in 0..ticks {
        if let Some(c) = &w.campaign {
            if c.pending_operation.is_none() {
                if let Some(op) = AttackOperation::for_stage(&c.stage) {
                    queue(w, op).unwrap();
                }
            }
        }
        w.tick();
    }
}

#[test]
fn cross_network_commands_views_and_logs_are_rejected_without_mutation() {
    let mut a = World::default();
    let mut b = World::default();
    b.identity.network_id = NetworkId("another-network".into());
    b.identity.owner = PlayerId("another-owner".into());
    let before_a = serde_json::to_string(&a).unwrap();
    let before_b = serde_json::to_string(&b).unwrap();
    let command = Command::defend(
        b.identity.network_id.clone(),
        Action::Isolate {
            host: "WEB-01".into(),
        },
    );
    assert!(a
        .apply(&Principal::local_operator(), command.clone())
        .is_err());
    assert!(b.apply(&Principal::local_operator(), command).is_err());
    assert!(b
        .defender_view(&Principal::local_operator(), &b.identity.network_id)
        .is_err());
    assert!(b
        .defender_logs(&Principal::local_operator(), &b.identity.network_id, "")
        .is_err());
    assert_eq!(before_a, serde_json::to_string(&a).unwrap());
    assert_eq!(before_b, serde_json::to_string(&b).unwrap());
}

#[test]
fn a_command_cannot_supply_a_claimed_principal() {
    let forged = serde_json::json!({
        "network_id": "local-network", "player_id": "local-player",
        "decision": {"role": "defender", "decision": {"type": "pause"}}
    });
    assert!(serde_json::from_value::<Command>(forged).is_err());
}

#[test]
fn attacker_cannot_defend_target_or_control_another_campaign() {
    let mut w = player_campaign();
    let before = serde_json::to_string(&w).unwrap();
    assert!(w
        .apply(
            &principal("red-player"),
            Command::defend(w.identity.network_id.clone(), Action::RotateCredentials)
        )
        .is_err());
    let cmd = Command {
        network_id: w.identity.network_id.clone(),
        decision: Decision::Attacker {
            campaign_id: w.campaign.as_ref().unwrap().id,
            operation: AttackOperation::EnumeratePublicService,
        },
    };
    assert!(w.apply(&principal("other-attacker"), cmd).is_err());
    assert!(w
        .attacker_view(&principal("other-attacker"), &w.identity.network_id, 1)
        .is_err());
    assert!(w
        .attacker_view(&Principal::local_operator(), &w.identity.network_id, 1)
        .is_err());
    assert!(w
        .attacker_view(&principal("red-player"), &w.identity.network_id, 2)
        .is_err());
    assert_eq!(before, serde_json::to_string(&w).unwrap());
}

#[test]
fn npc_campaign_cannot_be_claimed_by_a_player() {
    let mut w = player_campaign();
    w.campaign.as_mut().unwrap().controller = CampaignController::Npc;
    assert!(queue(&mut w, AttackOperation::EnumeratePublicService).is_err());
}

#[test]
fn queue_validates_stage_and_has_no_immediate_effect() {
    let mut w = player_campaign();
    let before = serde_json::to_string(&w).unwrap();
    assert!(queue(&mut w, AttackOperation::TransferRecords).is_err());
    assert_eq!(before, serde_json::to_string(&w).unwrap());
    for _ in 0..30 {
        w.tick();
    }
    assert_eq!(w.events.len(), 6); // starter + routine traffic, no attacker activity
    queue(&mut w, AttackOperation::EnumeratePublicService).unwrap();
    let queued = serde_json::to_string(&w).unwrap();
    assert!(queue(&mut w, AttackOperation::EnumeratePublicService).is_err());
    assert_eq!(queued, serde_json::to_string(&w).unwrap());
    for _ in 0..7 {
        w.tick();
    }
    assert_eq!(
        w.campaign.as_ref().unwrap().stage,
        AttackStage::Reconnaissance
    );
    w.tick();
    assert_eq!(
        w.campaign.as_ref().unwrap().stage,
        AttackStage::InitialAccess
    );
    assert!(w.campaign.as_ref().unwrap().current_access.is_empty());
}

#[test]
fn player_and_npc_operations_produce_identical_defender_evidence() {
    let mut player = player_campaign();
    let mut npc = player.clone();
    npc.campaign.as_mut().unwrap().controller = CampaignController::Npc;
    run_player(&mut player, 85);
    for _ in 0..85 {
        npc.tick();
    }
    assert_eq!(
        serde_json::to_value(&player.events).unwrap(),
        serde_json::to_value(&npc.events).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&player.alerts).unwrap(),
        serde_json::to_value(&npc.alerts).unwrap()
    );
    assert_eq!(player.reports[0].data_lost_mb, npc.reports[0].data_lost_mb);
}

#[test]
fn defender_changes_take_effect_during_a_queued_player_operation() {
    let mut w = player_campaign();
    run_player(&mut w, 8);
    queue(&mut w, AttackOperation::AttemptInitialAccess).unwrap();
    w.apply(
        &Principal::local_operator(),
        Command::defend(w.identity.network_id.clone(), Action::RotateCredentials),
    )
    .unwrap();
    for _ in 0..20 {
        w.tick();
    }
    assert!(w.campaign.is_none());
    assert!(w.footholds.is_empty());
}

#[test]
fn independent_defender_contains_a_player_foothold() {
    let mut w = player_campaign();
    run_player(&mut w, 29);
    let red = principal("red-player");
    let view = w.attacker_view(&red, &w.identity.network_id, 1).unwrap();
    assert_eq!(view.observed_access, vec!["WEB-01"]);
    w.apply(
        &Principal::local_operator(),
        Command::defend(
            w.identity.network_id.clone(),
            Action::Isolate {
                host: "WEB-01".into(),
            },
        ),
    )
    .unwrap();
    assert!(w.campaign.is_none());
    assert!(w.attacker_view(&red, &w.identity.network_id, 1).is_err());
    assert_eq!(w.reports[0].data_lost_mb, 0);
}

#[test]
fn attacker_projection_does_not_leak_an_unseen_host_or_defender_controls() {
    let mut w = player_campaign();
    w.cash = 500 * DOLLAR;
    w.act(Action::Buy {
        kind: "database".into(),
    })
    .unwrap();
    w.act(Action::Deploy {
        host: "DB-01".into(),
    })
    .unwrap();
    w.act(Action::Isolate {
        host: "DB-01".into(),
    })
    .unwrap();
    run_player(&mut w, 53);
    let view = w
        .attacker_view(&principal("red-player"), &w.identity.network_id, 1)
        .unwrap();
    assert!(!view.known_hosts.contains(&"DB-01".into()));
    let json = serde_json::to_value(view).unwrap();
    for key in [
        "servers",
        "network",
        "cash",
        "events",
        "alerts",
        "detection",
        "controller",
        "persistence",
        "discovered_credentials",
    ] {
        assert!(json.get(key).is_none(), "leaked {key}");
    }
    assert!(!json.to_string().contains("DB-01"));
}

#[test]
fn hosted_mode_cannot_pause_or_use_training_oracles() {
    let mut w = player_campaign();
    run_player(&mut w, 85);
    w.identity.mode = WorldMode::Hosted;
    for action in [
        Action::Pause,
        Action::Practice {
            kind: "brute_force".into(),
        },
    ] {
        assert!(w
            .apply(
                &Principal::local_operator(),
                Command::defend(w.identity.network_id.clone(), action)
            )
            .is_err());
    }
    w.paused = true;
    let tick = w.tick;
    w.tick();
    assert_eq!(w.tick, tick + 1);
    let view = w
        .defender_view(&Principal::local_operator(), &w.identity.network_id)
        .unwrap();
    assert!(!view.paused);
    assert!(view.reports.is_empty());
    assert!(!view
        .events
        .iter()
        .any(|e| e.event == EventKind::CampaignEnded));
    w.apply(
        &Principal::local_operator(),
        Command::defend(
            w.identity.network_id.clone(),
            Action::Isolate {
                host: "WEB-01".into(),
            },
        ),
    )
    .unwrap();
    // Hosted reconnect must not reveal whether hidden endpoint persistence exists.
    w.apply(
        &Principal::local_operator(),
        Command::defend(
            w.identity.network_id.clone(),
            Action::Resume {
                host: "WEB-01".into(),
            },
        ),
    )
    .unwrap();
    assert!(!w.footholds.is_empty());
    assert!(w.alerts.iter().any(|a| !a.resolved));
}

#[test]
fn the_same_player_can_defend_one_network_and_attack_another() {
    let red = principal("red-player");
    let mut home = World::default();
    home.identity.network_id = NetworkId("red-home".into());
    home.identity.owner = PlayerId("red-player".into());
    home.apply(
        &red,
        Command::defend(
            home.identity.network_id.clone(),
            Action::Patch {
                host: "WEB-01".into(),
            },
        ),
    )
    .unwrap();
    let mut target = player_campaign();
    queue(&mut target, AttackOperation::EnumeratePublicService).unwrap();
    assert!(home.servers[0].maintenance_kind.is_some());
    assert!(target.campaign.unwrap().pending_operation.is_some());
}

#[test]
fn player_lateral_movement_obeys_the_defenders_segmentation() {
    for segmented in [false, true] {
        let mut w = player_campaign();
        w.cash = 1000 * DOLLAR;
        for action in [
            Action::Buy {
                kind: "database".into(),
            },
            Action::Deploy {
                host: "DB-01".into(),
            },
        ] {
            w.apply(
                &Principal::local_operator(),
                Command::defend(w.identity.network_id.clone(), action),
            )
            .unwrap();
        }
        if segmented {
            w.apply(
                &Principal::local_operator(),
                Command::defend(w.identity.network_id.clone(), Action::Segment),
            )
            .unwrap();
        }
        run_player(&mut w, 61);
        assert_eq!(
            w.campaign
                .as_ref()
                .unwrap()
                .affected_hosts
                .contains(&"DB-01".into()),
            !segmented
        );
        if segmented {
            assert!(w
                .events
                .iter()
                .any(|e| e.event == EventKind::FirewallBlock && e.host == "DB-01"));
        }
    }
}

#[test]
fn pending_player_operation_survives_snapshot_reload() {
    let mut w = player_campaign();
    queue(&mut w, AttackOperation::EnumeratePublicService).unwrap();
    for _ in 0..4 {
        w.tick();
    }
    let mut loaded: World = serde_json::from_str(&serde_json::to_string(&w).unwrap()).unwrap();
    for _ in 0..4 {
        w.tick();
        loaded.tick();
    }
    assert_eq!(
        serde_json::to_value(w).unwrap(),
        serde_json::to_value(loaded).unwrap()
    );
}

#[test]
fn old_local_saves_default_to_local_ownership_and_npc_control() {
    let mut w = World::default();
    w.start_campaign("brute_force").unwrap();
    let mut legacy = serde_json::to_value(w).unwrap();
    legacy.as_object_mut().unwrap().remove("identity");
    let campaign = legacy["campaign"].as_object_mut().unwrap();
    campaign.remove("controller");
    campaign.remove("pending_operation");
    let loaded: World = serde_json::from_value(legacy).unwrap();
    assert_eq!(loaded.identity.mode, WorldMode::LocalTraining);
    assert!(matches!(
        loaded.campaign.unwrap().controller,
        CampaignController::Npc
    ));
}
