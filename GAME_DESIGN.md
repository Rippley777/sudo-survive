# sudo survive

A tiny data center at the edge of your desktop, living inside a persistent online
cybersecurity ecosystem. Build something worth defending. Investigate incomplete
evidence. Sometimes go hunting in someone else's **simulated** network.

**The destination is an MMO.** Every player can operate as both attacker and
defender; there are no permanent red/blue classes. The playable build is still a
local training slice. Multiplayer systems below are design requirements, not
claims of shipped features.

## Pillars

- Calm, passive infrastructure with occasional slow, consequential strategic duels.
- Outcomes come from configuration, connectivity, access, and decisions. No cyber
  power rating or generic “hack: 62% chance” button.
- Evidence before answers on both sides. An alert does not reveal NPC versus player,
  the attacker's identity, their objective, or all compromised machines.
- One simulation: NPC and player operations produce the same defender telemetry
  and meet the same security controls.
- Uptime, money, complexity, and security compete. Architecture and preparation
  matter more than constant attention.
- Recoverable losses. Tension must not erase months of progress or punish having a job.
- Everything offensive stays inside the game. Targets are server-issued game
  identifiers; addresses, services, credentials, and data are fictional. No payloads,
  real target input, real scanning, peer-to-peer connections, or device exposure.

## The two connected loops

**Blue team:** build → deploy → earn → harden → detect → investigate → contain →
eradicate → recover → review → automate. Firewalls, MFA, scoped identities,
patches, segmentation, logging, EDR, and backups enforce actual simulated rules.

**Red team:** discover an eligible organization → recon → enumerate exposed services
→ interpret fingerprints and authentication behavior → investigate a fictional
weakness → attempt access → discover reachable hosts → obtain appropriate access
→ move laterally → pursue an objective → withdraw or maintain a foothold.

Players choose game-native operations such as service enumeration, an abstract
credential attempt, or probing a fictional web weakness. They choose a target,
technique, timing, and resource budget; the server resolves the result against the
defender's current configuration. A failed route or invalid credential stays a
failed route or invalid credential, regardless of reputation.

## A miniature internet

The directory contains pseudonymous player organizations and fictional NPC small
businesses, corporations, cloud providers, and training environments. Start with
NPC offensive training: discover a public service, establish access, find DB-01,
meet the route and credential requirements, and retrieve fictional records.
Serious PvP unlocks after this training and after protective controls are available.

The directory reveals a limited public footprint, not a complete topology. Recon
can reveal service fingerprints, exposed ports, host aliases, fictional employee
identities, authentication surfaces, and inferred network relationships. Each
observation has provenance, observation time, and freshness. Old intelligence stays
old when a defender patches or rebuilds; it is not silently refreshed by the UI.
Discovery may be incomplete or wrong. Unknown is not equivalent to absent.

## Fog of war and stealth

| Participant | Sees                                                                 | Does not automatically see                                                                        |
| ----------- | -------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| Defender    | Owned inventory/configuration, collected telemetry, detection alerts | Attacker player/NPC identity, campaign state, intent, undetected access/persistence               |
| Attacker    | Public observations, acquired intel, own operation results           | Full topology, defender balance, rules, EDR/honeypot flags, alert state, exact containment action |
| Server      | Authoritative state and action provenance                            | Never shares that entire state with either client                                                 |

Fast enumeration covers more services and generates more observable connections.
Passive recon is slower and reveals only public/cached facts. Noisy authentication
attempts can cross a detector threshold quickly; spaced attempts take longer and
can still be correlated. Stealth changes time, coverage, and emitted evidence; it
is not a stat that makes traffic invisible or bypasses access controls.

Both players online should feel like chess through telemetry. A player sees a
successful login, isolates WEB-02, and the attacker observes a lost connection on
its next operation or heartbeat. That feedback does not explain which defensive
control fired or confirm whether another foothold was discovered.

## Campaign objectives

| Objective                    | Distinct pressure and counterplay                                                                      |
| ---------------------------- | ------------------------------------------------------------------------------------------------------ |
| Data theft                   | Collect valuable fictional records and traverse an allowed egress path; quotas cap losses              |
| Espionage / credential theft | Acquire scoped, expiring intelligence or simulated credentials; rotation invalidates reuse             |
| Mining                       | Consume spare compute over time; utilization/process evidence competes with covert income              |
| Botnet / persistence         | Maintain limited, costly footholds; integrity checks, endpoint hunting, and clean recovery remove them |
| Sabotage                     | Temporarily disrupt a service; redundancy, restart policy, and isolation limit impact                  |
| Ransomware                   | Lock simulated workloads under a loss/time cap; immutable recovery remains available                   |
| Bounty                       | Fulfill a verifiable contract on an eligible target; no exemption from protection rules                |

The local slice implements data collection/transfer through three initial entry
paths. Mining, botnets as a continuing objective, ransomware, espionage markets,
and bounties require their own mechanics before release.

## Persistence, deception, and attribution

Blocking an entry IP is containment of a path, not proof of eradication. Durable
footholds can outlive a connection and, within campaign budgets and expiry limits,
permit later reconnection. Rotation invalidates stolen credentials; it does not
remove every startup task. A clean restore or appropriate eradication removes the
specific persistence. Persistence must remain within immunity and recovery rules;
it cannot be used to bypass post-compromise protection indefinitely.

Defenders inspect process relationships, file changes, authentication, and flows;
then hunt related hosts and validate recovery. Online postmortems distinguish
confirmed facts, hypotheses, and unknowns. They must not expose hidden persistence
or certify a clean system simply because a campaign was marked contained. The
local training debrief may disclose ground truth to teach the lesson.

Honeypots present plausible services and valuable-looking hostnames. Interaction
produces high-quality evidence, not an automatic real player identity. Attackers
can notice clues; recon never receives a hidden `is_honeypot` flag.

Persistent attacker identities support recognizable behavior and reputation.
Attribution links observed techniques, fictional infrastructure, and historical
fingerprints, with provenance and uncertainty. Confidence is derived from evidence
and calibrated; do not display an arbitrary “78%” as if it were established fact.
No real IP, account email, machine name, or other device information enters attribution.

## Idle, asynchronous, and fair

Online infrastructure continues earning and producing telemetry when a desktop
closes. Bounded queued campaign plans continue on the authoritative server.
Configured firewall, MFA, rate limits, lockouts, EDR, SIEM, and response automation
remain active. Returning players receive a timeline of detection, automated
containment, blocked movement, affected services, losses, and required recovery.
Disconnecting or changing the local clock does not dodge an active campaign.
The local sandbox alone can pause and freezes while closed.

Before PvP release, require these protections. Values are initial playtest targets,
not final balance or implemented backend behavior:

- New players remain PvP-protected through training and until an explicit readiness
  choice. NPC play remains viable; voluntary high-risk zones are a later option.
- Match on infrastructure value and experience bands, using recent peaks to resist
  deliberate down-ranking. Reputation informs eligibility, not combat power.
- Start with one incoming and one outgoing PvP campaign per player/network; reserve
  slots atomically. Aggregate protections across organizations and related accounts.
- Charge upfront campaign resources and upkeep. Repeated attacks on the same target
  have diminishing rewards and escalating cost; successful repeats give zero reward
  during a rolling cooldown. Switching alts must not reset these controls.
- Pilot a 24-hour shield after serious compromise. It suspends hostile foothold use
  and new campaigns, cannot be cancelled by an attacker, and blocks outbound PvP
  while active. Reconnection and queued jobs recheck it.
- Cap total daily resource loss (initial target: 5% of eligible liquid reserves)
  across all attackers. Never delete purchased infrastructure or progression.
  Reserve a protected recovery fund and a basic clean recovery path even at $0.
- Insurance offsets bounded downtime/loss without creating a profitable collusion
  loop. Freeze rewards from self-attacks, circular transfers, and repeated collusion
  for review. Bounties and markets use the same limits.

Measure return-session recovery time, losses while away, repeat-target concentration,
false positives, and automation mistakes. Change the rules if the optimal strategy
becomes remaining online all day.

## Economy, information, and social play

Legitimate hosting provides dependable income. Contracts, defensive work, consulting,
bounties, and offensive objectives add distinct earning paths. Costs include
hardware, bandwidth, cloud capacity, backup storage, SIEM ingestion, EDR, staff,
insurance, and attack infrastructure. Offensive profits must have explicit sources,
caps, and sinks; stealing fictional records does not mint unlimited currency.

Information is a resource: service observations, scoped stolen credentials, topology
inferences, defensive behavior, and attacker indicators may expire, be shared, or
be traded. Transactions transfer only the authorized observation and its provenance,
not perpetual access to the target's live state. Rotated credentials remain invalid
in sold copies. Credential capabilities stay inside the simulation.

Later marketplaces support hosting, backup storage, monitoring, incident response,
threat intelligence, and explicitly consented assessments. Service providers receive
scoped, revocable access; buying monitoring never grants hidden owner privileges.
Organizations can share infrastructure through roles for building, detection,
hunting, and offense. Membership changes revoke access. Crews, SOC teams, MSPs, and
security firms emerge through responsibilities, not permanent player classes.

Track attacker reputation, defender reputation, incident-response experience,
threat hunting, stealth outcomes, infrastructure value, and notoriety separately.
They unlock complexity, establish credibility, and influence matchmaking, never
add generic attack/defense multipliers. Notoriety may create server-verified bounties,
subject to the same loss caps and target protections.

## Current local playable slice

Start with $160 and WEB-01 earning $10/min net. Buy web ($90), database ($150), or
storage ($120), then deploy its service. The first NPC campaign starts after 45
seconds; later campaigns wait four minutes after closure. Practice starts an NPC
exercise immediately. Auth failures produce an alert; inspect logs, respond, and
review the timeline. All operations and money persist locally in SQLite.

Password guessing produces noisy failures, stolen credentials produce a direct
login, and an unpatched web service produces HTTP/process evidence. Shared deploy
credentials and a flat network allow lateral movement; DMZ-to-internal SSH
segmentation blocks it. Revenue stops on isolated or maintained services while
operating costs remain. Patch takes 12 seconds. Restore requires isolation, $20,
and 15 seconds; the starter backup is clean and immutable.

The training mode prevents reconnection while a known simulated foothold remains
and gives an omniscient debrief. These are explicit tutorial aids, not the online
fog-of-war contract. Full offline automation, anti-griefing, recon, attribution,
recovery guarantees, and multiplayer admission are not implemented yet.

## Presentation and delivery gates

Charcoal, warm white, muted teal, restrained amber; physical-looking racks and
animated packets. Compact bottom dock, right sidebar, floating, and collapsed
modes preserve the idle companion identity. Expand into SOC or offensive operations
only when needed. Online status clearly distinguishes local training, connected,
reconnecting, and offline viewing; stale state never pretends to be live.

1. **Now:** preserve the polished local loop, share attack resolution between NPC
   policy and typed player decisions, and introduce ownership/view boundaries.
2. **Next:** local two-principal harness with NPC recon training, stale intel,
   scoped credentials, selective host actions, and evidence-only online debriefs.
3. **Private online alpha:** server authentication, durable progression, two-client
   duels, automated away defenses, loss caps, protection, and guaranteed recovery.
4. **Broader MMO:** scalable scheduling, richer objective library, organizations,
   marketplace, information economy, and bounties after abuse-resistant playtests.

The success moment remains small: an attacker reaches WEB-01, tries DB-01, and the
player sees their segmentation block the path. In the MMO, another person should
experience that same failed operation from the other side.
