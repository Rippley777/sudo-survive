PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS world_snapshot (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  format_version INTEGER NOT NULL,
  tick INTEGER NOT NULL,
  state_json TEXT NOT NULL CHECK (json_valid(state_json)),
  saved_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE IF NOT EXISTS security_events (
  id INTEGER PRIMARY KEY,
  tick INTEGER NOT NULL,
  kind TEXT NOT NULL,
  host TEXT NOT NULL,
  source_ip TEXT NOT NULL,
  user_name TEXT NOT NULL,
  payload_json TEXT NOT NULL CHECK (json_valid(payload_json))
);
CREATE INDEX IF NOT EXISTS events_host_tick ON security_events(host, tick);
CREATE INDEX IF NOT EXISTS events_source_tick ON security_events(source_ip, tick);
CREATE TABLE IF NOT EXISTS incident_reports (
  id INTEGER PRIMARY KEY,
  closed_tick INTEGER NOT NULL,
  report_json TEXT NOT NULL CHECK (json_valid(report_json))
);
PRAGMA user_version = 1;
