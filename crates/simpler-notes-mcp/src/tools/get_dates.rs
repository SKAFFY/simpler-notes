use std::sync::Arc;
use serde_json::{json, Value};
use chrono::NaiveDate;
use simpler_notes_core::vault::Vault;
use crate::dispatcher::Tool;

pub struct GetDatesTool {
    vault: Arc<Vault>,
}

impl GetDatesTool {
    pub fn new(vault: Arc<Vault>) -> Self {
        GetDatesTool { vault }
    }
}

impl Tool for GetDatesTool {
    fn call(&self, params: Option<Value>) -> Result<Value, (i32, String)> {
        let dates = if let Some(p) = params {
            let from = p.get("from").and_then(|v| v.as_str())
                .and_then(|s| NaiveDate::parse_from_str(s, "%d.%m.%Y").ok());
            let to = p.get("to").and_then(|v| v.as_str())
                .and_then(|s| NaiveDate::parse_from_str(s, "%d.%m.%Y").ok());
            match (from, to) {
                (Some(f), Some(t)) => self.vault.get_dates_in_range(f, t),
                _ => self.vault.index.dates.all_dates(),
            }
        } else {
            self.vault.index.dates.all_dates()
        };

        let items: Vec<Value> = dates.into_iter().map(|(date, entries)| {
            let notes: Vec<String> = entries.iter().map(|e| e.path.to_string_lossy().to_string()).collect();
            json!({"date": date.to_string(), "notes": notes})
        }).collect();
        Ok(json!(items))
    }
}
