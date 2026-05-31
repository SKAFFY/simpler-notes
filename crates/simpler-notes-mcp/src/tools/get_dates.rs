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
        let dates = if let Some(p) = &params {
            let from_str = p.get("from").and_then(|v| v.as_str());
            let to_str = p.get("to").and_then(|v| v.as_str());

            match (from_str, to_str) {
                (Some(f), Some(t)) => {
                    let from = NaiveDate::parse_from_str(f, "%d.%m.%Y")
                        .map_err(|_| (-32602, format!("Invalid date format for 'from': {}. Expected DD.MM.YYYY", f)))?;
                    let to = NaiveDate::parse_from_str(t, "%d.%m.%Y")
                        .map_err(|_| (-32602, format!("Invalid date format for 'to': {}. Expected DD.MM.YYYY", t)))?;
                    self.vault.get_dates_in_range(from, to)
                }
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
