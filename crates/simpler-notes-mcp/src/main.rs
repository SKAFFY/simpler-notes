mod transport;
mod dispatcher;
mod types;
mod tools;

use std::env;
use std::sync::Arc;
use simpler_notes_core::vault::{Vault, VaultConfig};
use crate::transport::McpTransport;
use crate::types::JsonRpcRequest;
use crate::dispatcher::Dispatcher;

fn main() {
    let vault_path = env::var("VAULT_PATH")
        .expect("VAULT_PATH environment variable is required");

    let config = VaultConfig {
        path: vault_path.into(),
        ..Default::default()
    };

    let vault = match Vault::open(config) {
        Ok(v) => Arc::new(v),
        Err(e) => {
            eprintln!("Failed to open vault: {}", e);
            std::process::exit(1);
        }
    };

    let mut dispatcher = Dispatcher::new();
    tools::register_all(&mut dispatcher, vault);

    loop {
        let body = match McpTransport::read_message() {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        };

        let request: JsonRpcRequest = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Parse error: {}", e);
                continue;
            }
        };

        let response = dispatcher.handle_jsonrpc(request);

        if let Some(resp) = response {
            let response_body = serde_json::to_string(&resp).unwrap_or_else(|e| {
                eprintln!("Serialization error: {}", e);
                String::from("{}")
            });
            if let Err(e) = McpTransport::write_message(&response_body) {
                eprintln!("Write error: {}", e);
                break;
            }
        }
    }
}
