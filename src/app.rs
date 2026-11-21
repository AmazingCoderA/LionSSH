use anyhow::Result;
use chrono::{DateTime, Local};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use crate::executor::{run_shell, script_shell, RunResult};
use crate::models::*;
use crate::ssh::build_ssh_command_line;
use crate::storage;
use crate::theme::Theme;
use crate::util::{push_history, short};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Dashboard, Terminal, Ssh, Scripts, Scheduler, Proxies, Aliases, Settings,
}
impl Tab {
    pub const ALL: [Tab; 8] = [Tab::Dashboard, Tab::Terminal, Tab::Ssh, Tab::Scripts, Tab::Scheduler, Tab::Proxies, Tab::Aliases, Tab::Settings];
    pub fn title(self) -> &'static str {
        match self {
            Tab::Dashboard => "Dashboard",
            Tab::Terminal => "Terminal",
            Tab::Ssh => "SSH",
            Tab::Scripts => "Scripts",
            Tab::Scheduler => "Scheduler",
            Tab::Proxies => "Proxies",
            Tab::Aliases => "Aliases",
            Tab::Settings => "Settings",
        }
    }
    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|t| *t == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }
    pub fn prev(self) -> Self {
        let i = Self::ALL.iter().position(|t| *t == self).unwrap_or(0);
        Self::ALL[(i + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

#[derive(Clone, Debug)]
pub enum Mode {
    Normal,
    Input(InputKind),
    Confirm(ConfirmKind),
    Help,
}

#[derive(Clone, Debug)]
pub enum InputKind {
    TerminalCommand,
    SshName, SshHost, SshUser, SshPort, SshIdentity, SshProxy, SshStartup, SshNotes,
    ScriptName, ScriptKind, ScriptDescription, ScriptContent, ScriptTags, ScriptProfile,
    TaskName, TaskKind, TaskTarget, TaskInterval, TaskRunAt,
    ProxyName, ProxyKind, ProxyHost, ProxyPort, ProxyUser, ProxyPassword,
    AliasName, AliasCommand, AliasDescription,
    SettingsTheme, SettingsEffect, SettingsShell, SettingsLogLimit,
}

#[derive(Clone, Debug)]
pub enum ConfirmKind {
    DeleteSsh(String),
    DeleteScript(String),
    DeleteTask(String),
    DeleteProxy(String),
    DeleteAlias(String),
}

pub struct RunJob {
    pub kind: String,
    pub title: String,
    pub result: Result<RunResult, String>,
}

pub struct App {
    pub ws: Workspace,
    pub tab: Tab,
    pub mode: Mode,
    pub should_quit: bool,
    pub status: String,
    pub input: String,
    pub input_prompt: String,
    pub selected_ssh: usize,
    pub selected_script: usize,
    pub selected_task: usize,
    pub selected_proxy: usize,
    pub selected_alias: usize,
    pub selected_history: usize,
    pub terminal_output: VecDeque<String>,
    pub terminal_scroll: usize,
    pub last_tick: DateTime<Local>,
    pub tick: u64,
    pub busy: usize,
    pub run_tx: Sender<RunJob>,
    pub run_rx: Receiver<RunJob>,
    pub theme: Theme,
    // draft objects for forms
    pub draft_ssh: SshProfile,
    pub draft_script: Script,
    pub draft_task: ScheduledTask,
    pub draft_proxy: ProxyProfile,
    pub draft_alias: CommandAlias,
    pub editing_id: Option<String>,
}

impl App {
    pub fn new() -> Result<Self> {
        let ws = storage::load()?;
        let theme = Theme::by_name(&ws.config.theme).unwrap_or_else(|| Theme::presets()[0].clone());
        let (run_tx, run_rx) = mpsc::channel();
        let mut app = Self {
            ws, tab: Tab::Dashboard, mode: Mode::Normal, should_quit: false,
            status: "LionSSH ready. Press ? for help.".into(),
            input: String::new(), input_prompt: String::new(),
            selected_ssh: 0, selected_script: 0, selected_task: 0, selected_proxy: 0, selected_alias: 0, selected_history: 0,
            terminal_output: VecDeque::new(), terminal_scroll: 0,
            last_tick: Local::now(), tick: 0, busy: 0, run_tx, run_rx, theme,
            draft_ssh: SshProfile::default(), draft_script: Script::default(), draft_task: ScheduledTask::default(),
            draft_proxy: ProxyProfile::default(), draft_alias: CommandAlias::default(), editing_id: None,
        };
        app.push_terminal("LionSSH mega command center booted.");
        app.push_terminal(format!("Data: {}", Workspace::data_file().unwrap_or_else(|_| PathBuf::from("?")).display()));
        Ok(app)
    }

    pub fn save(&mut self) {
        if let Err(e) = storage::save(&self.ws) {
            self.status = format!("Save error: {e}");
        }
    }

    pub fn push_terminal<S: Into<String>>(&mut self, line: S) {
        let line = line.into();
        for l in line.lines() {
            self.terminal_output.push_back(l.to_string());
        }
        while self.terminal_output.len() > 2000 { self.terminal_output.pop_front(); }
        self.terminal_scroll = 0;
    }

    pub fn log(&mut self, kind: &str, title: impl Into<String>, status: &str, detail: impl Into<String>) {
        let entry = HistoryEntry::new(kind, title, status, detail);
        self.push_terminal(format!("[{}] {} :: {} - {}", entry.at.format("%H:%M:%S"), entry.kind, entry.status, entry.title));
        push_history(&mut self.ws, entry);
        self.save();
    }

    pub fn on_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        self.drain_jobs();
        self.run_due_tasks();
    }

    fn drain_jobs(&mut self) {
        while let Ok(job) = self.run_rx.try_recv() {
            self.busy = self.busy.saturating_sub(1);
            match job.result {
                Ok(res) => {
                    let status = if res.ok { "ok" } else { "error" };
                    let detail = if res.output.trim().is_empty() { format!("exit {}", res.status) } else { res.output.clone() };
                    self.push_terminal(format!("--- {} ({}, {}ms) ---", job.title, res.status, res.duration_ms));
                    for l in detail.lines().take(200) { self.push_terminal(l.to_string()); }
                    self.log(&job.kind, job.title, status, short(&detail, 500));
                    self.status = format!("{}: {}", if res.ok { "OK" } else { "ERR" }, res.status);
                }
                Err(e) => {
                    self.push_terminal(format!("--- {} ERROR ---", job.title));
                    self.push_terminal(e.clone());
                    self.log(&job.kind, job.title, "error", short(&e, 500));
                    self.status = format!("Error: {}", short(&e, 80));
                }
            }
        }
    }

    fn spawn_run(&mut self, kind: &str, title: String, shell: String, command: String) {
        self.busy += 1;
        self.push_terminal(format!(">>> {title}"));
        let tx = self.run_tx.clone();
        let kind = kind.to_string();
        thread::spawn(move || {
            let result = run_shell(&shell, &command, Some(Duration::from_secs(300))).map_err(|e| e.to_string());
            let _ = tx.send(RunJob { kind, title, result });
        });
    }

    pub fn run_command(&mut self, command: String) {
        let cmd = command.trim().to_string();
        if cmd.is_empty() { return; }
        // alias expansion: if matches alias name exactly
        let resolved = self.ws.aliases.iter().find(|a| a.name == cmd).map(|a| a.command.clone()).unwrap_or(cmd.clone());
        let shell = self.ws.config.default_shell.clone();
        self.spawn_run("command", short(&resolved, 60), shell, resolved);
    }

    pub fn run_script_by_id(&mut self, id: &str) {
        let Some(script) = self.ws.scripts.iter().find(|s| s.id == id).cloned() else { self.status = "Script not found".into(); return; };
        if script.kind == "ssh" {
            self.run_ssh_script(&script);
            return;
        }
        let shell = script_shell(&script.kind, &self.ws.config.default_shell).to_string();
        self.spawn_run("script", format!("script:{}", script.name), shell, script.content);
    }

    fn run_ssh_script(&mut self, script: &Script) {
        let Some(profile) = self.ws.ssh_profiles.iter().find(|p| p.id == script.profile_id).cloned() else {
            self.status = "SSH profile for script not found".into();
            self.log("script", &script.name, "error", "SSH profile missing");
            return;
        };
        let proxy = self.ws.proxies.iter().find(|p| p.id == profile.proxy_id).cloned();
        let cmdline = build_ssh_command_line(&profile, proxy.as_ref(), Some(&script.content));
        let shell = self.ws.config.default_shell.clone();
        self.spawn_run("ssh", format!("ssh-script:{}", script.name), shell, cmdline);
    }

    pub fn connect_ssh(&mut self, profile_id: &str) {
        let Some(profile) = self.ws.ssh_profiles.iter().find(|p| p.id == profile_id).cloned() else { self.status = "Profile not found".into(); return; };
        let proxy = self.ws.proxies.iter().find(|p| p.id == profile.proxy_id).cloned();
        let startup = if profile.startup_commands.is_empty() { None } else { Some(profile.startup_commands.join(" && ")) };
        let cmdline = build_ssh_command_line(&profile, proxy.as_ref(), startup.as_deref());
        self.push_terminal(format!("SSH command: {cmdline}"));
        self.push_terminal("Launching interactive SSH in a new terminal window when possible; fallback runs in-app capture.");
        // Try launching external terminal on Windows for interactive ssh
        #[cfg(windows)]
        {
            let escaped = cmdline.replace('"', "`\"");
            let ps = format!("Start-Process powershell -ArgumentList '-NoExit','-Command','{escaped}'");
            let res = run_shell("powershell", &ps, Some(Duration::from_secs(15)));
            match res {
                Ok(r) if r.ok => { self.log("ssh", format!("connect {}", profile.name), "ok", cmdline); self.status = "SSH launched".into(); return; }
                Ok(r) => { self.push_terminal(format!("External launch failed: {}", r.output)); }
                Err(e) => { self.push_terminal(format!("External launch error: {e}")); }
            }
        }
        let shell = self.ws.config.default_shell.clone();
        self.spawn_run("ssh", format!("ssh:{}", profile.name), shell, cmdline);
    }
    fn run_due_tasks(&mut self) {
        let now = Local::now();
        let due: Vec<String> = self.ws.tasks.iter().filter(|t| {
            if !t.enabled { return false; }
            if let Some(run_at) = t.run_at {
                if t.last_run.is_none() && now >= run_at { return true; }
            }
            if t.interval_secs == 0 { return false; }
            match t.last_run {
                None => true,
                Some(lr) => now.signed_duration_since(lr).num_seconds() >= t.interval_secs as i64,
            }
        }).map(|t| t.id.clone()).collect();

        for id in due {
            let Some(task) = self.ws.tasks.iter().cloned().find(|t| t.id == id) else { continue; };
            if let Some(t) = self.ws.tasks.iter_mut().find(|t| t.id == id) {
                t.last_run = Some(now);
                t.run_count += 1;
                t.last_status = "triggered".into();
            }
            self.save();
            match task.target_kind.as_str() {
                "script" => self.run_script_by_id(&task.target),
                _ => self.run_command(task.target.clone()),
            }
            self.log("scheduler", format!("task:{}", task.name), "info", "triggered");
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if matches!(self.mode, Mode::Help) {
            self.mode = Mode::Normal; return;
        }
        if let Mode::Confirm(kind) = self.mode.clone() {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => { self.confirm(kind); self.mode = Mode::Normal; }
                _ => { self.mode = Mode::Normal; self.status = "Cancelled".into(); }
            }
            return;
        }
        if let Mode::Input(kind) = self.mode.clone() {
            self.handle_input_key(kind, key); return;
        }
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('?') => self.mode = Mode::Help,
            KeyCode::Tab => self.tab = self.tab.next(),
            KeyCode::BackTab => self.tab = self.tab.prev(),
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => { self.save(); self.status = "Saved".into(); },
            KeyCode::Char('1') => self.tab = Tab::Dashboard,
            KeyCode::Char('2') => self.tab = Tab::Terminal,
            KeyCode::Char('3') => self.tab = Tab::Ssh,
            KeyCode::Char('4') => self.tab = Tab::Scripts,
            KeyCode::Char('5') => self.tab = Tab::Scheduler,
            KeyCode::Char('6') => self.tab = Tab::Proxies,
            KeyCode::Char('7') => self.tab = Tab::Aliases,
            KeyCode::Char('8') => self.tab = Tab::Settings,
            _ => self.handle_tab_key(key),
        }
    }

    fn handle_tab_key(&mut self, key: KeyEvent) {
        match self.tab {
            Tab::Dashboard | Tab::Terminal => self.handle_terminal_key(key),
            Tab::Ssh => self.handle_ssh_key(key),
            Tab::Scripts => self.handle_scripts_key(key),
            Tab::Scheduler => self.handle_scheduler_key(key),
            Tab::Proxies => self.handle_proxies_key(key),
            Tab::Aliases => self.handle_aliases_key(key),
            Tab::Settings => self.handle_settings_key(key),
        }
    }

    fn move_sel(sel: &mut usize, len: usize, delta: i32) {
        if len == 0 { *sel = 0; return; }
        let cur = *sel as i32;
        let next = (cur + delta).rem_euclid(len as i32) as usize;
        *sel = next;
    }

    fn handle_terminal_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('c') if key.modifiers.is_empty() => self.begin_input(InputKind::TerminalCommand, "Command (alias or shell): ", ""),
            KeyCode::Char('l') => { self.terminal_output.clear(); self.status = "Terminal cleared".into(); }
            KeyCode::Up => { self.terminal_scroll = self.terminal_scroll.saturating_add(1); }
            KeyCode::Down => { self.terminal_scroll = self.terminal_scroll.saturating_sub(1); }
            _ => {}
        }
    }
    fn begin_input(&mut self, kind: InputKind, prompt: &str, initial: &str) {
        self.input_prompt = prompt.into();
        self.input = initial.into();
        self.mode = Mode::Input(kind);
    }

    fn handle_input_key(&mut self, kind: InputKind, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => { self.mode = Mode::Normal; self.status = "Input cancelled".into(); }
            KeyCode::Enter => { let value = self.input.clone(); self.mode = Mode::Normal; self.apply_input(kind, value); }
            KeyCode::Backspace => { self.input.pop(); }
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) { return; }
                self.input.push(c);
            }
            _ => {}
        }
    }

    fn apply_input(&mut self, kind: InputKind, value: String) {
        let v = value.trim().to_string();
        match kind {
            InputKind::TerminalCommand => self.run_command(v),
            InputKind::SshName => { self.draft_ssh.name = v; self.begin_input(InputKind::SshHost, "Host: ", &self.draft_ssh.host.clone()); }
            InputKind::SshHost => { self.draft_ssh.host = v; self.begin_input(InputKind::SshUser, "User: ", &self.draft_ssh.user.clone()); }
            InputKind::SshUser => { self.draft_ssh.user = v; self.begin_input(InputKind::SshPort, "Port (22): ", &if self.draft_ssh.port==0{"22".into()}else{self.draft_ssh.port.to_string()}); }
            InputKind::SshPort => { self.draft_ssh.port = v.parse().unwrap_or(22); self.begin_input(InputKind::SshIdentity, "Identity file (optional): ", &self.draft_ssh.identity.clone()); }
            InputKind::SshIdentity => { self.draft_ssh.identity = v; self.begin_input(InputKind::SshProxy, "Proxy id (empty for direct): ", &self.draft_ssh.proxy_id.clone()); }
            InputKind::SshProxy => { self.draft_ssh.proxy_id = v; self.begin_input(InputKind::SshStartup, "Startup commands, separated by ;; (optional): ", &self.draft_ssh.startup_commands.join(";;")); }
            InputKind::SshStartup => { self.draft_ssh.startup_commands = if v.is_empty(){vec![]} else { v.split(";;").map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect() }; self.begin_input(InputKind::SshNotes, "Notes (optional): ", &self.draft_ssh.notes.clone()); }
            InputKind::SshNotes => { self.draft_ssh.notes = v; self.finish_ssh(); }

            InputKind::ScriptName => { self.draft_script.name = v; self.begin_input(InputKind::ScriptKind, "Kind [powershell/bash/cmd/ssh/shell]: ", &self.draft_script.kind.clone()); }
            InputKind::ScriptKind => { self.draft_script.kind = if v.is_empty(){"powershell".into()} else {v}; self.begin_input(InputKind::ScriptDescription, "Description: ", &self.draft_script.description.clone()); }
            InputKind::ScriptDescription => { self.draft_script.description = v; self.begin_input(InputKind::ScriptProfile, "SSH profile id (only for ssh kind, else empty): ", &self.draft_script.profile_id.clone()); }
            InputKind::ScriptProfile => { self.draft_script.profile_id = v; self.begin_input(InputKind::ScriptTags, "Tags comma-separated: ", &self.draft_script.tags.join(",")); }
            InputKind::ScriptTags => { self.draft_script.tags = if v.is_empty(){vec![]} else { v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect() }; self.begin_input(InputKind::ScriptContent, "Content/command (single line; use ;; for lines): ", &self.draft_script.content.replace('\n', ";;")); }
            InputKind::ScriptContent => { self.draft_script.content = v.replace(";;", "\n"); self.finish_script(); }

            InputKind::TaskName => { self.draft_task.name = v; self.begin_input(InputKind::TaskKind, "Target kind [command/script]: ", &self.draft_task.target_kind.clone()); }
            InputKind::TaskKind => { self.draft_task.target_kind = if v.is_empty(){"command".into()} else {v}; self.begin_input(InputKind::TaskTarget, "Target (command text or script id): ", &self.draft_task.target.clone()); }
            InputKind::TaskTarget => { self.draft_task.target = v; self.begin_input(InputKind::TaskInterval, "Interval seconds (0=once): ", &self.draft_task.interval_secs.to_string()); }
            InputKind::TaskInterval => { self.draft_task.interval_secs = v.parse().unwrap_or(60); self.begin_input(InputKind::TaskRunAt, "Run at RFC3339 or empty: ", &self.draft_task.run_at.map(|d| d.to_rfc3339()).unwrap_or_default()); }
            InputKind::TaskRunAt => { self.draft_task.run_at = if v.is_empty(){None} else { DateTime::parse_from_rfc3339(&v).ok().map(|d| d.with_timezone(&Local)) }; self.finish_task(); }

            InputKind::ProxyName => { self.draft_proxy.name = v; self.begin_input(InputKind::ProxyKind, "Kind [jump/socks5/http]: ", &self.draft_proxy.kind.clone()); }
            InputKind::ProxyKind => { self.draft_proxy.kind = if v.is_empty(){"socks5".into()} else {v}; self.begin_input(InputKind::ProxyHost, "Host: ", &self.draft_proxy.host.clone()); }
            InputKind::ProxyHost => { self.draft_proxy.host = v; self.begin_input(InputKind::ProxyPort, "Port: ", &if self.draft_proxy.port==0{String::new()} else {self.draft_proxy.port.to_string()}); }
            InputKind::ProxyPort => { self.draft_proxy.port = v.parse().unwrap_or(1080); self.begin_input(InputKind::ProxyUser, "User (optional): ", &self.draft_proxy.user.clone()); }
            InputKind::ProxyUser => { self.draft_proxy.user = v; self.begin_input(InputKind::ProxyPassword, "Password (optional): ", &self.draft_proxy.password.clone()); }
            InputKind::ProxyPassword => { self.draft_proxy.password = v; self.finish_proxy(); }

            InputKind::AliasName => { self.draft_alias.name = v; self.begin_input(InputKind::AliasCommand, "Command: ", &self.draft_alias.command.clone()); }
            InputKind::AliasCommand => { self.draft_alias.command = v; self.begin_input(InputKind::AliasDescription, "Description: ", &self.draft_alias.description.clone()); }
            InputKind::AliasDescription => { self.draft_alias.description = v; self.finish_alias(); }

            InputKind::SettingsTheme => { if Theme::by_name(&v).is_some() { self.ws.config.theme = v.clone(); self.theme = Theme::by_name(&v).unwrap(); self.save(); self.status = format!("Theme: {v}"); } else { self.status = "Unknown theme".into(); } }
            InputKind::SettingsEffect => { self.ws.config.effect = v; self.save(); self.status = "Effect updated".into(); }
            InputKind::SettingsShell => { self.ws.config.default_shell = v; self.save(); self.status = "Default shell updated".into(); }
            InputKind::SettingsLogLimit => { self.ws.config.log_limit = v.parse().unwrap_or(300).max(50); self.save(); self.status = "Log limit updated".into(); }
        }
    }
    fn finish_ssh(&mut self) {
        if self.draft_ssh.name.is_empty() || self.draft_ssh.host.is_empty() { self.status = "SSH name/host required".into(); return; }
        if self.draft_ssh.id.is_empty() { self.draft_ssh.id = gen_id(); }
        if let Some(id) = &self.editing_id {
            if let Some(p) = self.ws.ssh_profiles.iter_mut().find(|p| &p.id == id) { *p = self.draft_ssh.clone(); self.status = "SSH profile updated".into(); }
        } else {
            self.ws.ssh_profiles.push(self.draft_ssh.clone());
            self.selected_ssh = self.ws.ssh_profiles.len().saturating_sub(1);
            self.status = "SSH profile added".into();
        }
        self.editing_id = None; self.save();
    }
    fn finish_script(&mut self) {
        if self.draft_script.name.is_empty() { self.status = "Script name required".into(); return; }
        if self.draft_script.id.is_empty() { self.draft_script.id = gen_id(); }
        self.draft_script.updated_at = now();
        if let Some(id) = &self.editing_id {
            if let Some(s) = self.ws.scripts.iter_mut().find(|s| &s.id == id) { let created = s.created_at; *s = self.draft_script.clone(); s.created_at = created; self.status = "Script updated".into(); }
        } else {
            self.ws.scripts.push(self.draft_script.clone());
            self.selected_script = self.ws.scripts.len().saturating_sub(1);
            self.status = "Script added".into();
        }
        self.editing_id = None; self.save();
    }
    fn finish_task(&mut self) {
        if self.draft_task.name.is_empty() || self.draft_task.target.is_empty() { self.status = "Task name/target required".into(); return; }
        if self.draft_task.id.is_empty() { self.draft_task.id = gen_id(); }
        if let Some(id) = &self.editing_id {
            if let Some(t) = self.ws.tasks.iter_mut().find(|t| &t.id == id) { *t = self.draft_task.clone(); self.status = "Task updated".into(); }
        } else {
            self.ws.tasks.push(self.draft_task.clone());
            self.selected_task = self.ws.tasks.len().saturating_sub(1);
            self.status = "Task added".into();
        }
        self.editing_id = None; self.save();
    }
    fn finish_proxy(&mut self) {
        if self.draft_proxy.name.is_empty() || self.draft_proxy.host.is_empty() { self.status = "Proxy name/host required".into(); return; }
        if self.draft_proxy.id.is_empty() { self.draft_proxy.id = gen_id(); }
        if let Some(id) = &self.editing_id {
            if let Some(p) = self.ws.proxies.iter_mut().find(|p| &p.id == id) { *p = self.draft_proxy.clone(); self.status = "Proxy updated".into(); }
        } else {
            self.ws.proxies.push(self.draft_proxy.clone());
            self.selected_proxy = self.ws.proxies.len().saturating_sub(1);
            self.status = "Proxy added".into();
        }
        self.editing_id = None; self.save();
    }
    fn finish_alias(&mut self) {
        if self.draft_alias.name.is_empty() || self.draft_alias.command.is_empty() { self.status = "Alias name/command required".into(); return; }
        if self.draft_alias.id.is_empty() { self.draft_alias.id = gen_id(); }
        if let Some(id) = &self.editing_id {
            if let Some(a) = self.ws.aliases.iter_mut().find(|a| &a.id == id) { *a = self.draft_alias.clone(); self.status = "Alias updated".into(); }
        } else {
            self.ws.aliases.push(self.draft_alias.clone());
            self.selected_alias = self.ws.aliases.len().saturating_sub(1);
            self.status = "Alias added".into();
        }
        self.editing_id = None; self.save();
    }
    fn handle_ssh_key(&mut self, key: KeyEvent) {
        let len = self.ws.ssh_profiles.len();
        match key.code {
            KeyCode::Up => Self::move_sel(&mut self.selected_ssh, len, -1),
            KeyCode::Down => Self::move_sel(&mut self.selected_ssh, len, 1),
            KeyCode::Char('a') => { self.editing_id = None; self.draft_ssh = SshProfile { id: String::new(), port: 22, ..Default::default() }; self.begin_input(InputKind::SshName, "SSH profile name: ", ""); }
            KeyCode::Char('e') => { if let Some(p) = self.ws.ssh_profiles.get(self.selected_ssh).cloned() { self.editing_id = Some(p.id.clone()); self.draft_ssh = p.clone(); self.begin_input(InputKind::SshName, "SSH profile name: ", &p.name); } }
            KeyCode::Char('d') => { if let Some(p) = self.ws.ssh_profiles.get(self.selected_ssh) { self.mode = Mode::Confirm(ConfirmKind::DeleteSsh(p.id.clone())); } }
            KeyCode::Enter | KeyCode::Char('c') => { if let Some(p) = self.ws.ssh_profiles.get(self.selected_ssh) { let id = p.id.clone(); self.connect_ssh(&id); } }
            KeyCode::Char('t') => { if let Some(p) = self.ws.ssh_profiles.get(self.selected_ssh).cloned() { let proxy = self.ws.proxies.iter().find(|x| x.id == p.proxy_id).cloned(); let cmd = build_ssh_command_line(&p, proxy.as_ref(), None); self.push_terminal(format!("SSH: {cmd}")); self.status = "SSH command shown in terminal".into(); } }
            _ => {}
        }
    }

    fn handle_scripts_key(&mut self, key: KeyEvent) {
        let len = self.ws.scripts.len();
        match key.code {
            KeyCode::Up => Self::move_sel(&mut self.selected_script, len, -1),
            KeyCode::Down => Self::move_sel(&mut self.selected_script, len, 1),
            KeyCode::Char('a') => { self.editing_id = None; self.draft_script = Script::default(); self.begin_input(InputKind::ScriptName, "Script name: ", ""); }
            KeyCode::Char('e') => { if let Some(s) = self.ws.scripts.get(self.selected_script).cloned() { self.editing_id = Some(s.id.clone()); self.draft_script = s.clone(); self.begin_input(InputKind::ScriptName, "Script name: ", &s.name); } }
            KeyCode::Char('d') => { if let Some(s) = self.ws.scripts.get(self.selected_script) { self.mode = Mode::Confirm(ConfirmKind::DeleteScript(s.id.clone())); } }
            KeyCode::Enter | KeyCode::Char('r') => { if let Some(s) = self.ws.scripts.get(self.selected_script) { let id = s.id.clone(); self.run_script_by_id(&id); } }
            KeyCode::Char('x') => { // export to scripts dir
                if let Some(s) = self.ws.scripts.get(self.selected_script).cloned() {
                    if let Ok(dir) = Workspace::scripts_dir() {
                        let _ = std::fs::create_dir_all(&dir);
                        let ext = match s.kind.as_str() { "powershell" => "ps1", "bash" => "sh", "cmd" => "cmd", _ => "txt" };
                        let path = dir.join(format!("{}.{}", sanitize_filename(&s.name), ext));
                        if std::fs::write(&path, &s.content).is_ok() { self.status = format!("Exported {}", path.display()); self.push_terminal(format!("Exported script to {}", path.display())); } else { self.status = "Export failed".into(); }
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_scheduler_key(&mut self, key: KeyEvent) {
        let len = self.ws.tasks.len();
        match key.code {
            KeyCode::Up => Self::move_sel(&mut self.selected_task, len, -1),
            KeyCode::Down => Self::move_sel(&mut self.selected_task, len, 1),
            KeyCode::Char('a') => { self.editing_id = None; self.draft_task = ScheduledTask::default(); self.begin_input(InputKind::TaskName, "Task name: ", ""); }
            KeyCode::Char('e') => { if let Some(t) = self.ws.tasks.get(self.selected_task).cloned() { self.editing_id = Some(t.id.clone()); self.draft_task = t.clone(); self.begin_input(InputKind::TaskName, "Task name: ", &t.name); } }
            KeyCode::Char('d') => { if let Some(t) = self.ws.tasks.get(self.selected_task) { self.mode = Mode::Confirm(ConfirmKind::DeleteTask(t.id.clone())); } }
            KeyCode::Char(' ') | KeyCode::Char('p') => { if let Some(t) = self.ws.tasks.get_mut(self.selected_task) { t.enabled = !t.enabled; self.status = format!("Task {}", if t.enabled {"enabled"} else {"paused"}); self.save(); } }
            KeyCode::Enter | KeyCode::Char('r') => {
                if let Some(t) = self.ws.tasks.get(self.selected_task).cloned() {
                    match t.target_kind.as_str() { "script" => self.run_script_by_id(&t.target), _ => self.run_command(t.target) }
                }
            }
            _ => {}
        }
    }

    fn handle_proxies_key(&mut self, key: KeyEvent) {
        let len = self.ws.proxies.len();
        match key.code {
            KeyCode::Up => Self::move_sel(&mut self.selected_proxy, len, -1),
            KeyCode::Down => Self::move_sel(&mut self.selected_proxy, len, 1),
            KeyCode::Char('a') => { self.editing_id = None; self.draft_proxy = ProxyProfile { kind: "socks5".into(), port: 1080, ..Default::default() }; self.begin_input(InputKind::ProxyName, "Proxy name: ", ""); }
            KeyCode::Char('e') => { if let Some(p) = self.ws.proxies.get(self.selected_proxy).cloned() { self.editing_id = Some(p.id.clone()); self.draft_proxy = p.clone(); self.begin_input(InputKind::ProxyName, "Proxy name: ", &p.name); } }
            KeyCode::Char('d') => { if let Some(p) = self.ws.proxies.get(self.selected_proxy) { self.mode = Mode::Confirm(ConfirmKind::DeleteProxy(p.id.clone())); } }
            _ => {}
        }
    }

    fn handle_aliases_key(&mut self, key: KeyEvent) {
        let len = self.ws.aliases.len();
        match key.code {
            KeyCode::Up => Self::move_sel(&mut self.selected_alias, len, -1),
            KeyCode::Down => Self::move_sel(&mut self.selected_alias, len, 1),
            KeyCode::Char('a') => { self.editing_id = None; self.draft_alias = CommandAlias::default(); self.begin_input(InputKind::AliasName, "Alias name: ", ""); }
            KeyCode::Char('e') => { if let Some(a) = self.ws.aliases.get(self.selected_alias).cloned() { self.editing_id = Some(a.id.clone()); self.draft_alias = a.clone(); self.begin_input(InputKind::AliasName, "Alias name: ", &a.name); } }
            KeyCode::Char('d') => { if let Some(a) = self.ws.aliases.get(self.selected_alias) { self.mode = Mode::Confirm(ConfirmKind::DeleteAlias(a.id.clone())); } }
            KeyCode::Enter | KeyCode::Char('r') => { if let Some(a) = self.ws.aliases.get(self.selected_alias).cloned() { self.run_command(a.command); } }
            _ => {}
        }
    }

    fn handle_settings_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('t') => { self.begin_input(InputKind::SettingsTheme, "Theme (cyberpunk/dracula/matrix/nord/solarized/aurora/blood/vaporwave): ", &self.ws.config.theme.clone()); }
            KeyCode::Char('e') => { self.ws.config.effects = !self.ws.config.effects; self.save(); self.status = format!("Effects {}", if self.ws.config.effects {"on"} else {"off"}); }
            KeyCode::Char('m') => { self.begin_input(InputKind::SettingsEffect, "Effect [pulse/matrix/rain/none]: ", &self.ws.config.effect.clone()); }
            KeyCode::Char('s') => { self.begin_input(InputKind::SettingsShell, "Default shell [powershell/bash/cmd]: ", &self.ws.config.default_shell.clone()); }
            KeyCode::Char('l') => { self.begin_input(InputKind::SettingsLogLimit, "History limit: ", &self.ws.config.log_limit.to_string()); }
            _ => {}
        }
    }

    fn confirm(&mut self, kind: ConfirmKind) {
        match kind {
            ConfirmKind::DeleteSsh(id) => { self.ws.ssh_profiles.retain(|p| p.id != id); self.selected_ssh = self.selected_ssh.saturating_sub(1); }
            ConfirmKind::DeleteScript(id) => { self.ws.scripts.retain(|s| s.id != id); self.selected_script = self.selected_script.saturating_sub(1); }
            ConfirmKind::DeleteTask(id) => { self.ws.tasks.retain(|t| t.id != id); self.selected_task = self.selected_task.saturating_sub(1); }
            ConfirmKind::DeleteProxy(id) => { self.ws.proxies.retain(|p| p.id != id); self.selected_proxy = self.selected_proxy.saturating_sub(1); }
            ConfirmKind::DeleteAlias(id) => { self.ws.aliases.retain(|a| a.id != id); self.selected_alias = self.selected_alias.saturating_sub(1); }
        }
        self.save();
        self.status = "Deleted".into();
    }
}

fn sanitize_filename(name: &str) -> String {
    let mut s: String = name.chars().map(|c| if c.is_alphanumeric() || c=='-' || c=='_' { c } else { '_' }).collect();
    if s.is_empty() { s = "script".into(); }
    s
}
