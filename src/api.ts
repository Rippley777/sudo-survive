import { invoke } from "@tauri-apps/api/core";
import type { Action, SecurityEvent, World } from "./types";
export const native = "__TAURI_INTERNALS__" in window;
async function bridge<T>(path: string, body?: unknown): Promise<T> {
  const res = await fetch(
    `/api/${path}`,
    body === undefined
      ? {}
      : {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(body),
        },
  );
  const data = await res.json();
  if (!res.ok) throw new Error(data.error || "Simulation connection failed");
  return data;
}
export const getWorld = () =>
  native ? invoke<World>("get_world") : bridge<World>("state");
export const act = (action: Action) =>
  native ? invoke<World>("act", { action }) : bridge<World>("action", action);
export const search = (query: string) =>
  native
    ? invoke<SecurityEvent[]>("search_logs", { query })
    : bridge<SecurityEvent[]>("search", query);
