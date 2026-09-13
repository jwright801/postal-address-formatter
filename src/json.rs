// No serde: this is the whole JSON surface the tool needs, and hand-rolling
// it means the binary has zero dependencies to audit or update.

pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

pub fn string(s: &str) -> String {
    format!("\"{}\"", escape(s))
}

pub fn nullable_string(s: &Option<String>) -> String {
    match s {
        Some(v) => string(v),
        None => "null".to_string(),
    }
}
