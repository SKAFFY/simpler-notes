use std::io::{self, Write, Read};

#[allow(dead_code)]
/// Parse a single MCP message from a byte buffer.
/// Expects `Content-Length: N\r\n\r\n` header followed by exactly N bytes.
/// Returns the body as a String, or an error.
pub fn parse_message(input: &[u8]) -> Result<String, String> {
    if input.is_empty() {
        return Err("Connection closed".to_string());
    }

    let text = std::str::from_utf8(input).map_err(|e| format!("Invalid UTF-8: {}", e))?;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Content-Length:") {
            let len_str = trimmed.trim_start_matches("Content-Length:").trim();
            let len: usize = len_str.parse().map_err(|e| format!("Invalid Content-Length: {}", e))?;

            let header_end = text.find("\r\n\r\n").or_else(|| text.find("\n\n"));
            let body_start = match header_end {
                Some(pos) => pos + if text[pos..].starts_with("\r\n\r\n") { 4 } else { 2 },
                None => return Err("Missing blank line after header".to_string()),
            };

            let body_bytes = &input[body_start..];
            if body_bytes.len() < len {
                return Err(format!(
                    "Content-Length {} exceeds body size {}",
                    len,
                    body_bytes.len()
                ));
            }

            let body = &body_bytes[..len];
            return String::from_utf8(body.to_vec()).map_err(|e| format!("Invalid UTF-8 body: {}", e));
        }
    }

    Err("No Content-Length header found".to_string())
}

#[allow(dead_code)]
/// Format a body into an MCP transport message bytes.
pub fn format_message(body: &str) -> Vec<u8> {
    format!("Content-Length: {}\r\n\r\n{}", body.len(), body).into_bytes()
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_message() {
        let msg = format_message("hello");
        let result = parse_message(&msg).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_parse_empty_body() {
        let msg = format_message("");
        let result = parse_message(&msg).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_parse_unicode_body() {
        let msg = format_message("привет мир 🎉");
        let result = parse_message(&msg).unwrap();
        assert_eq!(result, "привет мир 🎉");
    }

    #[test]
    fn test_parse_empty_input() {
        let err = parse_message(b"").unwrap_err();
        assert!(err.contains("Connection closed"));
    }

    #[test]
    fn test_parse_no_header() {
        let err = parse_message(b"hello world").unwrap_err();
        assert!(err.contains("No Content-Length"));
    }

    #[test]
    fn test_parse_invalid_content_length() {
        let input = b"Content-Length: abc\r\n\r\n{}";
        let err = parse_message(input).unwrap_err();
        assert!(err.contains("Invalid Content-Length"));
    }

    #[test]
    fn test_parse_content_length_too_short() {
        let input = b"Content-Length: 100\r\n\r\n{}";
        let err = parse_message(input).unwrap_err();
        assert!(err.contains("exceeds body size"));
    }

    #[test]
    fn test_parse_invalid_utf8() {
        let input = b"Content-Length: 1\r\n\r\n\xff";
        let err = parse_message(input).unwrap_err();
        assert!(err.contains("Invalid UTF-8"));
    }

    #[test]
    fn test_format_message_empty() {
        let result = format_message("");
        assert_eq!(
            String::from_utf8(result).unwrap(),
            "Content-Length: 0\r\n\r\n"
        );
    }

    #[test]
    fn test_format_message_with_body() {
        let result = format_message("test");
        assert_eq!(
            String::from_utf8(result).unwrap(),
            "Content-Length: 4\r\n\r\ntest"
        );
    }

    #[test]
    fn test_format_message_unicode() {
        let body = "привет";
        let result = format_message(body);
        let expected_len = body.len();
        assert_eq!(
            String::from_utf8(result).unwrap(),
            format!("Content-Length: {}\r\n\r\n{}", expected_len, body)
        );
    }

    #[test]
    fn test_parse_missing_blank_line() {
        let input = b"Content-Length: 4\r\nbody";
        let err = parse_message(input).unwrap_err();
        assert!(err.contains("Missing blank line"));
    }
}
