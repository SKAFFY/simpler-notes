use std::io::{self, Write, Read};

pub struct McpTransport;

impl McpTransport {
    pub fn read_message() -> Result<String, String> {
        let mut stdin = io::stdin();

        loop {
            let mut line = String::new();
            let n = stdin.read_line(&mut line).map_err(|e| e.to_string())?;
            if n == 0 {
                return Err("Connection closed".to_string());
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with("Content-Length:") {
                let len_str = trimmed.trim_start_matches("Content-Length:").trim();
                let len: usize = len_str.parse().map_err(|e| format!("Invalid Content-Length: {}", e))?;

                // Skip the blank line after header
                let mut blank = String::new();
                stdin.read_line(&mut blank).map_err(|e| e.to_string())?;

                let mut body = vec![0u8; len];
                stdin.read_exact(&mut body).map_err(|e| e.to_string())?;
                return String::from_utf8(body).map_err(|e| e.to_string());
            }
        }
    }

    pub fn write_message(body: &str) -> Result<(), String> {
        let stdout = io::stdout();
        let mut writer = stdout.lock();
        write!(writer, "Content-Length: {}\r\n\r\n{}", body.len(), body)
            .map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())?;
        Ok(())
    }
}
