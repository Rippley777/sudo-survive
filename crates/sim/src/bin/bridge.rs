//! Loopback-only browser development transport; the desktop uses native IPC.
use std::io::Read;
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use sudo_survive_sim::{storage::Store, Action, World};
use tiny_http::{Header, Method, Response, Server};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    std::fs::create_dir_all(".dev")?;
    let path =
        std::env::var("SUDO_SURVIVE_DB").unwrap_or_else(|_| ".dev/sudo-survive.sqlite".into());
    let mut store = Store::open(path)?;
    let world = store.load()?;
    store.save(&world)?;
    let shared = Arc::new(Mutex::new((world, store, None::<String>)));
    let ticker = shared.clone();
    let tick_ms = std::env::var("SUDO_SURVIVE_TICK_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(1000)
        .clamp(20, 10000);
    std::thread::spawn(move || {
        let interval = Duration::from_millis(tick_ms);
        let mut deadline = Instant::now() + interval;
        loop {
            std::thread::sleep(deadline.saturating_duration_since(Instant::now()));
            let mut state = ticker.lock().unwrap();
            let (world, store, error) = &mut *state;
            world.tick();
            if world.tick.is_multiple_of(5) {
                *error = store.save(world).err();
            }
            deadline += interval;
            if deadline < Instant::now() {
                deadline = Instant::now() + interval;
            }
        }
    });
    let server = Server::http("127.0.0.1:4318")?;
    println!("Rust simulation bridge listening on http://127.0.0.1:4318");
    for mut request in server.incoming_requests() {
        let mut state = shared.lock().unwrap();
        let (world, store, save_error) = &mut *state;
        let result: Result<serde_json::Value, String> = match (request.method(), request.url()) {
            (&Method::Get, "/api/state") => match save_error {
                Some(e) => Err(format!("Autosave failed: {e}")),
                None => Ok(serde_json::to_value(world.view())?),
            },
            (&Method::Post, "/api/action") => {
                let mut body = String::new();
                request.as_reader().take(4096).read_to_string(&mut body)?;
                serde_json::from_str::<Action>(&body)
                    .map_err(|e| e.to_string())
                    .and_then(|a| {
                        // Transaction-like memory semantics: a failed save never reports success.
                        let mut next: World = world.clone();
                        next.act(a)?;
                        store.save(&next)?;
                        *world = next;
                        *save_error = None;
                        Ok(serde_json::to_value(world.view()).unwrap())
                    })
            }
            (&Method::Post, "/api/search") => {
                let mut body = String::new();
                request.as_reader().take(4096).read_to_string(&mut body)?;
                serde_json::from_str::<String>(&body)
                    .map_err(|e| e.to_string())
                    .and_then(|q| sudo_survive_sim::query::search(&world.events, &q))
                    .map(|v| serde_json::to_value(v).unwrap())
            }
            _ => Err("Unknown endpoint".into()),
        };
        let (status, body) = match result {
            Ok(v) => (200, v),
            Err(e) => (400, serde_json::json!({ "error": e })),
        };
        let response = Response::from_string(body.to_string())
            .with_status_code(status)
            .with_header(Header::from_bytes("Content-Type", "application/json").unwrap());
        let _ = request.respond(response);
    }
    Ok(())
}
