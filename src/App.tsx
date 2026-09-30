import { trackView } from "./lib/analytics";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  Activity,
  ArrowDownToLine,
  ArrowRight,
  Check,
  ChevronDown,
  ChevronRight,
  CircleHelp,
  Database,
  Globe2,
  HardDrive,
  Layers3,
  LockKeyhole,
  Maximize2,
  Minus,
  Network,
  PanelBottom,
  PanelRight,
  Pause,
  Play,
  Plus,
  Radio,
  RotateCcw,
  Search,
  Server as ServerIcon,
  Shield,
  ShieldAlert,
  ShieldCheck,
  Terminal,
  X,
  Pin,
  Unplug,
  Wrench,
  FileClock,
  Wallet,
} from "lucide-react";
import * as api from "./api";
import { arrange, minimize, setPinned, type Mode } from "./window";
import type { Action, Report, SecurityEvent, Server, World } from "./types";

const money = (micro: number) =>
  (micro / 1_000_000).toLocaleString("en-US", {
    style: "currency",
    currency: "USD",
    minimumFractionDigits: 2,
  });
const time = (tick: number) =>
  `${Math.floor(tick / 3600)
    .toString()
    .padStart(2, "0")}:${Math.floor((tick / 60) % 60)
    .toString()
    .padStart(2, "0")}:${(tick % 60).toString().padStart(2, "0")}`;
const errorText = (e: unknown) => (e instanceof Error ? e.message : String(e));
const serviceIcon = (kind: string) =>
  kind === "database" ? (
    <Database size={15} />
  ) : kind === "storage" ? (
    <HardDrive size={15} />
  ) : (
    <Globe2 size={15} />
  );
type Tab = "events" | "response" | "reports";

function Rack({
  server,
  world,
  selected,
  onClick,
}: {
  server: Server;
  world: World;
  selected: boolean;
  onClick: () => void;
}) {
  const alerted = world.alerts.some((a) => a.host === server.id && !a.resolved);
  const maintenance = server.maintenance_until > world.tick;
  const online = server.deployed && !server.isolated && !maintenance;
  const spec = world.catalog.services.find((s) => s.kind === server.kind)!;
  return (
    <button
      className={`rack-item ${selected ? "selected" : ""} ${server.isolated ? "isolated" : ""} ${alerted ? "suspicious" : ""} ${online ? "online" : "offline"}`}
      onClick={onClick}
      aria-label={`Inspect ${server.id}`}
    >
      <div className="rack-name">
        <span className="status-dot" />
        {server.id}
        <span className="rack-zone">{server.zone}</span>
      </div>
      <div className="rack-chassis">
        <div className="rack-cap">
          <span>S / S</span>
          <span>● ●</span>
        </div>
        {[0, 1, 2].map((i) => (
          <div className={`rack-unit unit-${i}`} key={i}>
            <span className="screw" />
            <div className="vents">
              {[0, 1, 2, 3, 4, 5, 6, 7].map((j) => (
                <i key={j} />
              ))}
            </div>
            <div className="rack-lights">
              <i />
              <i />
              <i />
            </div>
            <span className="screw" />
          </div>
        ))}
        <div className="rack-screen">
          <span>
            {maintenance
              ? "MAINTENANCE"
              : server.isolated
                ? "ISOLATED"
                : !server.deployed
                  ? "READY TO DEPLOY"
                  : alerted
                    ? "REVIEW ACTIVITY"
                    : "SERVICE ONLINE"}
          </span>
          <div className="sparkline">
            {[
              14, 22, 11, 30, 20, 26, 12, 18, 34, 24, 17, 28, 20, 32, 14, 24,
            ].map((h, i) => (
              <i
                key={i}
                style={{
                  height: online ? `${h / 2}px` : "2px",
                  animationDelay: `${i * 0.12}s`,
                }}
              />
            ))}
          </div>
        </div>
        <div className="rack-feet">
          <i />
          <i />
        </div>
      </div>
      <div className="rack-description">
        {serviceIcon(server.kind)}
        <span>{spec.name}</span>
      </div>
      <div className="rack-income">
        {online
          ? `+$${spec.revenue_per_min - spec.cost_per_min}`
          : `−$${spec.cost_per_min}`}
        <span>/ min</span>
        <small>
          {maintenance
            ? `${server.maintenance_until - world.tick}s left`
            : online
              ? `${server.traffic} req/s`
              : server.isolated
                ? "offline"
                : "undeployed"}
        </small>
      </div>
      <div className="cable">
        <span />
      </div>
    </button>
  );
}

function EventTable({ events }: { events: SecurityEvent[] }) {
  return (
    <div
      className="event-table"
      role="region"
      aria-label="Security event logs"
      tabIndex={0}
    >
      <table>
        <thead>
          <tr>
            <th>TIME</th>
            <th>EVENT / EVIDENCE</th>
            <th>ASSET</th>
            <th>SOURCE</th>
          </tr>
        </thead>
        <tbody>
          {[...events].reverse().map((e) => (
            <tr
              key={e.id}
              className={
                e.event.includes("FAILED") || e.event === "DATA_TRANSFER"
                  ? "event-warning"
                  : e.user === "operator" || e.event === "FIREWALL_BLOCK"
                    ? "event-action"
                    : ""
              }
            >
              <td>{time(e.tick)}</td>
              <td>
                <span className="event-kind">{e.event}</span>
                <span className="event-message">
                  {e.message}
                  {e.user && e.user !== "visitor" ? ` · user=${e.user}` : ""}
                  {e.destination ? ` · dst=${e.destination}` : ""}
                </span>
              </td>
              <td>{e.host}</td>
              <td>{e.source_ip}</td>
            </tr>
          ))}
        </tbody>
      </table>
      {!events.length && (
        <div className="empty">
          <Search size={24} />
          <p>No matching events.</p>
          <small>Try clearing the filter or waiting for new telemetry.</small>
        </div>
      )}
    </div>
  );
}

function IncidentReview({ report }: { report: Report }) {
  return (
    <div className="report-content">
      <div className="report-heading">
        <div>
          <span className="eyebrow">
            INCIDENT {String(report.id).padStart(3, "0")} ·{" "}
            {time(report.closed_tick)}
          </span>
          <h2>
            {report.outcome === "Contained"
              ? "A quieter network. A clearer picture."
              : "The evidence tells a story."}
          </h2>
        </div>
        <span
          className={`pill ${report.outcome === "Contained" ? "good" : "warn"}`}
        >
          {report.outcome === "Contained" ? (
            <ShieldCheck size={13} />
          ) : (
            <ShieldAlert size={13} />
          )}{" "}
          {report.outcome}
        </span>
      </div>
      <div className="report-metrics">
        <div>
          <label>TIME TO DETECT</label>
          <strong>
            {report.time_to_detect === null
              ? "Undetected"
              : `${report.time_to_detect}s`}
          </strong>
        </div>
        <div>
          <label>TIME TO CONTAIN</label>
          <strong>
            {report.time_to_contain === null
              ? "Not contained"
              : `${report.time_to_contain}s`}
          </strong>
        </div>
        <div>
          <label>AFFECTED HOSTS</label>
          <strong>{report.affected_hosts.length}</strong>
        </div>
        <div>
          <label>DATA LOST</label>
          <strong>
            {report.data_lost_mb} <small>MB</small>
          </strong>
        </div>
        <div>
          <label>DOWNTIME*</label>
          <strong>
            {report.downtime_seconds} <small>host-sec</small>
          </strong>
        </div>
        <div>
          <label>LOST REVENUE*</label>
          <strong>{money(report.lost_revenue)}</strong>
        </div>
      </div>
      <div className="report-analysis">
        <div>
          <span className="eyebrow">ROOT CAUSE</span>
          <p>{report.root_cause}</p>
          <small>
            {report.attacker} ·{" "}
            {report.affected_hosts.join(", ") || "No hosts compromised"}
          </small>
        </div>
        <div>
          <span className="eyebrow">LESSONS LEARNED</span>
          <ul>
            {report.recommendations.map((r) => (
              <li key={r}>{r}</li>
            ))}
          </ul>
        </div>
      </div>
      <p className="footnote">
        * At campaign close. Recovery downtime continues in the asset record.
        Containment does not remove persistence.
      </p>
      <EventTable events={report.timeline} />
    </div>
  );
}

export default function App() {
  const [world, setWorld] = useState<World | null>(null);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [busy, setBusy] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const [tab, setTab] = useState<Tab>("events");
  useEffect(() => {
    trackView(expanded ? tab : "rack");
  }, [expanded, tab]);
  const [selected, setSelected] = useState("WEB-01");
  const [shop, setShop] = useState(false);
  const [help, setHelp] = useState(false);
  const [mode, setMode] = useState<Mode>(() => {
    const saved = localStorage.getItem("sudo-window-mode");
    return saved === "right" || saved === "floating" || saved === "collapsed"
      ? saved
      : "bottom";
  });
  const [pinned, setPin] = useState(true);
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState("");
  const [filtered, setFiltered] = useState<SecurityEvent[]>([]);
  const [queryError, setQueryError] = useState("");
  const [source, setSource] = useState("");
  const [reportId, setReportId] = useState<number | null>(null);
  const [practice, setPractice] = useState("brute_force");
  useEffect(() => {
    if (!shop && !help) return;
    const previous = document.activeElement as HTMLElement | null;
    const dialog = document.querySelector<HTMLElement>('[role="dialog"]');
    const focusable = () =>
      Array.from(
        dialog?.querySelectorAll<HTMLElement>(
          'button:not(:disabled), select, input, [tabindex="0"]',
        ) || [],
      );
    focusable()[0]?.focus();
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setShop(false);
        setHelp(false);
      }
      if (event.key !== "Tab") return;
      const items = focusable();
      const first = items[0];
      const last = items.at(-1);
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last?.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first?.focus();
      }
    };
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("keydown", onKey);
      previous?.focus();
    };
  }, [shop, help]);
  const mutating = useRef(false);
  useEffect(() => {
    let active = true;
    const poll = async () => {
      if (mutating.current) return;
      try {
        const state = await api.getWorld();
        if (active && !mutating.current) {
          setWorld(state);
          setError("");
        }
      } catch (e) {
        if (active) setError(errorText(e));
      }
    };
    void poll();
    const id = setInterval(poll, 1000);
    return () => {
      active = false;
      clearInterval(id);
    };
  }, []);
  useEffect(() => {
    if (api.native && !localStorage.getItem("sudo-window-initialized")) {
      void arrange("bottom", false)
        .then(() => localStorage.setItem("sudo-window-initialized", "yes"))
        .catch((e) => setError(errorText(e)));
    }
  }, []);
  useEffect(() => {
    if (!notice) return;
    const id = setTimeout(() => setNotice(""), 6000);
    return () => clearTimeout(id);
  }, [notice]);
  useEffect(() => {
    let active = true;
    if (filter && world)
      void api
        .search(filter)
        .then((events) => {
          if (active) {
            setFiltered(events);
            setQueryError("");
          }
        })
        .catch((e) => {
          if (active) setQueryError(errorText(e));
        });
    return () => {
      active = false;
    };
  }, [filter, world?.tick, world?.events.length]);
  const openConsole = useCallback(
    (nextTab: Tab = "events") => {
      setExpanded(true);
      setTab(nextTab);
      const nextMode = mode === "collapsed" ? "bottom" : mode;
      if (nextMode !== mode) {
        setMode(nextMode);
        localStorage.setItem("sudo-window-mode", nextMode);
      }
      void arrange(nextMode, true).catch((e) => setError(errorText(e)));
    },
    [mode],
  );
  const changeMode = (value: Mode) => {
    setMode(value);
    localStorage.setItem("sudo-window-mode", value);
    if (value === "collapsed") setExpanded(false);
    void arrange(value, value === "collapsed" ? false : expanded).catch((e) =>
      setError(errorText(e)),
    );
  };
  const perform = async (action: Action, message: string) => {
    if (mutating.current) return;
    mutating.current = true;
    setBusy(true);
    try {
      const next = await api.act(action);
      setWorld(next);
      setError("");
      setNotice(message);
      if (action.type === "buy") setSelected(next.servers.at(-1)!.id);
      if (world && next.reports.length > world.reports.length) {
        setReportId(next.reports.at(-1)!.id);
        setTab("reports");
      }
    } catch (e) {
      setNotice(errorText(e));
    } finally {
      mutating.current = false;
      setBusy(false);
    }
  };
  if (!world)
    return (
      <main className="connection-screen">
        <div className="brand">
          <Terminal size={24} />
          sudo<span>survive</span>
        </div>
        <Activity className="loading-icon" />
        <h2>
          {error ? "Waiting for the simulation" : "Booting your homelab…"}
        </h2>
        <p>{error || "Connecting to the local Rust engine."}</p>
        {!api.native && <code>Start with: npm run dev:browser</code>}
      </main>
    );
  const alerts = world.alerts.filter((a) => !a.resolved);
  const latest = alerts.at(-1);
  const host = world.servers.find((s) => s.id === selected) || world.servers[0];
  const spec = world.catalog.services.find((s) => s.kind === host.kind)!;
  const maintenance = host.maintenance_until > world.tick;
  const online = world.servers.filter(
    (s) => s.deployed && !s.isolated && s.maintenance_until <= world.tick,
  ).length;
  const report =
    world.reports.find((r) => r.id === reportId) || world.reports.at(-1);
  const investigate = () => {
    if (latest) {
      setSelected(latest.host);
      setSource(latest.source_ip);
      const q = `host = ${latest.host}`;
      setQuery(q);
      setFilter(q);
    }
    openConsole("events");
  };

  return (
    <main
      className={`app mode-${mode} ${expanded ? "expanded" : ""} ${world.paused ? "paused" : ""}`}
    >
      <header className="titlebar" data-tauri-drag-region>
        <div className="brand" data-tauri-drag-region>
          <span className="brand-icon">
            <Terminal size={18} />
          </span>
          sudo<span>survive</span>
          <span className="alpha">PREVIEW 0.1</span>
        </div>
        <div className="economy">
          <Wallet size={14} />
          <strong data-testid="balance">{money(world.cash)}</strong>
          <span className={world.net_per_min >= 0 ? "positive" : "negative"}>
            {world.net_per_min >= 0 ? "+" : ""}${world.net_per_min}
            <small>/min</small>
          </span>
        </div>
        <div className="window-actions">
          <button
            aria-label={world.paused ? "Resume simulation" : "Pause simulation"}
            title={world.paused ? "Resume" : "Pause"}
            onClick={() =>
              void perform(
                { type: "pause" },
                world.paused
                  ? "Simulation resumed."
                  : "Simulation paused. Take your time.",
              )
            }
            disabled={busy}
          >
            {world.paused ? <Play /> : <Pause />}
          </button>
          <button
            aria-label="Toggle always on top"
            aria-pressed={pinned}
            title="Always on top"
            className={pinned ? "active" : ""}
            onClick={() => {
              void setPinned(!pinned)
                .then(() => setPin(!pinned))
                .catch((e) => setError(errorText(e)));
            }}
          >
            <Pin />
          </button>
          <button
            aria-label="Minimize window"
            title="Minimize"
            onClick={() => {
              if (api.native)
                void minimize().catch((e) => setError(errorText(e)));
              else changeMode("collapsed");
            }}
          >
            <Minus />
          </button>
        </div>
      </header>
      {mode === "collapsed" ? (
        <div className="collapsed-content">
          <span className={`status-dot ${latest ? "amber" : ""}`} />
          <span>
            {latest
              ? `${latest.title} · ${latest.host}`
              : `${online} services online · your little corner of the internet`}
          </span>
          <button
            className={latest ? "button amber-button" : "button"}
            onClick={() => {
              if (latest) investigate();
              else changeMode("bottom");
            }}
          >
            {latest ? "Investigate" : "Expand"}
            <Maximize2 size={13} />
          </button>
        </div>
      ) : (
        <>
          <nav className="workspace-bar">
            <div className="workspace-name">
              <Network size={14} />
              <span>homelab</span>
              <span className="slash">/</span>
              <span className="muted">production</span>
              <span className="pill good small">
                <span className="status-dot" />
                {world.paused ? "PAUSED" : "LIVE"}
              </span>
            </div>
            <div className="workspace-tools">
              <button className="text-button" onClick={() => setHelp(true)}>
                <CircleHelp size={13} />
                <span>Field guide</span>
              </button>
              <span className="divider" />
              <button
                aria-label="Bottom dock"
                className={mode === "bottom" ? "active" : ""}
                title="Bottom dock"
                onClick={() => changeMode("bottom")}
              >
                <PanelBottom size={15} />
              </button>
              <button
                aria-label="Right dock"
                className={mode === "right" ? "active" : ""}
                title="Right dock"
                onClick={() => changeMode("right")}
              >
                <PanelRight size={15} />
              </button>
              <button
                aria-label="Floating window"
                className={mode === "floating" ? "active" : ""}
                title="Floating window"
                onClick={() => changeMode("floating")}
              >
                <Maximize2 size={14} />
              </button>
              <button
                aria-label="Collapse game"
                title="Collapse"
                onClick={() => changeMode("collapsed")}
              >
                <ChevronDown size={16} />
              </button>
            </div>
          </nav>
          <section className="infrastructure">
            <div className="scene-heading">
              <div>
                <span className="eyebrow">
                  YOUR LITTLE CORNER OF THE INTERNET
                </span>
                <h1>Small rack. Big responsibility.</h1>
              </div>
              <button className="button primary" onClick={() => setShop(true)}>
                <Plus size={14} />
                Add server<span className="key">+</span>
              </button>
            </div>
            <div className="network-scene">
              <div className="perimeter">
                <div className="internet-node">
                  <Globe2 size={22} />
                  <span>INTERNET</span>
                </div>
                <div className="uplink">
                  <i />
                  <i />
                </div>
                <button
                  className="firewall-node"
                  onClick={() => openConsole("response")}
                  aria-label="Inspect firewall"
                >
                  <ShieldCheck size={24} />
                  <strong>edge-fw</strong>
                  <span>
                    {world.network.segmented ? "SEGMENTED" : "BASIC FIREWALL"}
                  </span>
                  <div className="firewall-ports">
                    <i />
                    <i />
                    <i />
                    <i />
                    <i />
                    <i />
                  </div>
                </button>
                <div className="perimeter-label">
                  <span className="status-dot" />
                  {world.network.rules.length} blocked sources
                </div>
              </div>
              <div className="rack-floor">
                <div className="zone-label">
                  <span>
                    {world.network.segmented
                      ? "DMZ / INTERNAL · ADMIN PATH DENIED"
                      : "FLAT NETWORK"}
                  </span>
                  <span>{world.servers.length} / 8 slots</span>
                </div>
                <div className="rack-row">
                  {world.servers.map((s) => (
                    <Rack
                      key={s.id}
                      server={s}
                      world={world}
                      selected={selected === s.id}
                      onClick={() => {
                        setSelected(s.id);
                        openConsole("response");
                      }}
                    />
                  ))}
                  <button
                    className="empty-rack"
                    onClick={() => setShop(true)}
                    aria-label="Provision a new server"
                  >
                    <Plus size={22} />
                    <span>ROOM TO GROW</span>
                    <small>Provision a server</small>
                  </button>
                </div>
                <div className="network-bus">
                  <span className="packet p1" />
                  <span className="packet p2" />
                  <span className="packet p3" />
                </div>
              </div>
              <aside className={`scene-note ${latest ? "alert-note" : ""}`}>
                {latest ? (
                  <>
                    <div className="note-icon">
                      <ShieldAlert size={19} />
                      <span>SUSPICIOUS ACTIVITY</span>
                    </div>
                    <h3>{latest.host}</h3>
                    <p>{latest.title}. Follow the evidence before you act.</p>
                    <button
                      className="button amber-button"
                      onClick={investigate}
                    >
                      Investigate
                      <ArrowRight size={14} />
                    </button>
                  </>
                ) : (
                  <>
                    <div className="note-icon">
                      <Radio size={17} />
                      <span>ON WATCH</span>
                    </div>
                    <h3>
                      {world.reports.length
                        ? "Room to breathe."
                        : "Let it hum."}
                    </h3>
                    <p>
                      {world.reports.length
                        ? "Review the incident, recover your services, and build a more resilient network."
                        : "Your services earn while you work. We’ll flag unusual activity here."}
                    </p>
                    <span className="quiet-stats">
                      <Activity size={12} />
                      {world.servers.reduce(
                        (sum, s) => sum + s.traffic,
                        0,
                      )}{" "}
                      requests / sec
                    </span>
                  </>
                )}
              </aside>
            </div>
          </section>
          <div className="console-toggle">
            <button
              className={expanded ? "active" : ""}
              onClick={() => {
                if (expanded) {
                  setExpanded(false);
                  void arrange(mode, false).catch((e) =>
                    setError(errorText(e)),
                  );
                } else openConsole();
              }}
            >
              <Terminal size={14} />
              <span>Security console</span>
              <span className={`count ${alerts.length ? "warn" : ""}`}>
                {alerts.length}
              </span>
              <ChevronDown size={14} className={expanded ? "rotate" : ""} />
            </button>
            <span className="console-summary">
              {latest
                ? `${latest.title} on ${latest.host}`
                : "Observing authentication, endpoints & network flows"}
            </span>
            <span className="shortcut">LOCAL SIMULATION</span>
          </div>
          {expanded && (
            <section className="security-console">
              <nav className="console-tabs">
                <div>
                  <button
                    className={tab === "events" ? "selected" : ""}
                    onClick={() => setTab("events")}
                  >
                    <Activity size={14} />
                    Event stream<span>{world.events.length}</span>
                  </button>
                  <button
                    className={tab === "response" ? "selected" : ""}
                    onClick={() => setTab("response")}
                  >
                    <Shield size={14} />
                    Response
                  </button>
                  <button
                    className={tab === "reports" ? "selected" : ""}
                    onClick={() => setTab("reports")}
                  >
                    <FileClock size={14} />
                    Incident review
                    {world.reports.length > 0 && (
                      <span>{world.reports.length}</span>
                    )}
                  </button>
                </div>
                <span className="console-id">SOC / 01</span>
              </nav>
              {tab === "events" && (
                <div className="events-panel">
                  <aside className="alert-list">
                    <div className="panel-label">
                      DETECTIONS<span>{alerts.length} open</span>
                    </div>
                    {!alerts.length && (
                      <div className="empty-alert">
                        <ShieldCheck size={23} />
                        <strong>No open alerts</strong>
                        <p>
                          A quiet console is a good thing. Keep an eye on the
                          evidence.
                        </p>
                      </div>
                    )}
                    {[...alerts].reverse().map((a) => (
                      <button
                        key={a.id}
                        className={`alert-card ${selected === a.host ? "selected" : ""}`}
                        onClick={() => {
                          setSelected(a.host);
                          setSource(a.source_ip);
                          const q = `host = ${a.host}`;
                          setQuery(q);
                          setFilter(q);
                        }}
                      >
                        <span>
                          <span className="status-dot amber" />
                          {a.host}
                          <small>{time(a.tick)}</small>
                        </span>
                        <strong>{a.title}</strong>
                        <p>{a.detail}</p>
                        <code>{a.source_ip}</code>
                      </button>
                    ))}
                    <div className="detector-info">
                      <span className="eyebrow">ACTIVE DETECTION</span>
                      <strong>Authentication burst</strong>
                      <p>
                        ≥ {world.detection.threshold} failures /{" "}
                        {world.detection.window_seconds}s<br />
                        Grouped by source + host
                      </p>
                    </div>
                  </aside>
                  <div className="logs-panel">
                    <form
                      className="query-bar"
                      onSubmit={(e) => {
                        e.preventDefault();
                        setFilter(query.trim());
                        setQueryError("");
                      }}
                    >
                      <Search size={15} />
                      <input
                        aria-label="Search security logs"
                        value={query}
                        onChange={(e) => setQuery(e.target.value)}
                        placeholder="event = AUTH_LOGIN_FAILED AND user = deploy"
                      />
                      <button type="submit">
                        Search
                        <ChevronRight size={13} />
                      </button>
                      {filter && (
                        <button
                          type="button"
                          aria-label="Clear log filter"
                          onClick={() => {
                            setQuery("");
                            setFilter("");
                            setQueryError("");
                          }}
                        >
                          <X size={13} />
                        </button>
                      )}
                    </form>
                    {queryError && (
                      <p className="query-error" role="alert">
                        {queryError}
                      </p>
                    )}
                    <div className="log-toolbar">
                      <span>
                        <span className="status-dot" />{" "}
                        {filter ? "FILTERED" : "LIVE"} TELEMETRY · newest first
                      </span>
                      <button
                        className="text-button"
                        onClick={() => setTab("response")}
                      >
                        Respond on {host.id}
                        <ArrowRight size={12} />
                      </button>
                    </div>
                    <EventTable events={filter ? filtered : world.events} />
                  </div>
                </div>
              )}
              {tab === "response" && (
                <div className="response-panel">
                  <div className="response-host">
                    <div className="panel-label">
                      ASSET WORKBENCH
                      <select
                        aria-label="Selected asset"
                        value={host.id}
                        onChange={(e) => setSelected(e.target.value)}
                      >
                        {world.servers.map((s) => (
                          <option key={s.id}>{s.id}</option>
                        ))}
                      </select>
                    </div>
                    <div className="asset-title">
                      {serviceIcon(host.kind)}
                      <h2>{host.id}</h2>
                      <span
                        className={`pill ${host.isolated || maintenance ? "warn" : "good"}`}
                      >
                        {maintenance
                          ? `${host.maintenance_kind} · ${host.maintenance_until - world.tick}s`
                          : host.isolated
                            ? "ISOLATED"
                            : host.deployed
                              ? "ONLINE"
                              : "READY"}
                      </span>
                    </div>
                    <p>
                      {spec.name} · {host.zone.toUpperCase()} · port {spec.port}
                    </p>
                    <div className="asset-facts">
                      <span>
                        Service patch
                        <strong>
                          {host.patched ? "Current" : "Update available"}
                        </strong>
                      </span>
                      <span>
                        Recovery point<strong>Clean · immutable</strong>
                      </span>
                      <span>
                        Cumulative downtime<strong>{host.downtime}s</strong>
                      </span>
                    </div>
                    <div className="action-grid">
                      {!host.deployed && (
                        <button
                          className="button primary"
                          disabled={busy}
                          onClick={() =>
                            void perform(
                              { type: "deploy", host: host.id },
                              `${host.id} deployed. Income is now flowing.`,
                            )
                          }
                        >
                          <Play size={13} />
                          Deploy service
                        </button>
                      )}
                      <button
                        className="button"
                        disabled={busy || maintenance}
                        onClick={() =>
                          void perform(
                            {
                              type: host.isolated ? "resume" : "isolate",
                              host: host.id,
                            },
                            host.isolated
                              ? "Host reconnected. Service resumed."
                              : "Host isolated. All traffic denied; service income paused.",
                          )
                        }
                      >
                        <Unplug size={13} />
                        {host.isolated ? "Resume host" : "Isolate host"}
                      </button>
                      <button
                        className="button"
                        disabled={busy || maintenance || host.patched}
                        onClick={() =>
                          void perform(
                            { type: "patch", host: host.id },
                            "Patching service. 12 seconds of planned downtime.",
                          )
                        }
                      >
                        <Wrench size={13} />
                        {host.patched ? "Patched" : "Patch · 12s"}
                      </button>
                      <button
                        className="button"
                        disabled={
                          busy ||
                          maintenance ||
                          !host.isolated ||
                          world.cash < 20_000_000
                        }
                        title="Requires host isolation"
                        onClick={() =>
                          void perform(
                            { type: "restore", host: host.id },
                            "Restoring clean backup. Keep the host isolated until recovery completes.",
                          )
                        }
                      >
                        <RotateCcw size={13} />
                        Restore · $20
                      </button>
                    </div>
                    <small className="response-note">
                      Isolate → rotate credentials → restore → resume.
                      <br />
                      Patching closes a weakness; it does not evict an attacker.
                    </small>
                  </div>
                  <div className="response-controls">
                    <div className="control-section">
                      <span className="eyebrow">
                        <Shield size={12} />
                        PERIMETER FIREWALL
                      </span>
                      <form
                        className="block-form"
                        onSubmit={(e) => {
                          e.preventDefault();
                          void perform(
                            { type: "block_source", source_ip: source },
                            "Source blocked at the perimeter. Inspect any established sessions.",
                          );
                        }}
                      >
                        <input
                          aria-label="Source IP to block"
                          placeholder="192.0.2.41"
                          value={source}
                          onChange={(e) => setSource(e.target.value)}
                        />
                        <button className="button" disabled={busy || !source}>
                          Block source
                        </button>
                      </form>
                      <small>
                        Prevents new ingress. Existing access can survive.
                      </small>
                      <div className="rule-list">
                        {world.network.rules.map((r) => (
                          <code key={r.source}>
                            <span>DENY</span>
                            {r.source} → any
                          </code>
                        ))}
                      </div>
                      <button
                        className="text-button segment-button"
                        disabled={
                          busy ||
                          world.network.segmented ||
                          world.cash < 40_000_000
                        }
                        onClick={() =>
                          void perform(
                            { type: "segment" },
                            "DMZ → internal administrative traffic is now denied.",
                          )
                        }
                      >
                        <Layers3 size={13} />
                        {world.network.segmented
                          ? "DMZ → internal SSH denied"
                          : "Segment network · $40"}
                        {world.network.segmented && <Check size={12} />}
                      </button>
                    </div>
                    <div className="control-section">
                      <span className="eyebrow">
                        <LockKeyhole size={12} />
                        DEPLOYMENT IDENTITY
                      </span>
                      <p>
                        <code>deploy</code>
                        <span
                          className={`pill ${world.user.enabled ? "good" : "warn"}`}
                        >
                          {world.user.enabled ? "ENABLED" : "DISABLED"}
                        </span>
                        <small>
                          credential v{world.user.credential.version}
                        </small>
                      </p>
                      <div className="action-grid">
                        <button
                          className="button"
                          disabled={busy}
                          onClick={() =>
                            void perform(
                              { type: "rotate_credentials" },
                              "Credential rotated. Old credentials invalidated; existing sessions remain.",
                            )
                          }
                        >
                          <RotateCcw size={13} />
                          Rotate credential
                        </button>
                        <button
                          className="button"
                          disabled={busy}
                          onClick={() =>
                            void perform(
                              {
                                type: world.user.enabled
                                  ? "disable_user"
                                  : "enable_user",
                              },
                              world.user.enabled
                                ? "Account disabled and identity sessions revoked. Check endpoint persistence."
                                : "Deployment account enabled.",
                            )
                          }
                        >
                          <LockKeyhole size={13} />
                          {world.user.enabled
                            ? "Disable account"
                            : "Enable account"}
                        </button>
                      </div>
                      <small>
                        {world.user.credential.weak
                          ? "Shared credential · weak password · all hosts authorized"
                          : "Strong credential · all hosts authorized"}
                        <br />
                        Disabling the account revokes sessions, not startup
                        persistence.
                      </small>
                    </div>
                  </div>
                </div>
              )}
              {tab === "reports" && (
                <div className="reports-panel">
                  {report ? (
                    <>
                      <div className="report-picker">
                        <label htmlFor="report-select">
                          POST-INCIDENT REVIEW
                        </label>
                        <select
                          id="report-select"
                          value={report.id}
                          onChange={(e) => setReportId(Number(e.target.value))}
                        >
                          {[...world.reports].reverse().map((r) => (
                            <option key={r.id} value={r.id}>
                              Incident {r.id} · {r.outcome} ·{" "}
                              {time(r.closed_tick)}
                            </option>
                          ))}
                        </select>
                      </div>
                      <IncidentReview report={report} />
                    </>
                  ) : (
                    <div className="empty">
                      <FileClock size={28} />
                      <h3>Every incident leaves a story.</h3>
                      <p>
                        Contain an incident to reconstruct its timeline and
                        learn what worked.
                      </p>
                      <button className="button" onClick={() => setHelp(true)}>
                        Open field guide
                        <ArrowRight size={13} />
                      </button>
                    </div>
                  )}
                </div>
              )}
            </section>
          )}
        </>
      )}
      {mode !== "collapsed" && (
        <footer>
          <div>
            <span className="status-dot" />
            {online}/{world.servers.length} services online
            <span className="footer-separator">·</span>
            <span>
              {world.network.segmented ? "Segmented network" : "Flat network"}
            </span>
          </div>
          <div>
            <span className="save-dot" />
            SQLite autosave<span className="footer-separator">·</span>
            <span className="sim-clock">{time(world.tick)}</span>
            {world.paused && <span className="warn">PAUSED</span>}
          </div>
        </footer>
      )}
      {(notice || error) && (
        <div className={`toast ${error ? "toast-error" : ""}`} role="status">
          <Activity size={15} />
          <span>{error || notice}</span>
          <button
            aria-label="Dismiss notification"
            onClick={() => {
              setNotice("");
              setError("");
            }}
          >
            <X size={14} />
          </button>
        </div>
      )}
      {shop && (
        <div className="modal-backdrop" onClick={() => setShop(false)}>
          <section
            className="modal shop"
            role="dialog"
            aria-modal="true"
            aria-labelledby="shop-title"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="modal-heading">
              <div>
                <span className="eyebrow">MAKE ROOM FOR WHAT’S NEXT</span>
                <h2 id="shop-title">Grow your homelab.</h2>
              </div>
              <button
                aria-label="Close server shop"
                onClick={() => setShop(false)}
              >
                <X size={19} />
              </button>
            </div>
            <p>
              Every service earns its keep. Every service adds something to
              defend.
            </p>
            <div className="shop-cards">
              {world.catalog.services.map((s) => (
                <article key={s.kind}>
                  <div className="shop-icon">
                    {s.kind === "web" ? (
                      <Globe2 />
                    ) : s.kind === "database" ? (
                      <Database />
                    ) : (
                      <HardDrive />
                    )}
                  </div>
                  <span className="eyebrow">
                    {s.zone.toUpperCase()} / {s.port}
                  </span>
                  <h3>{s.name}</h3>
                  <p>
                    {s.kind === "web"
                      ? "A little home on the public internet."
                      : s.kind === "database"
                        ? "Valuable records. Worth protecting."
                        : "A dependable place for everyone’s files."}
                  </p>
                  <div className="shop-economy">
                    <strong>
                      +${s.revenue_per_min - s.cost_per_min}
                      <small>/min net</small>
                    </strong>
                    <span>${s.cost_per_min}/min operating</span>
                  </div>
                  <button
                    className="button primary"
                    disabled={
                      busy ||
                      world.cash < s.price * 1_000_000 ||
                      world.servers.length >= 8
                    }
                    onClick={async () => {
                      await perform(
                        { type: "buy", kind: s.kind },
                        "Server provisioned. Select it and deploy its service to start earning.",
                      );
                      setShop(false);
                      openConsole("response");
                    }}
                  >
                    <Plus size={14} />
                    Buy server <strong>${s.price}</strong>
                  </button>
                </article>
              ))}
            </div>
            <div className="modal-footer">
              <Wallet size={14} />
              {money(world.cash)} available
              <span>Provision first. Deploy when you’re ready.</span>
            </div>
          </section>
        </div>
      )}
      {help && (
        <div className="modal-backdrop" onClick={() => setHelp(false)}>
          <section
            className="modal guide"
            role="dialog"
            aria-modal="true"
            aria-labelledby="guide-title"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="modal-heading">
              <div>
                <span className="eyebrow">THE OPERATOR’S FIELD GUIDE</span>
                <h2 id="guide-title">Stay curious. Follow the evidence.</h2>
              </div>
              <button
                aria-label="Close field guide"
                onClick={() => setHelp(false)}
              >
                <X size={19} />
              </button>
            </div>
            <div className="guide-steps">
              <p>
                <span>01</span>
                <strong>Build something worth defending.</strong>Your first site
                is already earning. Buy a server, select it, and deploy its
                service.
              </p>
              <p>
                <span>02</span>
                <strong>An alert is a lead, not a verdict.</strong>Inspect
                failures, successful logins, new processes, and unfamiliar
                connections. The first incident starts after 45 simulated
                seconds.
              </p>
              <p>
                <span>03</span>
                <strong>Match the response to the evidence.</strong>Block a
                source before access. After compromise, isolate hosts, rotate
                credentials, restore clean backups, then resume.
              </p>
              <p>
                <span>04</span>
                <strong>Make the next incident smaller.</strong>Patch exposed
                services. Separate internal administration from the DMZ. Read
                the postmortem.
              </p>
            </div>
            <div className="practice-box">
              <div>
                <span className="eyebrow">SANDBOX EXERCISE</span>
                <p>
                  Start a simulated incident now. It affects your current
                  homelab.
                </p>
              </div>
              <select
                aria-label="Practice incident type"
                value={practice}
                onChange={(e) => setPractice(e.target.value)}
              >
                {world.catalog.attacks.map((a) => (
                  <option value={a.kind} key={a.kind}>
                    {a.name}
                  </option>
                ))}
              </select>
              <button
                className="button"
                disabled={busy}
                onClick={async () => {
                  await perform(
                    { type: "practice", kind: practice },
                    "Exercise started. Watch for evidence as the campaign unfolds.",
                  );
                  setHelp(false);
                }}
              >
                <Play size={13} />
                Run exercise
              </button>
            </div>
            <p className="footnote">
              Everything here is fictional and simulated locally. Closing the
              app freezes time. Pause whenever you need.
            </p>
          </section>
        </div>
      )}
    </main>
  );
}
