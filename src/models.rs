use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use uuid::Uuid;

pub fn now() -> DateTime<Local> {
    Local::now()
}

pub fn gen_id() -> String {
    Uuid::new_v4().to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ProxyProfile {
    pub id: String,
    pub name: String,
    /// "jump" | "socks5" | "http" | "none"
    pub kind: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
}

impl ProxyProfile {
    pub fn display(&self) -> String {
        if self.kind == "none" || self.host.is_empty() {
            return "direct".into();
        }
        let auth = if self.user.is_empty() { String::new() } else { format!("{}@", self.user) };
        format!("{}://{}{}:{}", self.kind, auth, self.host, self.port)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SshProfile {
    pub id: String,
    pub name: String,
    pub host: String,
    pub user: String,
    pub port: u16,
    pub identity: String,
    pub proxy_id: String,
    pub startup_commands: Vec<String>,
    pub notes: String,
}

impl SshProfile {
    pub fn target(&self) -> String {
        let user = if self.user.is_empty() { "user".to_string() } else { self.user.clone() };
        format!("{}@{}:{}", user, if self.host.is_empty() { "host" } else { &self.host }, if self.port == 0 { 22 } else { self.port })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Script {
    pub id: String,
    pub name: String,
    pub description: String,
    /// "shell" | "powershell" | "bash" | "cmd" | "ssh"
    pub kind: String,
    pub content: String,
    /// Optional profile id for ssh scripts
    pub profile_id: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

impl Default for Script {
    fn default() -> Self {
        let t = now();
        Self {
            id: gen_id(), name: String::new(), description: String::new(), kind: "powershell".into(),
            content: String::new(), profile_id: String::new(), tags: vec![], created_at: t, updated_at: t,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: String,
    pub name: String,
    /// "command" | "script"
    pub target_kind: String,
    pub target: String, // command text or script id
    /// interval seconds; 0 means run once at run_at
    pub interval_secs: u64,
    pub run_at: Option<DateTime<Local>>,
    pub enabled: bool,
    pub last_run: Option<DateTime<Local>>,
    pub last_status: String,
    pub run_count: u64,
}

impl Default for ScheduledTask {
    fn default() -> Self {
        Self {
            id: gen_id(), name: String::new(), target_kind: "command".into(), target: String::new(),
            interval_secs: 60, run_at: None, enabled: true, last_run: None, last_status: "never".into(), run_count: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CommandAlias {
    pub id: String,
    pub name: String,
    pub command: String,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub at: DateTime<Local>,
    pub kind: String, // command/script/ssh/scheduler/system
    pub title: String,
    pub status: String, // ok/error/running/info
    pub detail: String,
}

impl HistoryEntry {
    pub fn new(kind: &str, title: impl Into<String>, status: &str, detail: impl Into<String>) -> Self {
        Self { at: now(), kind: kind.into(), title: title.into(), status: status.into(), detail: detail.into() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: String,
    pub effects: bool,
    pub effect: String, // matrix/rain/pulse/none
    pub default_shell: String, // powershell/bash/cmd
    pub log_limit: usize,
    pub scheduler_tick_ms: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "cyberpunk".into(),
            effects: true,
            effect: "pulse".into(),
            default_shell: "powershell".into(),
            log_limit: 300,
            scheduler_tick_ms: 1000,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Workspace {
    pub config: AppConfig,
    pub proxies: Vec<ProxyProfile>,
    pub ssh_profiles: Vec<SshProfile>,
    pub scripts: Vec<Script>,
    pub tasks: Vec<ScheduledTask>,
    pub aliases: Vec<CommandAlias>,
    #[serde(default)]
    pub history: VecDeque<HistoryEntry>,
}

impl Workspace {
    pub fn data_dir() -> Result<PathBuf> {
        let base = dirs::data_local_dir().or_else(dirs::home_dir).context("no home/data dir")?;
        Ok(base.join("LionSSH"))
    }
    pub fn data_file() -> Result<PathBuf> {
        Ok(Self::data_dir()?.join("workspace.json"))
    }
    pub fn scripts_dir() -> Result<PathBuf> {
        Ok(Self::data_dir()?.join("scripts"))
    }
}
