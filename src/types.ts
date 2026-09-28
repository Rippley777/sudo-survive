export type Server = {
  id: string;
  kind: string;
  zone: string;
  deployed: boolean;
  isolated: boolean;
  patched: boolean;
  maintenance_until: number;
  maintenance_kind: string | null;
  backup: { tick: number; immutable: boolean; clean: boolean };
  traffic: number;
  downtime: number;
  lost_revenue: number;
};
export type Service = {
  kind: string;
  name: string;
  prefix: string;
  zone: string;
  price: number;
  revenue_per_min: number;
  cost_per_min: number;
  port: number;
};
export type SecurityEvent = {
  id: number;
  tick: number;
  event: string;
  host: string;
  source_ip: string;
  destination: string;
  user: string;
  message: string;
};
export type Alert = {
  id: number;
  tick: number;
  host: string;
  source_ip: string;
  title: string;
  detail: string;
  severity: string;
  resolved: boolean;
};
export type Report = {
  id: number;
  closed_tick: number;
  outcome: string;
  root_cause: string;
  attacker: string;
  time_to_detect: number | null;
  time_to_contain: number | null;
  affected_hosts: string[];
  data_lost_mb: number;
  downtime_seconds: number;
  lost_revenue: number;
  recommendations: string[];
  timeline: SecurityEvent[];
};
export type World = {
  tick: number;
  cash: number;
  net_per_min: number;
  paused: boolean;
  servers: Server[];
  network: {
    segmented: boolean;
    rules: {
      source: string;
      destination: string;
      service: string;
      action: string;
    }[];
  };
  user: {
    name: string;
    enabled: boolean;
    allowed_hosts: string[];
    credential: { version: number; weak: boolean };
  };
  events: SecurityEvent[];
  alerts: Alert[];
  reports: Report[];
  catalog: { services: Service[]; attacks: { kind: string; name: string }[] };
  detection: { id: string; threshold: number; window_seconds: number };
};
export type Action =
  | { type: "buy" | "practice"; kind: string }
  | {
      type: "deploy" | "isolate" | "resume" | "patch" | "restore";
      host: string;
    }
  | { type: "block_source"; source_ip: string }
  | {
      type:
        | "rotate_credentials"
        | "disable_user"
        | "enable_user"
        | "segment"
        | "pause";
    };
