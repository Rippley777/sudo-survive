#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};
use sudo_survive_sim::authority::{Command, Principal};
use sudo_survive_sim::{storage::Store, Action, SecurityEvent, World, WorldView};
use tauri::{Manager, State};

struct Runtime {
    world: World,
    store: Store,
    save_error: Option<String>,
}
#[tauri::command]
fn get_world(state: State<Mutex<Runtime>>) -> Result<WorldView, String> {
    let s = state.lock().map_err(|e| e.to_string())?;
    if let Some(e) = &s.save_error {
        return Err(format!("Autosave failed: {e}"));
    }
    s.world
        .defender_view(&Principal::local_operator(), &s.world.identity.network_id)
}
#[tauri::command]
fn act(action: Action, state: State<Mutex<Runtime>>) -> Result<WorldView, String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    let mut next = s.world.clone();
    next.apply(
        &Principal::local_operator(),
        Command::defend(next.identity.network_id.clone(), action),
    )?;
    s.store.save(&next)?;
    s.world = next;
    s.save_error = None;
    s.world
        .defender_view(&Principal::local_operator(), &s.world.identity.network_id)
}
#[tauri::command]
fn search_logs(query: String, state: State<Mutex<Runtime>>) -> Result<Vec<SecurityEvent>, String> {
    let s = state.lock().map_err(|e| e.to_string())?;
    s.world.defender_logs(
        &Principal::local_operator(),
        &s.world.identity.network_id,
        &query,
    )
}
fn main() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            let path = app.path().app_data_dir()?;
            std::fs::create_dir_all(&path)?;
            let mut store =
                Store::open(path.join("world.sqlite")).map_err(std::io::Error::other)?;
            let world = store.load().map_err(std::io::Error::other)?;
            store.save(&world).map_err(std::io::Error::other)?;
            app.manage(Mutex::new(Runtime {
                world,
                store,
                save_error: None,
            }));
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let mut deadline = Instant::now() + Duration::from_secs(1);
                loop {
                    std::thread::sleep(deadline.saturating_duration_since(Instant::now()));
                    let state = handle.state::<Mutex<Runtime>>();
                    let mut runtime = state.lock().unwrap();
                    let Runtime {
                        world,
                        store,
                        save_error,
                    } = &mut *runtime;
                    world.tick();
                    if world.tick.is_multiple_of(5) {
                        *save_error = store.save(world).err();
                    }
                    deadline += Duration::from_secs(1);
                    // OS sleep freezes the world; don't replay missed overnight ticks.
                    if deadline < Instant::now() {
                        deadline = Instant::now() + Duration::from_secs(1);
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_world, act, search_logs])
        .build(tauri::generate_context!())
        .expect("Unable to launch sudo survive");
    app.run(|handle, event| {
        if matches!(event, tauri::RunEvent::ExitRequested { .. }) {
            let state = handle.state::<Mutex<Runtime>>();
            let mut runtime = state.lock().unwrap();
            let Runtime { world, store, .. } = &mut *runtime;
            if let Err(e) = store.save(world) {
                eprintln!("Save on exit failed: {e}");
            }
        }
    });
}
