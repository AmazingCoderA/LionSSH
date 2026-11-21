use anyhow::{Context, Result};
use std::fs;
use crate::models::Workspace;

pub fn load() -> Result<Workspace> {
    let path = Workspace::data_file()?;
    if !path.exists() { let ws = Workspace::default(); save(&ws)?; return Ok(ws); }
    let data = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let mut ws: Workspace = serde_json::from_str(&data).with_context(|| "parse workspace.json")?;
    let limit = ws.config.log_limit.max(50);
    while ws.history.len() > limit { ws.history.pop_front(); }
    Ok(ws)
}

pub fn save(ws: &Workspace) -> Result<()> {
    let dir = Workspace::data_dir()?; fs::create_dir_all(&dir)?;
    fs::create_dir_all(Workspace::scripts_dir()?)?;
    let path = Workspace::data_file()?;
    let data = serde_json::to_string_pretty(ws)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, data).with_context(|| format!("write {}", tmp.display()))?;
    fs::rename(&tmp, &path).with_context(|| format!("rename {}", path.display()))?;
    Ok(())
}
