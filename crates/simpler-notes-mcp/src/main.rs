mod handler;
mod protocol;
mod server;
mod tool;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let vault_path = if let Some(pos) = args.iter().position(|a| a == "--vault") {
        args.get(pos + 1).cloned().unwrap_or_else(|| {
            eprintln!("Usage: simpler-notes-mcp --vault <path>");
            std::process::exit(1);
        })
    } else {
        eprintln!("Usage: simpler-notes-mcp --vault <path>");
        std::process::exit(1);
    };

    let vault = match simpler_notes_core::vault::Vault::open(std::path::Path::new(&vault_path)) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to open vault: {}", e);
            std::process::exit(1);
        }
    };

    server::run(&vault);
}
