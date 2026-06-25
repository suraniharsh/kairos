//! Minimal hand-rolled JSON encoding for CLI `--json` output.
//!
//! The shapes are small and fixed (a flat task object plus string arrays), so a
//! ~40-line encoder beats pulling in `serde`/`serde_json` — keeping the
//! size-optimized release binary dependency-free.

use crate::todo::Task;

/// Append a JSON-escaped, double-quoted string.
pub fn esc(s: &str, out: &mut String) {
    out.push('"');
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
    out.push('"');
}

fn opt(out: &mut String, key: &str, value: &Option<String>) {
    esc(key, out);
    out.push(':');
    match value {
        Some(v) => esc(v, out),
        None => out.push_str("null"),
    }
}

fn arr(out: &mut String, key: &str, values: &[String]) {
    esc(key, out);
    out.push_str(":[");
    for (i, v) in values.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        esc(v, out);
    }
    out.push(']');
}

/// Serialize one task as a JSON object with its 1-based number `n`.
pub fn task_object(n: usize, t: &Task, out: &mut String) {
    out.push('{');
    esc("n", out);
    out.push(':');
    out.push_str(&n.to_string());
    out.push(',');
    opt(out, "raw", &Some(t.raw.clone()));
    out.push(',');
    esc("done", out);
    out.push(':');
    out.push_str(if t.done { "true" } else { "false" });
    out.push(',');
    esc("priority", out);
    out.push(':');
    match t.priority {
        Some(c) => esc(&c.to_string(), out),
        None => out.push_str("null"),
    }
    out.push(',');
    opt(out, "created", &t.created_date);
    out.push(',');
    opt(out, "completed", &t.done_date);
    out.push(',');
    arr(out, "projects", &t.projects);
    out.push(',');
    arr(out, "contexts", &t.contexts);
    out.push(',');
    opt(out, "due", &t.due);
    out.push(',');
    opt(out, "rec", &t.rec);
    out.push(',');
    opt(out, "t", &t.threshold);
    out.push('}');
}

/// A JSON array of task objects from `(n, task)` pairs.
pub fn task_array(items: &[(usize, &Task)]) -> String {
    let mut out = String::from("[");
    for (i, (n, t)) in items.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        task_object(*n, t, &mut out);
    }
    out.push(']');
    out
}

/// A JSON array of bare strings.
pub fn string_array(items: &[String]) -> String {
    let mut out = String::new();
    arr_only(&mut out, items);
    out
}

fn arr_only(out: &mut String, values: &[String]) {
    out.push('[');
    for (i, v) in values.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        esc(v, out);
    }
    out.push(']');
}
