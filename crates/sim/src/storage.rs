use crate::World;
use rusqlite::{params, Connection};
use std::path::Path;

pub struct Store {
    conn: Connection,
}
impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        conn.busy_timeout(std::time::Duration::from_secs(3))
            .map_err(|e| e.to_string())?;
        let version: u32 = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(|e| e.to_string())?;
        if version > 1 {
            return Err("This save was created by a newer version of sudo survive.".into());
        }
        conn.execute_batch("PRAGMA journal_mode = WAL;")
            .map_err(|e| e.to_string())?;
        conn.execute_batch(include_str!("../migrations/001_initial.sql"))
            .map_err(|e| e.to_string())?;
        Ok(Self { conn })
    }
    pub fn load(&self) -> Result<World, String> {
        use rusqlite::OptionalExtension;
        let row: Option<(u32, String)> = self
            .conn
            .query_row(
                "SELECT format_version, state_json FROM world_snapshot WHERE id=1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        match row {
            None => Ok(World::default()),
            Some((1, json)) => {
                let w: World = serde_json::from_str(&json)
                    .map_err(|e| format!("Save could not be read: {e}"))?;
                if w.format_version != 1 {
                    return Err("Unsupported world format".into());
                }
                Ok(w)
            }
            Some(_) => Err("Unsupported save format; existing save was not modified.".into()),
        }
    }
    pub fn save(&mut self, world: &World) -> Result<(), String> {
        let json = serde_json::to_string(world).map_err(|e| e.to_string())?;
        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        let result = (|| -> rusqlite::Result<()> {
            tx.execute("INSERT INTO world_snapshot(id, format_version, tick, state_json) VALUES (1,1,?1,?2) ON CONFLICT(id) DO UPDATE SET tick=excluded.tick, state_json=excluded.state_json, saved_at=CURRENT_TIMESTAMP", params![world.tick, json])?;
            tx.execute("DELETE FROM security_events", [])?;
            for e in &world.events {
                tx.execute(
                    "INSERT INTO security_events VALUES (?1,?2,?3,?4,?5,?6,?7)",
                    params![
                        e.id,
                        e.tick,
                        e.event.label(),
                        e.host,
                        e.source_ip,
                        e.user,
                        serde_json::to_string(e).unwrap()
                    ],
                )?;
            }
            tx.execute("DELETE FROM incident_reports", [])?;
            for r in &world.reports {
                tx.execute(
                    "INSERT INTO incident_reports VALUES (?1,?2,?3)",
                    params![r.id, r.closed_tick, serde_json::to_string(r).unwrap()],
                )?;
            }
            Ok(())
        })();
        result.map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())
    }
}
