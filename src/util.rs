use crate::models::{HistoryEntry, Workspace};

pub fn push_history(ws: &mut Workspace, entry: HistoryEntry) {
    let limit = ws.config.log_limit.max(50);
    ws.history.push_back(entry);
    while ws.history.len() > limit { ws.history.pop_front(); }
}

pub fn short(s: &str, max: usize) -> String {
    let s = s.replace('\n', " ").replace('\r', "");
    if s.chars().count() <= max { return s; }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('…'); out
}

pub fn sanitize_filename(name: &str) -> String {
    let mut s: String = name.chars().map(|c| if c.is_alphanumeric() || c=='-' || c=='_' { c } else { '_' }).collect();
    if s.is_empty() { s = "script".into(); }
    s
}
