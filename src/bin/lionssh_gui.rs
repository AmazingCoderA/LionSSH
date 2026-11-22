#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use eframe::egui;
use lionssh::executor::{run_shell, script_shell};
use lionssh::i18n::{self, Lang};
use lionssh::ssh::build_ssh_command_line;
use lionssh::*;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 760.0])
            .with_min_inner_size([980.0, 620.0])
            .with_title("LionSSH"),
        ..Default::default()
    };
    eframe::run_native(
        "LionSSH",
        options,
        Box::new(|cc| Ok(Box::new(GuiApp::new(cc)))),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tab {
    Dashboard,
    Terminal,
    Ssh,
    Scripts,
    Scheduler,
    Proxies,
    Aliases,
    Settings,
}

struct RunMsg {
    kind: String,
    title: String,
    ok: bool,
    status: String,
    output: String,
    ms: u128,
}
struct GuiApp {
    ws: Workspace,
    tab: Tab,
    status: String,
    last_err: String,
    term_cmd: String,
    term_out: Vec<String>,
    busy: usize,
    tx: Sender<RunMsg>,
    rx: Receiver<RunMsg>,
    ssh_sel: Option<String>,
    script_sel: Option<String>,
    task_sel: Option<String>,
    proxy_sel: Option<String>,
    alias_sel: Option<String>,
    ssh_form: SshProfile,
    script_form: Script,
    task_form: ScheduledTask,
    proxy_form: ProxyProfile,
    alias_form: CommandAlias,
    form_mode: FormMode,
    last_tick: Instant,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum FormMode {
    None,
    Add,
    Edit,
}

impl GuiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(egui::Color32::from_rgb(230, 236, 245));
        cc.egui_ctx.set_visuals(visuals);
        let mut ws = storage::load().unwrap_or_default();
        if ws.config.language.is_empty() {
            ws.config.language = "ru".into();
        }
        let (tx, rx) = mpsc::channel();
        let status = i18n::t(Lang::from_code(&ws.config.language), "ready").to_string();
        Self {
            ws,
            tab: Tab::Dashboard,
            status,
            last_err: String::new(),
            term_cmd: String::new(),
            term_out: vec!["LionSSH GUI ready.".into()],
            busy: 0,
            tx,
            rx,
            ssh_sel: None,
            script_sel: None,
            task_sel: None,
            proxy_sel: None,
            alias_sel: None,
            ssh_form: SshProfile::default(),
            script_form: Script::default(),
            task_form: ScheduledTask::default(),
            proxy_form: ProxyProfile::default(),
            alias_form: CommandAlias::default(),
            form_mode: FormMode::None,
            last_tick: Instant::now(),
        }
    }
    fn lang(&self) -> Lang {
        Lang::from_code(&self.ws.config.language)
    }
    fn tr(&self, key: &str) -> &'static str {
        i18n::t(self.lang(), key)
    }
    fn active_profile_id(&self) -> String {
        let tab_key = format!("{:?}", self.tab);
        self.ws
            .tab_profiles
            .get(&tab_key)
            .cloned()
            .filter(|id| self.ws.ssh_profiles.iter().any(|p| &p.id == id))
            .or_else(|| {
                let id = self.ws.config.active_ssh_profile.clone();
                if self.ws.ssh_profiles.iter().any(|p| p.id == id) { Some(id) } else { None }
            })
            .unwrap_or_default()
    }
    fn active_profile(&self) -> Option<SshProfile> {
        let id = self.active_profile_id();
        if id.is_empty() { return None; }
        self.ws.ssh_profiles.iter().find(|p| p.id == id).cloned()
    }
    fn set_active_profile(&mut self, id: String) {
        let tab_key = format!("{:?}", self.tab);
        if id.is_empty() {
            self.ws.tab_profiles.remove(&tab_key);
        } else {
            self.ws.tab_profiles.insert(tab_key, id.clone());
        }
        self.ws.config.active_ssh_profile = id;
        self.save();
    }
    fn save(&mut self) {
        if let Err(e) = storage::save(&self.ws) {
            self.last_err = e.to_string();
        }
    }
    fn log(
        &mut self,
        kind: &str,
        title: impl Into<String>,
        status: &str,
        detail: impl Into<String>,
    ) {
        util::push_history(&mut self.ws, HistoryEntry::new(kind, title, status, detail));
        self.save();
    }
    fn push_out(&mut self, s: impl Into<String>) {
        self.term_out.push(s.into());
        if self.term_out.len() > 2000 {
            let drain = self.term_out.len() - 2000;
            self.term_out.drain(0..drain);
        }
    }
    fn spawn_shell(&mut self, kind: &str, title: String, shell: String, command: String) {
        self.spawn_shell_on_profile(kind, title, shell, command, self.active_profile())
    }

    fn spawn_shell_on_profile(&mut self, kind: &str, title: String, shell: String, command: String, profile: Option<SshProfile>) {
        let (shell, command, label) = if let Some(p) = profile {
            let proxy = self.ws.proxies.iter().find(|x| x.id == p.proxy_id).cloned();
            let cmdline = build_ssh_command_line(&p, proxy.as_ref(), Some(&command));
            (self.ws.config.default_shell.clone(), cmdline, format!("{title} @ {}", p.name))
        } else {
            (shell, command, title)
        };
        self.busy += 1;
        self.push_out(format!(">>> {label}"));
        let tx = self.tx.clone();
        let kind = kind.to_string();
        std::thread::spawn(move || {
            let r = run_shell(&shell, &command, Some(Duration::from_secs(300)));
            let msg = match r {
                Ok(res) => RunMsg {
                    kind,
                    title: label,
                    ok: res.ok,
                    status: res.status,
                    output: res.output,
                    ms: res.duration_ms,
                },
                Err(e) => RunMsg {
                    kind,
                    title: label,
                    ok: false,
                    status: "error".into(),
                    output: e.to_string(),
                    ms: 0,
                },
            };
            let _ = tx.send(msg);
        });
    }
    fn drain(&mut self) {
        while let Ok(m) = self.rx.try_recv() {
            self.busy = self.busy.saturating_sub(1);
            self.push_out(format!("--- {} ({}, {}ms) ---", m.title, m.status, m.ms));
            for l in m.output.lines().take(300) {
                self.term_out.push(l.to_string());
            }
            self.log(
                &m.kind,
                m.title.clone(),
                if m.ok { "ok" } else { "error" },
                util::short(&m.output, 500),
            );
            self.status = if m.ok {
                format!("OK: {}", m.title)
            } else {
                format!("ERR: {}", m.title)
            };
            if !m.ok && self.last_err.is_empty() {
                self.last_err = util::short(&m.output, 200);
            }
        }
    }
    fn tick_tasks(&mut self) {
        let now = chrono::Local::now();
        let due: Vec<String> = self
            .ws
            .tasks
            .iter()
            .filter(|t| {
                t.enabled
                    && ((t.interval_secs > 0
                        && t.last_run
                            .map(|lr| {
                                now.signed_duration_since(lr).num_seconds()
                                    >= t.interval_secs as i64
                            })
                            .unwrap_or(true))
                        || (t
                            .run_at
                            .map(|ra| t.last_run.is_none() && now >= ra)
                            .unwrap_or(false)))
            })
            .map(|t| t.id.clone())
            .collect();
        for id in due {
            if let Some(t) = self.ws.tasks.iter_mut().find(|t| t.id == id) {
                t.last_run = Some(now);
                t.run_count += 1;
                t.last_status = "triggered".into();
                let tk = t.clone();
                self.save();
                self.run_task(&tk);
            }
        }
    }
    fn run_task(&mut self, t: &ScheduledTask) {
        match t.target_kind.as_str() {
            "script" => self.run_script_id(&t.target),
            _ => {
                let cmd = self.resolve_alias(&t.target);
                let shell = self.ws.config.default_shell.clone();
                self.spawn_shell("scheduler", format!("task:{}", t.name), shell, cmd);
            }
        }
    }
    fn resolve_alias(&self, cmd: &str) -> String {
        self.ws
            .aliases
            .iter()
            .find(|a| a.name == cmd)
            .map(|a| a.command.clone())
            .unwrap_or_else(|| cmd.to_string())
    }
    fn run_script_id(&mut self, id: &str) {
        let Some(s) = self
            .ws
            .scripts
            .iter()
            .find(|s| s.id == id || s.name.eq_ignore_ascii_case(id))
            .cloned()
        else {
            self.status = "Script not found".into();
            return;
        };
        if s.kind == "ssh" {
            let Some(p) = self
                .ws
                .ssh_profiles
                .iter()
                .find(|p| p.id == s.profile_id)
                .cloned()
            else {
                self.status = "SSH profile missing".into();
                return;
            };
            let proxy = self.ws.proxies.iter().find(|x| x.id == p.proxy_id).cloned();
            let cmd = build_ssh_command_line(&p, proxy.as_ref(), Some(&s.content));
            let shell = self.ws.config.default_shell.clone();
            self.spawn_shell_on_profile("ssh", format!("ssh-script:{}", s.name), shell, cmd, None);
        } else {
            let shell = script_shell(&s.kind, &self.ws.config.default_shell).to_string();
            self.spawn_shell("script", format!("script:{}", s.name), shell, s.content);
        }
    }
    fn connect_ssh(&mut self, id: &str) {
        let Some(p) = self.ws.ssh_profiles.iter().find(|p| p.id == id).cloned() else {
            return;
        };
        let proxy = self.ws.proxies.iter().find(|x| x.id == p.proxy_id).cloned();
        let startup = if p.startup_commands.is_empty() {
            None
        } else {
            Some(p.startup_commands.join(" && "))
        };
        let cmdline = build_ssh_command_line(&p, proxy.as_ref(), startup.as_deref());
        self.push_out(format!("SSH: {cmdline}"));
        #[cfg(windows)]
        {
            let escaped = cmdline.replace("'", "''");
            let ps = format!(
                "Start-Process powershell -ArgumentList '-NoExit','-Command','{}'",
                escaped.replace('"', "`\"")
            );
            if let Ok(r) = run_shell("powershell", &ps, Some(Duration::from_secs(15))) {
                if r.ok {
                    self.log("ssh", format!("connect {}", p.name), "ok", cmdline);
                    self.status = "SSH launched".into();
                    return;
                }
            }
        }
        let shell = self.ws.config.default_shell.clone();
        self.spawn_shell_on_profile("ssh", format!("ssh:{}", p.name), shell, cmdline, None);
    }
}
impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain();
        if self.last_tick.elapsed() >= Duration::from_millis(1000) {
            self.last_tick = Instant::now();
            self.tick_tasks();
        }
        ctx.request_repaint_after(Duration::from_millis(500));
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading(egui::RichText::new("LionSSH").strong());
                ui.separator();
                let tabs = [
                    (Tab::Dashboard, self.tr("dashboard")),
                    (Tab::Terminal, self.tr("terminal")),
                    (Tab::Ssh, "SSH"),
                    (Tab::Scripts, self.tr("scripts")),
                    (Tab::Scheduler, self.tr("scheduler")),
                    (Tab::Proxies, self.tr("proxies")),
                    (Tab::Aliases, self.tr("aliases")),
                    (Tab::Settings, self.tr("settings")),
                ];
                for (tab, label) in tabs {
                    if ui.selectable_label(self.tab == tab, label).clicked() {
                        self.tab = tab;
                        self.form_mode = FormMode::None;
                    }
                }
                ui.separator();
                ui.label(format!("{}:", self.tr("active_profile")));
                let mut active = self.active_profile_id();
                let active_name = self.ws.ssh_profiles.iter().find(|p| p.id == active).map(|p| p.name.as_str()).unwrap_or(self.tr("no_profile")).to_string();
                let profiles: Vec<(String, String)> = self.ws.ssh_profiles.iter().map(|p| (p.id.clone(), p.name.clone())).collect();
                let mut picked: Option<String> = None;
                egui::ComboBox::from_id_salt("global_profile")
                    .selected_text(active_name)
                    .show_ui(ui, |ui| {
                        if ui.selectable_value(&mut active, String::new(), self.tr("no_profile")).changed() {
                            picked = Some(active.clone());
                        }
                        for (id, name) in &profiles {
                            if ui.selectable_value(&mut active, id.clone(), name).changed() {
                                picked = Some(active.clone());
                            }
                        }
                    });
                if let Some(id) = picked { self.set_active_profile(id); }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut lang = self.lang();
                    egui::ComboBox::from_id_salt("lang_pick")
                        .selected_text(lang.label())
                        .show_ui(ui, |ui| {
                            for l in i18n::Lang::ALL {
                                if ui.selectable_value(&mut lang, l, l.label()).changed() {
                                    self.ws.config.language = lang.code().into();
                                    self.save();
                                }
                            }
                        });
                    if self.busy > 0 {
                        ui.label(format!("running: {}", self.busy));
                    }
                    ui.label(&self.status);
                });
            });
        });
        egui::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.small(format!(
                    "Data: {}",
                    Workspace::data_file()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default()
                ));
                if !self.last_err.is_empty() {
                    ui.colored_label(egui::Color32::RED, util::short(&self.last_err, 120));
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::Dashboard => self.ui_dashboard(ui),
            Tab::Terminal => self.ui_terminal(ui),
            Tab::Ssh => self.ui_ssh(ui),
            Tab::Scripts => self.ui_scripts(ui),
            Tab::Scheduler => self.ui_scheduler(ui),
            Tab::Proxies => self.ui_proxies(ui),
            Tab::Aliases => self.ui_aliases(ui),
            Tab::Settings => self.ui_settings(ui),
        });
    }
}

impl GuiApp {
    fn ui_dashboard(&mut self, ui: &mut egui::Ui) {
        self.profile_bar(ui);
        ui.columns(2, |cols| {
            cols[0].group(|ui| {
                ui.heading(self.tr("overview"));
                ui.separator();
                ui.label(format!("SSH profiles: {}", self.ws.ssh_profiles.len()));
                ui.label(format!("Scripts: {}", self.ws.scripts.len()));
                ui.label(format!("Tasks: {}", self.ws.tasks.len()));
                ui.label(format!("Proxies: {}", self.ws.proxies.len()));
                ui.label(format!("Aliases: {}", self.ws.aliases.len()));
                ui.label(format!("Default shell: {}", self.ws.config.default_shell));
                ui.label(format!("Theme: {}", self.ws.config.theme));
            });
            cols[0].group(|ui| {
                ui.heading(self.tr("recent_activity"));
                ui.separator();
                egui::ScrollArea::vertical()
                    .max_height(320.0)
                    .show(ui, |ui| {
                        for h in self.ws.history.iter().rev().take(30) {
                            ui.horizontal(|ui| {
                                ui.monospace(h.at.format("%H:%M:%S").to_string());
                                ui.label(&h.kind);
                                ui.label(&h.status);
                                ui.label(util::short(&h.title, 40));
                            });
                        }
                    });
            });
            cols[1].heading(self.tr("quick_command"));
            cols[1].horizontal(|ui| {
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut self.term_cmd)
                        .desired_width(f32::INFINITY)
                        .hint_text("Run command or alias..."),
                );
                let run = ui.button("Run").clicked()
                    || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
                if run {
                    let cmd = self.term_cmd.trim().to_string();
                    if !cmd.is_empty() {
                        let resolved = self.resolve_alias(&cmd);
                        let shell = self.ws.config.default_shell.clone();
                        self.spawn_shell("command", util::short(&resolved, 60), shell, resolved);
                        self.term_cmd.clear();
                    }
                }
            });
            cols[1].separator();
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(&mut cols[1], |ui| {
                    for l in self.term_out.iter().rev().take(200).rev() {
                        ui.monospace(l);
                    }
                });
        });
    }

    fn ui_terminal(&mut self, ui: &mut egui::Ui) {
        self.profile_bar(ui);
        ui.horizontal(|ui| {
            let resp = ui.add(
                egui::TextEdit::singleline(&mut self.term_cmd)
                    .desired_width(ui.available_width() - 140.0)
                    .hint_text("Command or alias"),
            );
            let run = ui.button("Run").clicked()
                || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
            if run {
                let cmd = self.term_cmd.trim().to_string();
                if !cmd.is_empty() {
                    let resolved = self.resolve_alias(&cmd);
                    let shell = self.ws.config.default_shell.clone();
                    self.spawn_shell("command", util::short(&resolved, 60), shell, resolved);
                    self.term_cmd.clear();
                }
            }
            if ui.button(self.tr("clear")).clicked() {
                self.term_out.clear();
            }
        });
        ui.separator();
        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for l in &self.term_out {
                    ui.monospace(l);
                }
            });
    }
    fn ui_ssh(&mut self, ui: &mut egui::Ui) {
        self.profile_bar(ui);
        ui.horizontal(|ui| {
            if ui.button(self.tr("add")).clicked() {
                self.form_mode = FormMode::Add;
                self.ssh_form = SshProfile {
                    port: 22,
                    ..Default::default()
                };
            }
            if ui.button(self.tr("edit")).clicked() {
                if let Some(id) = &self.ssh_sel {
                    if let Some(p) = self.ws.ssh_profiles.iter().find(|p| &p.id == id).cloned() {
                        self.ssh_form = p;
                        self.form_mode = FormMode::Edit;
                    }
                }
            }
            if ui.button(self.tr("delete")).clicked() {
                if let Some(id) = self.ssh_sel.clone() {
                    self.ws.ssh_profiles.retain(|p| p.id != id);
                    self.ssh_sel = None;
                    self.save();
                }
            }
            if ui.button(self.tr("connect")).clicked() {
                if let Some(id) = self.ssh_sel.clone() {
                    self.connect_ssh(&id);
                }
            }
            if ui.button(self.tr("copy_command")).clicked() {
                if let Some(id) = &self.ssh_sel {
                    if let Some(p) = self.ws.ssh_profiles.iter().find(|p| &p.id == id).cloned() {
                        let proxy = self.ws.proxies.iter().find(|x| x.id == p.proxy_id).cloned();
                        let cmd = build_ssh_command_line(&p, proxy.as_ref(), None);
                        ui.output_mut(|o| o.copied_text = cmd.clone());
                        self.status = "SSH command copied".into();
                        self.push_out(cmd);
                    }
                }
            }
        });
        ui.separator();
        ui.columns(2, |cols| {
            cols[0].heading("Profiles");
            egui::ScrollArea::vertical().show(&mut cols[0], |ui| {
                let mut sel = self.ssh_sel.clone();
                for p in &self.ws.ssh_profiles {
                    let proxy = self
                        .ws
                        .proxies
                        .iter()
                        .find(|x| x.id == p.proxy_id)
                        .map(|x| x.name.as_str())
                        .unwrap_or("direct");
                    let label = format!("{}\n{}  [{}]", p.name, p.target(), proxy);
                    if ui
                        .selectable_label(sel.as_ref() == Some(&p.id), label)
                        .clicked()
                    {
                        sel = Some(p.id.clone());
                    }
                    ui.separator();
                }
                self.ssh_sel = sel;
            });
            cols[1].heading(if self.form_mode == FormMode::Add {
                "Add SSH profile"
            } else if self.form_mode == FormMode::Edit {
                "Edit SSH profile"
            } else {
                "Details"
            });
            if self.form_mode != FormMode::None {
                self.ssh_form_ui(&mut cols[1]);
            } else if let Some(id) = &self.ssh_sel {
                if let Some(p) = self.ws.ssh_profiles.iter().find(|p| &p.id == id).cloned() {
                    cols[1].label(format!("Name: {}", p.name));
                    cols[1].label(format!("Target: {}", p.target()));
                    cols[1].label(format!(
                        "Identity: {}",
                        if p.identity.is_empty() {
                            "<default>"
                        } else {
                            &p.identity
                        }
                    ));
                    let proxy = self
                        .ws
                        .proxies
                        .iter()
                        .find(|x| x.id == p.proxy_id)
                        .map(|x| x.display())
                        .unwrap_or_else(|| "direct".into());
                    cols[1].label(format!("Proxy: {proxy}"));
                    cols[1].label(format!(
                        "Startup: {}",
                        if p.startup_commands.is_empty() {
                            "<none>".into()
                        } else {
                            p.startup_commands.join(" && ")
                        }
                    ));
                    cols[1].label(format!("Notes: {}", p.notes));
                }
            }
        });
    }
    fn ssh_form_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Name");
        ui.text_edit_singleline(&mut self.ssh_form.name);
        ui.label("Host");
        ui.text_edit_singleline(&mut self.ssh_form.host);
        ui.label("User");
        ui.text_edit_singleline(&mut self.ssh_form.user);
        ui.label("Port");
        ui.add(egui::DragValue::new(&mut self.ssh_form.port).range(1..=65535));
        ui.label("Identity file");
        ui.text_edit_singleline(&mut self.ssh_form.identity);
        ui.label("Proxy");
        egui::ComboBox::from_id_salt("ssh_proxy")
            .selected_text(
                self.ws
                    .proxies
                    .iter()
                    .find(|p| p.id == self.ssh_form.proxy_id)
                    .map(|p| p.name.as_str())
                    .unwrap_or("direct"),
            )
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.ssh_form.proxy_id, String::new(), "direct");
                for p in &self.ws.proxies {
                    ui.selectable_value(&mut self.ssh_form.proxy_id, p.id.clone(), &p.name);
                }
            });
        ui.label("Startup commands (one per line)");
        let mut startup = self.ssh_form.startup_commands.join("\n");
        if ui
            .add(egui::TextEdit::multiline(&mut startup).desired_rows(3))
            .changed()
        {
            self.ssh_form.startup_commands = startup
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        ui.label("Notes");
        ui.add(egui::TextEdit::multiline(&mut self.ssh_form.notes).desired_rows(3));
        ui.horizontal(|ui| {
            if ui.button(self.tr("save")).clicked() {
                if self.ssh_form.name.is_empty() || self.ssh_form.host.is_empty() {
                    self.status = "Name and host required".into();
                } else {
                    if self.ssh_form.id.is_empty() {
                        self.ssh_form.id = gen_id();
                    }
                    if self.form_mode == FormMode::Edit {
                        if let Some(p) = self
                            .ws
                            .ssh_profiles
                            .iter_mut()
                            .find(|p| p.id == self.ssh_form.id)
                        {
                            *p = self.ssh_form.clone();
                        }
                    } else {
                        self.ws.ssh_profiles.push(self.ssh_form.clone());
                    }
                    self.ssh_sel = Some(self.ssh_form.id.clone());
                    self.form_mode = FormMode::None;
                    self.save();
                    self.status = "SSH saved".into();
                }
            }
            if ui.button(self.tr("cancel")).clicked() {
                self.form_mode = FormMode::None;
            }
        });
    }
    fn ui_scripts(&mut self, ui: &mut egui::Ui) {
        self.profile_bar(ui);
        ui.horizontal(|ui| {
            if ui.button(self.tr("add")).clicked() {
                self.form_mode = FormMode::Add;
                self.script_form = Script::default();
            }
            if ui.button(self.tr("edit")).clicked() {
                if let Some(id) = &self.script_sel {
                    if let Some(s) = self.ws.scripts.iter().find(|s| &s.id == id).cloned() {
                        self.script_form = s;
                        self.form_mode = FormMode::Edit;
                    }
                }
            }
            if ui.button(self.tr("delete")).clicked() {
                if let Some(id) = self.script_sel.clone() {
                    self.ws.scripts.retain(|s| s.id != id);
                    self.script_sel = None;
                    self.save();
                }
            }
            if ui.button(self.tr("run")).clicked() {
                if let Some(id) = self.script_sel.clone() {
                    self.run_script_id(&id);
                }
            }
            if ui.button(self.tr("export")).clicked() {
                if let Some(id) = &self.script_sel {
                    if let Some(s) = self.ws.scripts.iter().find(|s| &s.id == id).cloned() {
                        if let Ok(dir) = Workspace::scripts_dir() {
                            let _ = std::fs::create_dir_all(&dir);
                            let ext = match s.kind.as_str() {
                                "powershell" => "ps1",
                                "bash" => "sh",
                                "cmd" => "cmd",
                                _ => "txt",
                            };
                            let path =
                                dir.join(format!("{}.{}", util::sanitize_filename(&s.name), ext));
                            if std::fs::write(&path, &s.content).is_ok() {
                                self.status = format!("Exported {}", path.display());
                            }
                        }
                    }
                }
            }
        });
        ui.separator();
        ui.columns(2, |cols| {
            cols[0].heading("Scripts");
            egui::ScrollArea::vertical().show(&mut cols[0], |ui| {
                let mut sel = self.script_sel.clone();
                for s in &self.ws.scripts {
                    let label = format!(
                        "{}\n[{}] {}",
                        s.name,
                        s.kind,
                        util::short(&s.description, 50)
                    );
                    if ui
                        .selectable_label(sel.as_ref() == Some(&s.id), label)
                        .clicked()
                    {
                        sel = Some(s.id.clone());
                    }
                    ui.separator();
                }
                self.script_sel = sel;
            });
            cols[1].heading(if self.form_mode == FormMode::Add {
                "Add script"
            } else if self.form_mode == FormMode::Edit {
                "Edit script"
            } else {
                "Details"
            });
            if self.form_mode != FormMode::None {
                self.script_form_ui(&mut cols[1]);
            } else if let Some(id) = &self.script_sel {
                if let Some(s) = self.ws.scripts.iter().find(|s| &s.id == id).cloned() {
                    cols[1].label(format!("Name: {}", s.name));
                    cols[1].label(format!("Kind: {}", s.kind));
                    cols[1].label(format!(
                        "Tags: {}",
                        if s.tags.is_empty() {
                            "-".into()
                        } else {
                            s.tags.join(", ")
                        }
                    ));
                    cols[1].separator();
                    egui::ScrollArea::vertical().show(&mut cols[1], |ui| {
                        ui.monospace(&s.content);
                    });
                }
            }
        });
    }
    fn script_form_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Name");
        ui.text_edit_singleline(&mut self.script_form.name);
        ui.label("Kind");
        egui::ComboBox::from_id_salt("script_kind")
            .selected_text(&self.script_form.kind)
            .show_ui(ui, |ui| {
                for k in ["shell", "powershell", "bash", "cmd", "ssh"] {
                    ui.selectable_value(&mut self.script_form.kind, k.to_string(), k);
                }
            });
        ui.label("Description");
        ui.text_edit_singleline(&mut self.script_form.description);
        ui.label("SSH profile (for ssh kind)");
        egui::ComboBox::from_id_salt("script_profile")
            .selected_text(
                self.ws
                    .ssh_profiles
                    .iter()
                    .find(|p| p.id == self.script_form.profile_id)
                    .map(|p| p.name.as_str())
                    .unwrap_or("<none>"),
            )
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.script_form.profile_id, String::new(), "<none>");
                for p in &self.ws.ssh_profiles {
                    ui.selectable_value(&mut self.script_form.profile_id, p.id.clone(), &p.name);
                }
            });
        ui.label("Tags (comma separated)");
        let mut tags = self.script_form.tags.join(", ");
        if ui.text_edit_singleline(&mut tags).changed() {
            self.script_form.tags = tags
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        ui.label("Content");
        ui.add(
            egui::TextEdit::multiline(&mut self.script_form.content)
                .desired_rows(10)
                .desired_width(f32::INFINITY)
                .font(egui::TextStyle::Monospace),
        );
        ui.horizontal(|ui| {
            if ui.button(self.tr("save")).clicked() {
                if self.script_form.name.is_empty() {
                    self.status = "Script name required".into();
                } else {
                    if self.script_form.id.is_empty() {
                        self.script_form.id = gen_id();
                    }
                    self.script_form.updated_at = now();
                    if self.form_mode == FormMode::Edit {
                        if let Some(s) = self
                            .ws
                            .scripts
                            .iter_mut()
                            .find(|s| s.id == self.script_form.id)
                        {
                            let created = s.created_at;
                            *s = self.script_form.clone();
                            s.created_at = created;
                        }
                    } else {
                        self.ws.scripts.push(self.script_form.clone());
                    }
                    self.script_sel = Some(self.script_form.id.clone());
                    self.form_mode = FormMode::None;
                    self.save();
                    self.status = "Script saved".into();
                }
            }
            if ui.button(self.tr("cancel")).clicked() {
                self.form_mode = FormMode::None;
            }
        });
    }
    fn ui_scheduler(&mut self, ui: &mut egui::Ui) {
        self.profile_bar(ui);
        ui.horizontal(|ui| {
            if ui.button(self.tr("add")).clicked() {
                self.form_mode = FormMode::Add;
                self.task_form = ScheduledTask::default();
            }
            if ui.button(self.tr("edit")).clicked() {
                if let Some(id) = &self.task_sel {
                    if let Some(t) = self.ws.tasks.iter().find(|t| &t.id == id).cloned() {
                        self.task_form = t;
                        self.form_mode = FormMode::Edit;
                    }
                }
            }
            if ui.button(self.tr("delete")).clicked() {
                if let Some(id) = self.task_sel.clone() {
                    self.ws.tasks.retain(|t| t.id != id);
                    self.task_sel = None;
                    self.save();
                }
            }
            if ui.button(self.tr("run_now")).clicked() {
                if let Some(id) = &self.task_sel {
                    if let Some(t) = self.ws.tasks.iter().find(|t| &t.id == id).cloned() {
                        self.run_task(&t);
                    }
                }
            }
            if ui.button(self.tr("toggle")).clicked() {
                if let Some(id) = &self.task_sel {
                    if let Some(t) = self.ws.tasks.iter_mut().find(|t| &t.id == id) {
                        t.enabled = !t.enabled;
                        self.save();
                    }
                }
            }
        });
        ui.separator();
        ui.columns(2, |cols| {
            cols[0].heading("Tasks");
            egui::ScrollArea::vertical().show(&mut cols[0], |ui| {
                let mut sel = self.task_sel.clone();
                for t in &self.ws.tasks {
                    let label = format!(
                        "{} [{}]\n{} -> {} | {}s | runs {}",
                        t.name,
                        if t.enabled { "ON" } else { "OFF" },
                        t.target_kind,
                        util::short(&t.target, 35),
                        t.interval_secs,
                        t.run_count
                    );
                    if ui
                        .selectable_label(sel.as_ref() == Some(&t.id), label)
                        .clicked()
                    {
                        sel = Some(t.id.clone());
                    }
                    ui.separator();
                }
                self.task_sel = sel;
            });
            cols[1].heading(if self.form_mode == FormMode::Add {
                "Add task"
            } else if self.form_mode == FormMode::Edit {
                "Edit task"
            } else {
                "Details"
            });
            if self.form_mode != FormMode::None {
                self.task_form_ui(&mut cols[1]);
            } else if let Some(id) = &self.task_sel {
                if let Some(t) = self.ws.tasks.iter().find(|t| &t.id == id).cloned() {
                    cols[1].label(format!("Name: {}", t.name));
                    cols[1].label(format!("Enabled: {}", t.enabled));
                    cols[1].label(format!("Type: {}", t.target_kind));
                    cols[1].label(format!("Target: {}", t.target));
                    cols[1].label(format!("Interval: {}s", t.interval_secs));
                    cols[1].label(format!(
                        "Last run: {}",
                        t.last_run
                            .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                            .unwrap_or_else(|| "-".into())
                    ));
                    cols[1].label(format!("Last status: {}", t.last_status));
                }
            }
        });
    }
    fn task_form_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Name");
        ui.text_edit_singleline(&mut self.task_form.name);
        ui.label("Type");
        egui::ComboBox::from_id_salt("task_kind")
            .selected_text(&self.task_form.target_kind)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.task_form.target_kind, "command".into(), "command");
                ui.selectable_value(&mut self.task_form.target_kind, "script".into(), "script");
            });
        ui.label("Target (command text or script id/name)");
        ui.text_edit_singleline(&mut self.task_form.target);
        ui.label("Interval seconds (0=once)");
        ui.add(egui::DragValue::new(&mut self.task_form.interval_secs).range(0..=86400));
        ui.checkbox(&mut self.task_form.enabled, "Enabled");
        ui.horizontal(|ui| {
            if ui.button(self.tr("save")).clicked() {
                if self.task_form.name.is_empty() || self.task_form.target.is_empty() {
                    self.status = "Task name/target required".into();
                } else {
                    if self.task_form.id.is_empty() {
                        self.task_form.id = gen_id();
                    }
                    if self.form_mode == FormMode::Edit {
                        if let Some(t) =
                            self.ws.tasks.iter_mut().find(|t| t.id == self.task_form.id)
                        {
                            *t = self.task_form.clone();
                        }
                    } else {
                        self.ws.tasks.push(self.task_form.clone());
                    }
                    self.task_sel = Some(self.task_form.id.clone());
                    self.form_mode = FormMode::None;
                    self.save();
                    self.status = "Task saved".into();
                }
            }
            if ui.button(self.tr("cancel")).clicked() {
                self.form_mode = FormMode::None;
            }
        });
    }
    fn ui_proxies(&mut self, ui: &mut egui::Ui) {
        self.profile_bar(ui);
        ui.horizontal(|ui| {
            if ui.button(self.tr("add")).clicked() {
                self.form_mode = FormMode::Add;
                self.proxy_form = ProxyProfile {
                    kind: "socks5".into(),
                    port: 1080,
                    ..Default::default()
                };
            }
            if ui.button(self.tr("edit")).clicked() {
                if let Some(id) = &self.proxy_sel {
                    if let Some(p) = self.ws.proxies.iter().find(|p| &p.id == id).cloned() {
                        self.proxy_form = p;
                        self.form_mode = FormMode::Edit;
                    }
                }
            }
            if ui.button(self.tr("delete")).clicked() {
                if let Some(id) = self.proxy_sel.clone() {
                    self.ws.proxies.retain(|p| p.id != id);
                    self.proxy_sel = None;
                    self.save();
                }
            }
        });
        ui.separator();
        ui.columns(2, |cols| {
            cols[0].heading("Proxies");
            egui::ScrollArea::vertical().show(&mut cols[0], |ui| {
                let mut sel = self.proxy_sel.clone();
                for p in &self.ws.proxies {
                    let label = format!("{}\n{}", p.name, p.display());
                    if ui
                        .selectable_label(sel.as_ref() == Some(&p.id), label)
                        .clicked()
                    {
                        sel = Some(p.id.clone());
                    }
                    ui.separator();
                }
                self.proxy_sel = sel;
            });
            cols[1].heading(if self.form_mode == FormMode::Add {
                "Add proxy"
            } else if self.form_mode == FormMode::Edit {
                "Edit proxy"
            } else {
                "Details"
            });
            if self.form_mode != FormMode::None {
                self.proxy_form_ui(&mut cols[1]);
            } else if let Some(id) = &self.proxy_sel {
                if let Some(p) = self.ws.proxies.iter().find(|p| &p.id == id).cloned() {
                    cols[1].label(format!("Name: {}", p.name));
                    cols[1].label(format!("Kind: {}", p.kind));
                    cols[1].label(format!("Endpoint: {}", p.display()));
                    cols[1].label(format!("User: {}", p.user));
                }
            }
        });
    }
    fn proxy_form_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Name");
        ui.text_edit_singleline(&mut self.proxy_form.name);
        ui.label("Kind");
        egui::ComboBox::from_id_salt("proxy_kind")
            .selected_text(&self.proxy_form.kind)
            .show_ui(ui, |ui| {
                for k in ["jump", "socks5", "http"] {
                    ui.selectable_value(&mut self.proxy_form.kind, k.to_string(), k);
                }
            });
        ui.label("Host");
        ui.text_edit_singleline(&mut self.proxy_form.host);
        ui.label("Port");
        ui.add(egui::DragValue::new(&mut self.proxy_form.port).range(1..=65535));
        ui.label("User");
        ui.text_edit_singleline(&mut self.proxy_form.user);
        ui.label("Password");
        ui.add(egui::TextEdit::singleline(&mut self.proxy_form.password).password(true));
        ui.horizontal(|ui| {
            if ui.button(self.tr("save")).clicked() {
                if self.proxy_form.name.is_empty() || self.proxy_form.host.is_empty() {
                    self.status = "Proxy name/host required".into();
                } else {
                    if self.proxy_form.id.is_empty() {
                        self.proxy_form.id = gen_id();
                    }
                    if self.form_mode == FormMode::Edit {
                        if let Some(p) = self
                            .ws
                            .proxies
                            .iter_mut()
                            .find(|p| p.id == self.proxy_form.id)
                        {
                            *p = self.proxy_form.clone();
                        }
                    } else {
                        self.ws.proxies.push(self.proxy_form.clone());
                    }
                    self.proxy_sel = Some(self.proxy_form.id.clone());
                    self.form_mode = FormMode::None;
                    self.save();
                    self.status = "Proxy saved".into();
                }
            }
            if ui.button(self.tr("cancel")).clicked() {
                self.form_mode = FormMode::None;
            }
        });
    }

    fn ui_aliases(&mut self, ui: &mut egui::Ui) {
        self.profile_bar(ui);
        ui.horizontal(|ui| {
            if ui.button(self.tr("add")).clicked() {
                self.form_mode = FormMode::Add;
                self.alias_form = CommandAlias::default();
            }
            if ui.button(self.tr("edit")).clicked() {
                if let Some(id) = &self.alias_sel {
                    if let Some(a) = self.ws.aliases.iter().find(|a| &a.id == id).cloned() {
                        self.alias_form = a;
                        self.form_mode = FormMode::Edit;
                    }
                }
            }
            if ui.button(self.tr("delete")).clicked() {
                if let Some(id) = self.alias_sel.clone() {
                    self.ws.aliases.retain(|a| a.id != id);
                    self.alias_sel = None;
                    self.save();
                }
            }
            if ui.button(self.tr("run")).clicked() {
                if let Some(id) = &self.alias_sel {
                    if let Some(a) = self.ws.aliases.iter().find(|a| &a.id == id).cloned() {
                        let shell = self.ws.config.default_shell.clone();
                        self.spawn_shell("command", format!("alias:{}", a.name), shell, a.command);
                    }
                }
            }
        });
        ui.separator();
        ui.columns(2, |cols| {
            cols[0].heading("Aliases");
            egui::ScrollArea::vertical().show(&mut cols[0], |ui| {
                let mut sel = self.alias_sel.clone();
                for a in &self.ws.aliases {
                    let label = format!("{}\n{}", a.name, util::short(&a.command, 50));
                    if ui
                        .selectable_label(sel.as_ref() == Some(&a.id), label)
                        .clicked()
                    {
                        sel = Some(a.id.clone());
                    }
                    ui.separator();
                }
                self.alias_sel = sel;
            });
            cols[1].heading(if self.form_mode == FormMode::Add {
                "Add alias"
            } else if self.form_mode == FormMode::Edit {
                "Edit alias"
            } else {
                "Details"
            });
            if self.form_mode != FormMode::None {
                self.alias_form_ui(&mut cols[1]);
            } else if let Some(id) = &self.alias_sel {
                if let Some(a) = self.ws.aliases.iter().find(|a| &a.id == id).cloned() {
                    cols[1].label(format!("Name: {}", a.name));
                    cols[1].label(format!("Command: {}", a.command));
                    cols[1].label(format!("Description: {}", a.description));
                }
            }
        });
    }
    fn alias_form_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Name");
        ui.text_edit_singleline(&mut self.alias_form.name);
        ui.label("Command");
        ui.text_edit_singleline(&mut self.alias_form.command);
        ui.label("Description");
        ui.text_edit_singleline(&mut self.alias_form.description);
        ui.horizontal(|ui| {
            if ui.button(self.tr("save")).clicked() {
                if self.alias_form.name.is_empty() || self.alias_form.command.is_empty() {
                    self.status = "Alias name/command required".into();
                } else {
                    if self.alias_form.id.is_empty() {
                        self.alias_form.id = gen_id();
                    }
                    if self.form_mode == FormMode::Edit {
                        if let Some(a) = self
                            .ws
                            .aliases
                            .iter_mut()
                            .find(|a| a.id == self.alias_form.id)
                        {
                            *a = self.alias_form.clone();
                        }
                    } else {
                        self.ws.aliases.push(self.alias_form.clone());
                    }
                    self.alias_sel = Some(self.alias_form.id.clone());
                    self.form_mode = FormMode::None;
                    self.save();
                    self.status = "Alias saved".into();
                }
            }
            if ui.button(self.tr("cancel")).clicked() {
                self.form_mode = FormMode::None;
            }
        });
    }

    fn profile_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(format!("{}:", self.tr("active_profile")));
            let mut active = self.active_profile_id();
            let active_name = self
                .ws
                .ssh_profiles
                .iter()
                .find(|p| p.id == active)
                .map(|p| p.name.as_str())
                .unwrap_or(self.tr("no_profile"))
                .to_string();
            let profiles: Vec<(String, String)> = self.ws.ssh_profiles.iter().map(|p| (p.id.clone(), p.name.clone())).collect();
            let mut picked: Option<String> = None;
            egui::ComboBox::from_id_salt(format!("profile_{:?}", self.tab))
                .selected_text(active_name)
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut active, String::new(), self.tr("no_profile")).changed() {
                        picked = Some(active.clone());
                    }
                    for (id, name) in &profiles {
                        if ui.selectable_value(&mut active, id.clone(), name).changed() {
                            picked = Some(active.clone());
                        }
                    }
                });
            if let Some(id) = picked { self.set_active_profile(id); }
        });
        ui.separator();
    }

    fn ui_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading(self.tr("settings"));
        self.profile_bar(ui);
        ui.horizontal(|ui| {
            ui.label(self.tr("language"));
            let mut lang = self.lang();
            egui::ComboBox::from_id_salt("cfg_lang")
                .selected_text(lang.label())
                .show_ui(ui, |ui| {
                    for l in i18n::Lang::ALL {
                        if ui.selectable_value(&mut lang, l, l.label()).changed() {
                            self.ws.config.language = lang.code().into();
                            self.save();
                        }
                    }
                });
        });
        ui.horizontal(|ui| {
            ui.label("Default shell");
            egui::ComboBox::from_id_salt("cfg_shell")
                .selected_text(&self.ws.config.default_shell)
                .show_ui(ui, |ui| {
                    for s in ["powershell", "bash", "cmd", "sh"] {
                        ui.selectable_value(&mut self.ws.config.default_shell, s.to_string(), s);
                    }
                });
        });
        ui.horizontal(|ui| {
            ui.label("Log limit");
            ui.add(egui::DragValue::new(&mut self.ws.config.log_limit).range(50..=5000));
        });
        ui.horizontal(|ui| {
            ui.label("Theme");
            ui.text_edit_singleline(&mut self.ws.config.theme);
        });
        ui.separator();
        ui.label(format!("Workspace: {}", Workspace::data_file().map(|p|p.display().to_string()).unwrap_or_default()));
        ui.label(format!("Scripts dir: {}", Workspace::scripts_dir().map(|p|p.display().to_string()).unwrap_or_default()));
        if ui.button(self.tr("save_settings")).clicked() {
            self.save();
            self.status = self.tr("settings_saved").into();
        }
    }
}


