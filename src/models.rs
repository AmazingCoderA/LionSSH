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
    /// jump | socks5 | http | none
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
        let auth = if self.user.is_empty() {
            String::new()
        } else {
            format!("{}@", self.user)
        };
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
        let user = if self.user.is_empty() {
            "user".to_string()
        } else {
            self.user.clone()
        };
        let host = if self.host.is_empty() {
            "host"
        } else {
            &self.host
        };
        let port = if self.port == 0 { 22 } else { self.port };
        format!("{user}@{host}:{port}")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Script {
    pub id: String,
    pub name: String,
    pub description: String,
    /// powershell | bash | cmd | shell | ssh
    pub kind: String,
    pub content: String,
    pub profile_id: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

impl Default for Script {
    fn default() -> Self {
        let t = now();
        Self {
            id: gen_id(),
            name: String::new(),
            description: String::new(),
            kind: "shell".into(),
            content: String::new(),
            profile_id: String::new(),
            tags: vec![],
            created_at: t,
            updated_at: t,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: String,
    pub name: String,
    /// command | script
    pub target_kind: String,
    pub target: String,
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
            id: gen_id(),
            name: String::new(),
            target_kind: "command".into(),
            target: String::new(),
            interval_secs: 60,
            run_at: None,
            enabled: true,
            last_run: None,
            last_status: "never".into(),
            run_count: 0,
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
    pub kind: String,
    pub title: String,
    pub status: String,
    pub detail: String,
}

impl HistoryEntry {
    pub fn new(
        kind: &str,
        title: impl Into<String>,
        status: &str,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            at: now(),
            kind: kind.into(),
            title: title.into(),
            status: status.into(),
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: String,
    pub default_shell: String,
    pub log_limit: usize,
    pub gui_scale: f32,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub active_ssh_profile: String,
}

fn default_language() -> String {
    "ru".into()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "cyberpunk".into(),
            default_shell: default_shell().into(),
            log_limit: 300,
            gui_scale: 1.0,
            language: default_language(),
            active_ssh_profile: String::new(),
        }
    }
}

pub fn default_shell() -> &'static str {
    if cfg!(windows) {
        "powershell"
    } else {
        "bash"
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
    #[serde(default)]
    pub tab_profiles: std::collections::BTreeMap<String, String>,
}

impl Workspace {
    pub fn data_dir() -> Result<PathBuf> {
        let base = dirs::data_local_dir()
            .or_else(dirs::home_dir)
            .context("no home/data dir")?;
        Ok(base.join("LionSSH"))
    }
    pub fn data_file() -> Result<PathBuf> {
        Ok(Self::data_dir()?.join("workspace.json"))
    }
    pub fn scripts_dir() -> Result<PathBuf> {
        Ok(Self::data_dir()?.join("scripts"))
    }
}
