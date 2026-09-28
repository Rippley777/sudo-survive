# sudo survive

A tiny data center for the edge of your desktop, designed for a persistent
cybersecurity MMO where every player can attack and defend fictional networks.
The current playable build is **local training**, built with Tauri, Rust, React,
TypeScript, and SQLite. There are no hosted accounts or live PvP yet.

The simulation now has scoped player/network identities, an authorized command/view
boundary, and queued attacker operations that use the same resolver and telemetry
as NPC campaigns. These are library foundations, tested in a two-principal harness;
they are not a player-facing offensive UI or an authenticated backend. The planned
Rust/PostgreSQL host will own online state; local saves will never import money,
inventory, attack results, or intelligence into the MMO.

## Run the desktop app

Requires Node 20+, npm, Rust via rustup, and the
[Tauri prerequisites for your platform](https://v2.tauri.app/start/prerequisites/).
The repository pins Rust 1.90; rustup installs the project toolchain automatically.

```sh
npm ci
npm run desktop
```

Starts a compact, always-on-top native window. Use the top-right controls for
bottom docking, right docking, floating, or collapsed mode. Drag the title bar to
move it. Click the pin to toggle always-on-top. Open the security console to expand
the window. Window size and position are restored across launches. Quit through
the OS application menu / standard quit shortcut. The app does not reserve screen
space or move other applications.

To build a standalone package:

```sh
npm run desktop:build
# Faster local macOS bundle:
npm run desktop:build -- --debug --bundles app
```

The macOS debug build is at `target/debug/bundle/macos/sudo survive.app`.
It is a local development build, not signed or notarized for distribution.
Windows and Linux are configured through Tauri but have not yet been tested on
their native platforms.

## Play the first loop

1. WEB-01 starts online, earning $10/min net. Your balance starts at $160.
2. **Add server**, buy a server, then **Deploy service** in its asset workbench.
3. The first attacker starts at 00:00:45. The first authentication alert normally
   appears at 00:01:03. Subsequent attacks have a four-minute cooldown.
4. **Investigate**. Read the source, account, failed/successful authentications,
   endpoint activity, and network connections. Search with, for example:

   ```text
   event = AUTH_LOGIN_FAILED AND user = deploy
   source_ip = 192.0.2.41
   host = WEB-01
   ```

5. Before a successful login, **Block source** can prevent entry. After compromise,
   blocking that IP alone does not revoke a session. **Isolate host** stops its
   network traffic and income. If access has spread, contain every affected host.
6. **Rotate credential**, **Restore · $20**, wait 15 simulated seconds, then
   **Resume host**. Patch exposed web services and segment internal administration
   to make future incidents smaller.
7. Read **Incident review** for the timeline and separate operational metrics.

The pause button freezes everything, including maintenance. The field guide has
three practice incident types if you don't want to wait. Exercises affect the same
world. All traffic and attacks are fictional, generated locally; no real systems
are scanned or contacted.

## Browser development

```sh
npm run dev:browser
```

Open `http://127.0.0.1:1420`. Vite proxies to a loopback-only Rust bridge on port 4318. This runs the same engine as the desktop app, with a separate SQLite save.
The bridge is a development tool; the native app uses Tauri IPC and opens no HTTP
listener. Native positioning controls change the layout in a browser but cannot
move the browser window. Restart the bridge after Rust changes; React hot-reloads.

## Saves

- macOS: `~/Library/Application Support/dev.sudosurvive.desktop/world.sqlite`
- Other desktops: Tauri's platform application-data directory, same app identifier.
- Browser development: `.dev/sudo-survive.sqlite`.
- Saves are transactional, every five ticks and after every successful action;
  the desktop also saves on graceful quit. A crash can lose up to five seconds.
- Closing the application freezes simulated time. Offline catch-up is deferred.
- This applies to local training only. Hosted networks will continue while clients
  are away; client clocks, pause controls, and SQLite saves will have no authority.
- Save errors are shown; incompatible/corrupt saves are not silently replaced.
- To start over, quit the app and move its database and any `-wal`/`-shm` sidecars
  into a backup directory. The next launch creates a fresh world.

## Checks

```sh
npm test
npm run build
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p sudo-survive-sim --no-default-features
npx playwright install chromium
npm run test:ui
```

UI tests start an isolated, in-memory Rust world at four simulated seconds per real
second. Stop any development servers using ports 1420/4318 first. They cover the
complete incident-response loop, purchase/deployment, state across page reload,
query errors, and compact/sidebar/collapsed layouts. Rust tests cover causal attack
constraints, deterministic replay, persistence, and recovery. Page reload checks
are separate from the Rust test that verifies SQLite save/load continuity.
Authority tests cover cross-network access rejection, campaign-controller checks,
projection isolation, NPC/player event equivalence, independent defensive responses,
queued-operation reload, and compatibility with existing local saves.

## Layout and scope

- [GAME_DESIGN.md](GAME_DESIGN.md): pillars, playable loop, tradeoffs, progression.
- [TECHNICAL_DESIGN.md](TECHNICAL_DESIGN.md): boundaries, models, event pipeline,
  state machine, persistence, and extension points.
- `crates/sim/src`: engine, models, query language, SQLite repository, causal tests.
- `crates/sim/src/authority.rs`: principal/ownership boundary, typed campaign
  decisions, and separate owner/attacker projections. Authentication remains external.
- `crates/sim/data/catalog.json`: three service types and three attack entry paths.
- `crates/sim/migrations/001_initial.sql`: initial versioned SQLite schema.
- `src-tauri`: desktop scheduler, commands, window state, and packaging.
- `src`: observable world rendering and player controls.

This is the first local vertical slice of the MMO design. Hosted authority,
recon/fog-of-war progression, asynchronous PvP, anti-griefing policies, and online
recovery guarantees remain future work. General firewall editing, benign alert noise,
detector tuning, advanced EDR views, backup scheduling, offline progress, attacker
learning, employees, and response automation are future work. Network segmentation
currently denies DMZ-to-internal SSH; the shared deployment identity intentionally
demonstrates the risk of overly broad authorization. Recovery uses a clean immutable
starter backup. Incident metrics are measured at campaign close, with ongoing
asset downtime visible in the workbench.
