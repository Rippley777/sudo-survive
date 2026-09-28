use crate::SecurityEvent;

pub fn search(events: &[SecurityEvent], query: &str) -> Result<Vec<SecurityEvent>, String> {
    if query.trim().is_empty() {
        return Ok(events.to_vec());
    }
    let mut clauses = vec![];
    // The intentionally small grammar rejects unsupported syntax explicitly.
    for clause in query.split(" AND ") {
        let (field, value) = clause
            .split_once('=')
            .ok_or("Use field = value AND field = value")?;
        let field = field.trim().to_lowercase();
        if !["event", "source_ip", "host", "user"].contains(&field.as_str()) {
            return Err(format!(
                "Unknown field '{field}'. Use event, source_ip, host, or user."
            ));
        }
        let value = value.trim();
        if value.is_empty() || value.contains('=') || value.contains(" AND ") {
            return Err("Each filter needs one value".into());
        }
        let value = if value.starts_with('"') && value.ends_with('"') && value.len() > 2 {
            &value[1..value.len() - 1]
        } else {
            value
        };
        if value.contains('"') || value.contains(' ') {
            return Err("Invalid value; join filters with uppercase AND".into());
        }
        clauses.push((field, value.to_lowercase()));
    }
    Ok(events
        .iter()
        .filter(|e| {
            clauses.iter().all(|(field, value)| {
                let actual = match field.as_str() {
                    "event" => e.event.label(),
                    "source_ip" => e.source_ip.clone(),
                    "host" => e.host.clone(),
                    "user" => e.user.clone(),
                    _ => unreachable!(),
                };
                actual.to_lowercase() == *value
            })
        })
        .cloned()
        .collect())
}
