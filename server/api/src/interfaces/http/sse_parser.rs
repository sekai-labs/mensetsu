use std::time::Duration;
use axum::response::sse::Event;

#[derive(Debug, Default)]
pub struct SseParser {
    buffer: Vec<u8>,
    current_event_type: Option<String>,
    current_data_lines: Vec<String>,
    current_id: Option<String>,
    current_retry: Option<Duration>,
    has_event_fields: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ParsedSseFrame {
    pub id: Option<String>,
    pub event: Option<String>,
    pub data: String,
    pub retry: Option<Duration>,
}

impl ParsedSseFrame {
    pub fn into_axum_event(self) -> Event {
        let mut ev = Event::default().data(self.data);
        if let Some(id) = self.id {
            ev = ev.id(id);
        }
        if let Some(event) = self.event {
            ev = ev.event(event);
        }
        if let Some(retry) = self.retry {
            ev = ev.retry(retry);
        }
        ev
    }
}

impl SseParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn feed(&mut self, chunk: &[u8]) -> Vec<ParsedSseFrame> {
        self.buffer.extend_from_slice(chunk);
        let mut frames = Vec::new();

        while let Some(line) = self.next_line() {
            if line.is_empty() {
                if self.has_event_fields || !self.current_data_lines.is_empty() {
                    let frame = ParsedSseFrame {
                        id: self.current_id.take(),
                        event: self.current_event_type.take(),
                        data: self.current_data_lines.join("\n"),
                        retry: self.current_retry.take(),
                    };
                    self.current_data_lines.clear();
                    self.has_event_fields = false;
                    frames.push(frame);
                }
            } else if line.starts_with(':') {
            } else {
                self.process_field(&line);
            }
        }

        frames
    }

    fn next_line(&mut self) -> Option<String> {
        let buf = &self.buffer;
        let len = buf.len();
        let mut i = 0;

        while i < len {
            if buf[i] == b'\n' {
                let line_bytes = &buf[..i];
                let line_str = String::from_utf8_lossy(line_bytes).to_string();
                self.buffer.drain(..=i);
                return Some(line_str);
            } else if buf[i] == b'\r' {
                if i + 1 < len && buf[i + 1] == b'\n' {
                    let line_bytes = &buf[..i];
                    let line_str = String::from_utf8_lossy(line_bytes).to_string();
                    self.buffer.drain(..=i + 1);
                    return Some(line_str);
                } else if i + 1 == len {
                    return None;
                } else {
                    let line_bytes = &buf[..i];
                    let line_str = String::from_utf8_lossy(line_bytes).to_string();
                    self.buffer.drain(..=i);
                    return Some(line_str);
                }
            }
            i += 1;
        }

        None
    }

    fn process_field(&mut self, line: &str) {
        let (field, value) = match line.find(':') {
            Some(colon_idx) => {
                let f = &line[..colon_idx];
                let v = &line[colon_idx + 1..];
                let v = v.strip_prefix(' ').unwrap_or(v);
                (f, v)
            }
            None => (line, ""),
        };

        match field {
            "data" => {
                self.current_data_lines.push(value.to_string());
                self.has_event_fields = true;
            }
            "event" => {
                self.current_event_type = Some(value.to_string());
                self.has_event_fields = true;
            }
            "id" => {
                if !value.contains('\0') {
                    self.current_id = Some(value.to_string());
                    self.has_event_fields = true;
                }
            }
            "retry" => {
                if let Ok(ms) = value.parse::<u64>() {
                    self.current_retry = Some(Duration::from_millis(ms));
                    self.has_event_fields = true;
                }
            }
            _ => {
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_frame_lf() {
        let mut parser = SseParser::new();
        let chunk = b"id: 1\nevent: task.started\ndata: {\"seq\":1}\n\n";
        let frames = parser.feed(chunk);
        assert_eq!(frames.len(), 1);
        let f = &frames[0];
        assert_eq!(f.id.as_deref(), Some("1"));
        assert_eq!(f.event.as_deref(), Some("task.started"));
        assert_eq!(f.data, "{\"seq\":1}");
    }

    #[test]
    fn test_single_frame_crlf() {
        let mut parser = SseParser::new();
        let chunk = b"id: 42\r\nevent: workflow.completed\r\ndata: {\"done\":true}\r\n\r\n";
        let frames = parser.feed(chunk);
        assert_eq!(frames.len(), 1);
        let f = &frames[0];
        assert_eq!(f.id.as_deref(), Some("42"));
        assert_eq!(f.event.as_deref(), Some("workflow.completed"));
        assert_eq!(f.data, "{\"done\":true}");
    }

    #[test]
    fn test_chunk_boundary_split_every_byte() {
        let mut parser = SseParser::new();
        let raw = b": comment\nid: 99\nevent: ping\ndata: {\"msg\":\"hello world\"}\n\n";
        let mut frames = Vec::new();
        for byte in raw {
            frames.extend(parser.feed(&[*byte]));
        }
        assert_eq!(frames.len(), 1);
        let f = &frames[0];
        assert_eq!(f.id.as_deref(), Some("99"));
        assert_eq!(f.event.as_deref(), Some("ping"));
        assert_eq!(f.data, "{\"msg\":\"hello world\"}");
    }

    #[test]
    fn test_multiline_data() {
        let mut parser = SseParser::new();
        let raw = b"event: log\ndata: line 1\ndata: line 2\ndata: line 3\n\n";
        let frames = parser.feed(raw);
        assert_eq!(frames.len(), 1);
        let f = &frames[0];
        assert_eq!(f.event.as_deref(), Some("log"));
        assert_eq!(f.data, "line 1\nline 2\nline 3");
    }

    #[test]
    fn test_id_preservation_and_reset() {
        let mut parser = SseParser::new();
        let raw = b"id: 1\ndata: first\n\ndata: second\n\nid: 3\ndata: third\n\n";
        let frames = parser.feed(raw);
        assert_eq!(frames.len(), 3);
        assert_eq!(frames[0].id.as_deref(), Some("1"));
        assert_eq!(frames[0].data, "first");
        assert_eq!(frames[1].id, None);
        assert_eq!(frames[1].data, "second");
        assert_eq!(frames[2].id.as_deref(), Some("3"));
        assert_eq!(frames[2].data, "third");
    }

    #[test]
    fn test_comments_and_retry() {
        let mut parser = SseParser::new();
        let raw = b": initial heartbeat\nretry: 5000\nid: 10\ndata: test\n\n";
        let frames = parser.feed(raw);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].retry, Some(Duration::from_millis(5000)));
        assert_eq!(frames[0].id.as_deref(), Some("10"));
        assert_eq!(frames[0].data, "test");
    }

    #[test]
    fn test_into_axum_event() {
        let frame = ParsedSseFrame {
            id: Some("123".into()),
            event: Some("task.started".into()),
            data: "{\"sequence\":123}".into(),
            retry: None,
        };
        let ev = frame.into_axum_event();
        let _ = ev;
    }
}
