use std::io::{self, BufRead, Write};

use simpler_notes_core::vault::Vault;

use crate::handler::handle_request;
use crate::protocol::{JsonRpcRequest, JsonRpcResponse};

pub fn run(vault: &Vault) {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(line) => {
                let trimmed = line.trim().to_string();
                if trimmed.is_empty() {
                    continue;
                }

                let response = match serde_json::from_str::<JsonRpcRequest>(&trimmed) {
                    Ok(request) => {
                        let is_shutdown = request.method == "shutdown";
                        let resp = handle_request(vault, &request.method, &request.params, request.id);
                        let serialized = serde_json::to_string(&resp).unwrap_or_default();
                        if is_shutdown {
                            println!("{}", serialized);
                            io::stdout().flush().ok();
                            std::process::exit(0);
                        }
                        serialized
                    }
                    Err(e) => {
                        serde_json::to_string(&JsonRpcResponse::parse_error(None, e.to_string()))
                            .unwrap_or_default()
                    }
                };

                println!("{}", response);
                io::stdout().flush().ok();
            }
            Err(e) => {
                eprintln!("Error reading stdin: {}", e);
                break;
            }
        }
    }
}
