//! Strongly-typed IDE events delivered to `Plugin::on_event`.

use crate::abi::{EditorCursor, EventType, KeyEvent};

/// Decoded event dispatched by ODIT to a plugin's `on_event` handler.
#[derive(Debug, Clone)]
pub enum Event<'a> {
    ActiveFileChanged(&'a str),
    BufferModified {
        path: String,
        version: u32,
        saved: bool,
    },
    CursorMoved(EditorCursor),
    DiagnosticsUpdated(&'a str),
    SymbolsUpdated(&'a str),
    Command(String),
    DebugStateChanged(&'a str),
    KeyInput(KeyEvent),
    WorkspaceChanged(&'a str),
    Raw {
        event_type: EventType,
        payload: &'a [u8],
    },
}

/// Lightweight zero-dep JSON string field extractor (e.g. `json_get_str(js, "id")`).
pub fn json_get_str(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\"", key);
    let idx = json.find(&pattern)?;
    let rest = &json[idx + pattern.len()..];
    let colon = rest.find(':')?;
    let after = rest[colon + 1..].trim_start();
    if !after.starts_with('"') {
        return None;
    }
    let bytes = after.as_bytes();
    let mut i = 1;
    let mut out = String::new();
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\\' && i + 1 < bytes.len() {
            i += 1;
            match bytes[i] {
                b'n' => out.push('\n'),
                b'r' => out.push('\r'),
                b't' => out.push('\t'),
                b'"' => out.push('"'),
                b'\\' => out.push('\\'),
                other => out.push(other as char),
            }
        } else if b == b'"' {
            return Some(out);
        } else {
            out.push(b as char);
        }
        i += 1;
    }
    Some(out)
}

/// Lightweight zero-dep JSON integer field extractor.
pub fn json_get_u32(json: &str, key: &str) -> Option<u32> {
    let pattern = format!("\"{}\"", key);
    let idx = json.find(&pattern)?;
    let rest = &json[idx + pattern.len()..];
    let colon = rest.find(':')?;
    let after = rest[colon + 1..].trim_start();
    let end = after
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(after.len());
    after[..end].parse::<u32>().ok()
}

/// Lightweight zero-dep JSON bool field extractor.
pub fn json_get_bool(json: &str, key: &str) -> bool {
    let pattern = format!("\"{}\"", key);
    let Some(idx) = json.find(&pattern) else {
        return false;
    };
    let rest = &json[idx + pattern.len()..];
    let Some(colon) = rest.find(':') else {
        return false;
    };
    rest[colon + 1..].trim_start().starts_with("true")
}

/// Decodes raw `(event_type, payload)` bytes from the host into a typed [`Event`].
pub fn decode_event(event_type: u32, payload: &[u8]) -> Event<'_> {
    let ev = EventType::from_u32(event_type);
    let as_str = || core::str::from_utf8(payload).unwrap_or("");

    match ev {
        EventType::ActiveFileChanged => Event::ActiveFileChanged(as_str()),
        EventType::BufferModified => {
            let s = as_str();
            Event::BufferModified {
                path: json_get_str(s, "path").unwrap_or_default(),
                version: json_get_u32(s, "version").unwrap_or(0),
                saved: json_get_bool(s, "saved"),
            }
        }
        EventType::CursorMoved if payload.len() >= core::mem::size_of::<EditorCursor>() => {
            let cur = unsafe { core::ptr::read_unaligned(payload.as_ptr() as *const EditorCursor) };
            Event::CursorMoved(cur)
        }
        EventType::DiagnosticsUpdated => Event::DiagnosticsUpdated(as_str()),
        EventType::SymbolsUpdated => Event::SymbolsUpdated(as_str()),
        EventType::CommandInvoked => {
            let s = as_str();
            let id = json_get_str(s, "id").unwrap_or_else(|| s.to_string());
            Event::Command(id)
        }
        EventType::DebugStateChanged => Event::DebugStateChanged(as_str()),
        EventType::KeyInput if payload.len() >= core::mem::size_of::<KeyEvent>() => {
            let ke = unsafe { core::ptr::read_unaligned(payload.as_ptr() as *const KeyEvent) };
            Event::KeyInput(ke)
        }
        EventType::WorkspaceChanged => Event::WorkspaceChanged(as_str()),
        _ => Event::Raw {
            event_type: ev,
            payload,
        },
    }
}
