# Technical design

## Boundaries

`crates/sim`: pure deterministic Rust world, typed commands/events, campaign engine,
catalog, query parser, tests, and SQLite repository. No desktop or UI dependency.
`src-tauri`: native window, one-second scheduler, command boundary, save lifecycle.
`src`: React/TypeScript rendering, local selection/search state, native window modes.
A loopback-only development bridge runs the **same Rust engine** for browser testing;
there is no second JavaScript simulation. It is excluded from desktop runtime.

React submits typed actions; Rust validates resources and prerequisites. A single
mutex serializes ticks/actions/saves. The frontend polls a redacted view, never the
live campaign or hidden foothold state. SQLite snapshots contain full private state.
The scheduler uses monotonic deadlines, independent of webview focus or visibility.
One tick = one simulated second; pause/closed application means no progression.
No wall-clock offline catch-up in this slice.

## Models

World owns Server, ServiceSpec, FirewallRule, User, Credential, Attacker,
AttackCampaign, SecurityEvent, Alert, Backup, DetectionRule, and IncidentReport.
Server types and entry-path timings live in `crates/sim/data/catalog.json`.
Adding a catalog service or tuning a known entry path is data-only; a genuinely
new capability requires a small Rust mechanic and causal tests.

Stable IDs and ordered vectors keep iteration deterministic. Money uses integer
microdollars with per-minute remainder carry. A seeded PRNG selects targets and
entry paths; seed/counters/campaign stage are saved. Server capacity is eight for
the slice. Telemetry is capped at 600 recent events; incident evidence has its own
bounded history. Reports and alerts are bounded too. No unbounded per-tick history.

## Event pipeline

Economy / maintenance / synthetic traffic / attacks → typed event append →
threshold and correlation detectors → observable alerts → view projection.
Events carry monotonic ID, simulated timestamp, asset, source, destination, user,
kind, and message. Credentials are fictional version counters, never real secrets.
AUTH_LOGIN_FAILED, AUTH_LOGIN_SUCCESS, HTTP_REQUEST, PROCESS_STARTED, FILE_CREATED,
NETWORK_CONNECTION, FIREWALL_BLOCK, DATA_TRANSFER, and response/maintenance events
form the initial vocabulary. Ordinary success/HTTP traffic is interleaved with
attack evidence. Detectors consume telemetry, not hidden campaign stage.

SIEM queries use `field = value AND field = value`, optionally quoted. Supported
fields: event, source_ip, host, user. Matching is case-insensitive and conjunctive;
invalid syntax produces an explicit error, never silent empty results.

## Campaign state machine

Reconnaissance → InitialAccess → Execution → Persistence → Discovery →
LateralMovement → Collection → Exfiltration → Complete.
Any active stage can become Contained when no reachable footholds remain.
Access is tracked per host with session credential version and persistence state;
known hosts and stolen credentials constrain lateral movement. Entry requirements
are explicitly evaluated: exposed service + firewall route + applicable weakness.
Lateral access requires route + deployed service + enabled account + valid stolen
credential + account authorization for the destination. Segmentation denies the
administrative path to internal hosts. Exfiltration requires an unisolated foothold
and an allowed outbound route. Isolation halts all paths. Blocking ingress after
compromise does not revoke the endpoint foothold. Account disable revokes sessions,
but persistence survives; rotation stops credential reuse, not existing sessions.

The first campaign is deterministic password guessing, with a deliberately early
alert. Later entry paths vary. One active campaign in the slice; persistent attacker
identities retain attempt counts. Learning/adaptation and concurrent attackers are
future work. Actor metadata never makes movement bypass the network model.

## Persistence

`crates/sim/migrations/001_initial.sql` is the initial SQLite schema. One versioned
world snapshot is authoritative; normalized event/report projections support local
inspection. A transaction replaces snapshot and projections atomically. WAL mode,
foreign keys, busy timeout, schema version, and strict version validation are enabled.
Autosave every five ticks, after successful commands, and on graceful exit.
Load failures are surfaced rather than silently overwriting a user's saved world.
Desktop database lives in the platform app-data directory; browser development uses
`.dev/sudo-survive.sqlite`. Native window geometry is saved separately by the window
state plugin. No network permissions, real scans, shell execution, or exploit code.

## Verification and scope

Rust tests cover deterministic replay, income, purchase validation, detector timing,
early source block, late source block, isolation, credential invalidation,
segmentation/lateral access, restoration, query parsing, and SQLite round trips.
TypeScript is checked and production assets built. Browser integration exercises
the real Rust bridge; native compilation checks the Tauri boundary.

The slice supports three service types and three initial access variants, one
global deploy identity, a simple directional segmentation policy, clean recovery,
and evidence-based incident reports. Full firewall editing, custom detection rules,
backup scheduling, employee automation, large content libraries, and OS work-area
reservation are deferred. Docking positions an ordinary always-on-top window; it
does not reserve space or move other applications. Windows/Linux packaging still
requires native CI and desktop testing on those platforms.

## References

- [Tauri command interface](https://v2.tauri.app/develop/calling-rust/)
- [Tauri configuration](https://v2.tauri.app/reference/config/)
- [Tauri platform prerequisites](https://v2.tauri.app/start/prerequisites/)
