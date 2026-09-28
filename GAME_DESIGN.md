# sudo survive

A tiny, living data center at the edge of your desktop. Build a profitable homelab,
follow evidence when something looks wrong, and engineer your way toward quieter days.

## Pillars

- Fun and calm first. Income, packets, rack lights, and backups continue without clicks.
- Evidence before answers. Alerts describe observations; campaign truth is hidden until review.
- Security is causal. Connectivity, account scope, credential versions, and isolation constrain attacks.
- Uptime matters. Containment stops revenue; patching takes time; recovery requires a clean backup.
- Safe realism. All addresses, identities, traffic, and attacker actions are fictional and internal.

## First playable slice

Start with $160, WEB-01 hosting a small site, and a basic perimeter firewall. Buy a
web ($90), database ($150), or storage ($120) server; explicitly deploy its service.
Revenue and operating costs use integer microdollars, once per simulated second.
The first attacker arrives after 45 seconds. Subsequent campaigns are spaced several
minutes apart. A labeled practice control starts an incident without waiting.

Observe repeated login failures. A threshold detector creates an alert, not a diagnosis.
Open the console, filter events, correlate a successful login with endpoint and network
activity, then act. Blocking an external source prevents new access; an established
session and persistence survive that block. Isolate affected hosts, rotate the deploy
credential, restore a clean backup, and resume service. The review shows evidence,
time to detection/containment, affected hosts, data loss, and lost revenue.

Three entry paths share a campaign engine but have distinct prerequisites/telemetry:
password guessing (noisy failures), stolen credentials (direct success), and an
unpatched web service (HTTP plus process evidence). Persistence, lateral movement,
collection, and exfiltration follow. A deploy account is initially overprivileged.
Segmentation denies web-to-internal administration, demonstrating why topology matters.

## Controls and tradeoffs

| Control                | Benefit                                                  | Cost or limit                                 |
| ---------------------- | -------------------------------------------------------- | --------------------------------------------- |
| Block source           | Denies future ingress from that address                  | Does not remove existing access               |
| Isolate host           | Denies all host network paths                            | Service revenue stops; operating cost remains |
| Disable deploy account | Revokes sessions and prevents account authentication     | Does not remove endpoint persistence          |
| Rotate credentials     | Invalidates stolen credential versions and weak password | Existing sessions/persistence remain          |
| Patch                  | Removes web weakness                                     | 12 seconds service downtime; not eradication  |
| Restore clean backup   | Removes access and persistence on isolated host          | $20 and 15 seconds downtime                   |
| Segment network        | Denies DMZ → internal administration                     | $40 architecture investment                   |

The starter backup is immutable and clean; restores preserve current patches and
credential policy (a deliberately simplified data/endpoint restore). Later backup
designs will expose recovery-point age, unprotected data, and online backup compromise.

## Presentation

Charcoal, warm white, desaturated teal, and restrained amber. Small physical racks,
animated packet paths, and human-readable telemetry. Compact bottom dock by default;
right dock, floating, and collapsed modes. Investigation expands the native window.
Window geometry and the world survive restarts. Pausing is available; closing the
app freezes simulated time. Offline attacks are deliberately deferred.

## Progression beyond the slice

Asset inventory → segmentation → EDR and vulnerability scanning → scoped identities
and MFA → backup architecture → detection tuning → conditional response automation.
Specialists do specific work (triage duplicates, hunt persistence, configure rules),
never generic security multipliers. Quiet, patient operators and insiders arrive only
after players have tools to understand their evidence. Add benign alerts, tuning,
DDoS capacity pressure, ransomware recovery, and supply-chain campaigns as distinct
mechanics, not renamed brute-force events.

## Success criteria

The game stays readable while working elsewhere. A new player can buy and deploy a
server, investigate an alert, contain the appropriate host, recover it, and explain
the incident from the timeline. A blocked lateral movement event should make network
segmentation feel useful. No single arbitrary security score replaces the facts.
