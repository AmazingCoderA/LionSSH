use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap};
use ratatui::Frame;

use crate::app::{App, InputKind, Mode, Tab};
use crate::models::Workspace;

fn rgb(c: [u8;3]) -> Color { Color::Rgb(c[0], c[1], c[2]) }

pub fn draw(f: &mut Frame, app: &App) {
    let t = &app.theme;
    let area = f.area();
    let bg = Block::default().style(Style::default().bg(rgb(t.bg)).fg(rgb(t.text)));
    f.render_widget(bg, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5), Constraint::Length(3)])
        .split(area);

    draw_header(f, app, chunks[0]);
    match app.tab {
        Tab::Dashboard => draw_dashboard(f, app, chunks[1]),
        Tab::Terminal => draw_terminal(f, app, chunks[1]),
        Tab::Ssh => draw_ssh(f, app, chunks[1]),
        Tab::Scripts => draw_scripts(f, app, chunks[1]),
        Tab::Scheduler => draw_scheduler(f, app, chunks[1]),
        Tab::Proxies => draw_proxies(f, app, chunks[1]),
        Tab::Aliases => draw_aliases(f, app, chunks[1]),
        Tab::Settings => draw_settings(f, app, chunks[1]),
    }
    draw_footer(f, app, chunks[2]);

    match &app.mode {
        Mode::Input(kind) => draw_input(f, app, kind),
        Mode::Help => draw_help(f, app),
        Mode::Confirm(_) => draw_confirm(f, app),
        Mode::Normal => {}
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let titles: Vec<Line> = Tab::ALL.iter().map(|tab| {
        let style = if *tab == app.tab { Style::default().fg(rgb(t.bg)).bg(rgb(t.primary)).add_modifier(Modifier::BOLD) } else { Style::default().fg(rgb(t.text)) };
        Line::from(Span::styled(format!(" {} ", tab.title()), style))
    }).collect();
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(rgb(t.primary))).title(Span::styled(" LionSSH MEGA COMMAND CENTER ", Style::default().fg(rgb(t.accent)).add_modifier(Modifier::BOLD))))
        .highlight_style(Style::default().fg(rgb(t.accent)));
    f.render_widget(tabs, area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let help = match app.tab {
        Tab::Dashboard | Tab::Terminal => "c:command  l:clear  Up/Down:scroll  Tab:switch  ?:help  q:quit",
        Tab::Ssh => "a:add  e:edit  d:del  Enter/c:connect  t:show-cmd  Tab:switch",
        Tab::Scripts => "a:add  e:edit  d:del  Enter/r:run  x:export  Tab:switch",
        Tab::Scheduler => "a:add  e:edit  d:del  Space:pause/resume  Enter/r:run-now  Tab:switch",
        Tab::Proxies => "a:add  e:edit  d:del  Tab:switch",
        Tab::Aliases => "a:add  e:edit  d:del  Enter/r:run  Tab:switch",
        Tab::Settings => "t:theme  e:effects  m:effect-mode  s:shell  l:log-limit  Tab:switch",
    };
    let busy = if app.busy > 0 { format!(" | running:{}", app.busy) } else { String::new() };
    let p = Paragraph::new(vec![
        Line::from(vec![Span::styled(help, Style::default().fg(rgb(t.dim))), Span::styled(busy, Style::default().fg(rgb(t.warning)))]),
        Line::from(Span::styled(format!(" {} ", app.status), Style::default().fg(rgb(t.secondary)))),
    ]).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(rgb(t.secondary))));
    f.render_widget(p, area);
}
fn draw_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let cols = Layout::default().direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)]).split(area);
    let left = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(10), Constraint::Min(5)]).split(cols[0]);

    let stats = vec![
        Line::from(vec![Span::styled("SSH profiles: ", Style::default().fg(rgb(t.dim))), Span::styled(app.ws.ssh_profiles.len().to_string(), Style::default().fg(rgb(t.primary)).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::styled("Scripts:      ", Style::default().fg(rgb(t.dim))), Span::styled(app.ws.scripts.len().to_string(), Style::default().fg(rgb(t.primary)).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::styled("Tasks:        ", Style::default().fg(rgb(t.dim))), Span::styled(app.ws.tasks.len().to_string(), Style::default().fg(rgb(t.primary)).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::styled("Proxies:      ", Style::default().fg(rgb(t.dim))), Span::styled(app.ws.proxies.len().to_string(), Style::default().fg(rgb(t.primary)).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::styled("Aliases:      ", Style::default().fg(rgb(t.dim))), Span::styled(app.ws.aliases.len().to_string(), Style::default().fg(rgb(t.primary)).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::styled("Theme:        ", Style::default().fg(rgb(t.dim))), Span::styled(&app.ws.config.theme, Style::default().fg(rgb(t.accent)))]),
        Line::from(vec![Span::styled("Shell:        ", Style::default().fg(rgb(t.dim))), Span::styled(&app.ws.config.default_shell, Style::default().fg(rgb(t.accent)))]),
        Line::from(vec![Span::styled("Effects:      ", Style::default().fg(rgb(t.dim))), Span::styled(format!("{} / {}", if app.ws.config.effects {"on"} else {"off"}, app.ws.config.effect), Style::default().fg(rgb(t.accent)))]),
    ];
    f.render_widget(Paragraph::new(stats).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(rgb(t.primary))).title(" Status ")), left[0]);

    let recent: Vec<ListItem> = app.ws.history.iter().rev().take(12).map(|h| {
        let c = match h.status.as_str() { "ok" => rgb(t.success), "error" => rgb(t.error), "info" => rgb(t.accent), _ => rgb(t.warning) };
        ListItem::new(Line::from(vec![
            Span::styled(h.at.format("%H:%M:%S ").to_string(), Style::default().fg(rgb(t.dim))),
            Span::styled(format!("{:<9}", h.kind), Style::default().fg(rgb(t.secondary))),
            Span::styled(format!("{:<6}", h.status), Style::default().fg(c)),
            Span::styled(h.title.clone(), Style::default().fg(rgb(t.text))),
        ]))
    }).collect();
    f.render_widget(List::new(recent).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(rgb(t.secondary))).title(" Recent Activity ")), left[1]);

    draw_terminal(f, app, cols[1]);
}

fn draw_terminal(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let h = area.height.saturating_sub(2) as usize;
    let total = app.terminal_output.len();
    let start = total.saturating_sub(h + app.terminal_scroll);
    let end = total.saturating_sub(app.terminal_scroll);
    let lines: Vec<Line> = app.terminal_output.iter().skip(start).take(end.saturating_sub(start)).map(|l| {
        let style = if l.starts_with(">>>") { Style::default().fg(rgb(t.accent)).add_modifier(Modifier::BOLD) }
            else if l.starts_with("---") { Style::default().fg(rgb(t.secondary)) }
            else if l.to_lowercase().contains("error") { Style::default().fg(rgb(t.error)) }
            else { Style::default().fg(rgb(t.text)) };
        Line::from(Span::styled(l.clone(), style))
    }).collect();
    let title = format!(" Terminal / Output  ({} lines) ", total);
    f.render_widget(Paragraph::new(lines).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(rgb(t.primary))).title(title)).wrap(Wrap { trim: false }), area);
}
fn draw_ssh(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let cols = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(45), Constraint::Percentage(55)]).split(area);
    let items: Vec<ListItem> = app.ws.ssh_profiles.iter().enumerate().map(|(i, p)| {
        let sel = i == app.selected_ssh;
        let style = if sel { Style::default().fg(rgb(t.bg)).bg(rgb(t.primary)).add_modifier(Modifier::BOLD) } else { Style::default().fg(rgb(t.text)) };
        let proxy = app.ws.proxies.iter().find(|x| x.id == p.proxy_id).map(|x| x.name.as_str()).unwrap_or("direct");
        ListItem::new(Line::from(vec![
            Span::styled(format!("{:<18}", p.name), style),
            Span::styled(format!(" {} ", p.target()), if sel { style } else { Style::default().fg(rgb(t.secondary)) }),
            Span::styled(format!(" [{}]", proxy), if sel { style } else { Style::default().fg(rgb(t.dim)) }),
        ]))
    }).collect();
    f.render_widget(List::new(items).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(rgb(t.primary))).title(" SSH Profiles ")), cols[0]);

    let detail = if let Some(p) = app.ws.ssh_profiles.get(app.selected_ssh) {
        let proxy = app.ws.proxies.iter().find(|x| x.id == p.proxy_id).map(|x| x.display()).unwrap_or_else(|| "direct".into());
        vec![
            Line::from(vec![Span::styled("Name: ", Style::default().fg(rgb(t.dim))), Span::styled(&p.name, Style::default().fg(rgb(t.accent)))]),
            Line::from(vec![Span::styled("Target: ", Style::default().fg(rgb(t.dim))), Span::styled(p.target(), Style::default().fg(rgb(t.text)))]),
            Line::from(vec![Span::styled("Identity: ", Style::default().fg(rgb(t.dim))), Span::styled(if p.identity.is_empty(){"<default>"} else {&p.identity}, Style::default().fg(rgb(t.text)))]),
            Line::from(vec![Span::styled("Proxy: ", Style::default().fg(rgb(t.dim))), Span::styled(proxy, Style::default().fg(rgb(t.text)))]),
            Line::from(vec![Span::styled("Startup: ", Style::default().fg(rgb(t.dim))), Span::styled(if p.startup_commands.is_empty(){"<none>".into()} else {p.startup_commands.join(" && ")}, Style::default().fg(rgb(t.text)))]),
            Line::from(vec![Span::styled("Notes: ", Style::default().fg(rgb(t.dim))), Span::styled(&p.notes, Style::default().fg(rgb(t.text)))]),
            Line::from(""),
            Line::from(Span::styled("Enter/c connect, t show command, a add, e edit, d delete", Style::default().fg(rgb(t.dim)))),
        ]
    } else { vec![Line::from(Span::styled("No SSH profiles. Press 'a' to add.", Style::default().fg(rgb(t.dim))))] };
    f.render_widget(Paragraph::new(detail).wrap(Wrap { trim: false }).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(rgb(t.secondary))).title(" Details ")), cols[1]);
}

fn draw_scripts(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let cols = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(40), Constraint::Percentage(60)]).split(area);
    let items: Vec<ListItem> = app.ws.scripts.iter().enumerate().map(|(i, s)| {
        let sel = i == app.selected_script;
        let style = if sel { Style::default().fg(rgb(t.bg)).bg(rgb(t.primary)).add_modifier(Modifier::BOLD) } else { Style::default().fg(rgb(t.text)) };
        ListItem::new(Line::from(vec![
            Span::styled(format!("{:<20}", s.name), style),
            Span::styled(format!(" [{}]", s.kind), if sel { style } else { Style::default().fg(rgb(t.secondary)) }),
        ]))
    }).collect();
    f.render_widget(List::new(items).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(rgb(t.primary))).title(" Scripts ")), cols[0]);

    let detail = if let Some(s) = app.ws.scripts.get(app.selected_script) {
        let mut lines = vec![
            Line::from(vec![Span::styled("Name: ", Style::default().fg(rgb(t.dim))), Span::styled(&s.name, Style::default().fg(rgb(t.accent)))]),
            Line::from(vec![Span::styled("Kind: ", Style::default().fg(rgb(t.dim))), Span::styled(&s.kind, Style::default().fg(rgb(t.text)))]),
            Line::from(vec![Span::styled("Desc: ", Style::default().fg(rgb(t.dim))), Span::styled(&s.description, Style::default().fg(rgb(t.text)))]),
            Line::from(vec![Span::styled("Tags: ", Style::default().fg(rgb(t.dim))), Span::styled(if s.tags.is_empty(){"-".into()} else {s.tags.join(", ")}, Style::default().fg(rgb(t.text)))]),
            Line::from(Span::styled("Content:", Style::default().fg(rgb(t.dim)))),
        ];
        for l in s.content.lines().take(20) { lines.push(Line::from(Span::styled(format!("  {l}"), Style::default().fg(rgb(t.text))))); }
        lines
    } else { vec![Line::from(Span::styled("No scripts. Press 'a' to add. For multiline, use ;; in input.", Style::default().fg(rgb(t.dim))))] };
    f.render_widget(Paragraph::new(detail).wrap(Wrap { trim: false }).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(rgb(t.secondary))).title(" Script Details ")), cols[1]);
}
fn draw_scheduler(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let items: Vec<ListItem> = app.ws.tasks.iter().enumerate().map(|(i, task)| {
        let sel = i == app.selected_task;
        let base = if sel { Style::default().fg(rgb(t.bg)).bg(rgb(t.primary)).add_modifier(Modifier::BOLD) } else { Style::default().fg(rgb(t.text)) };
        let status_c = if task.enabled { rgb(t.success) } else { rgb(t.warning) };
        let last = task.last_run.map(|d| d.format("%H:%M:%S").to_string()).unwrap_or_else(|| "-".into());
        ListItem::new(Line::from(vec![
            Span::styled(format!("{:<20}", task.name), base),
            Span::styled(format!(" {:<7}", if task.enabled{"ON"} else {"OFF"}), if sel { base } else { Style::default().fg(status_c) }),
            Span::styled(format!(" {}s ", task.interval_secs), if sel { base } else { Style::default().fg(rgb(t.secondary)) }),
            Span::styled(format!(" {}:{} ", task.target_kind, crate::util::short(&task.target, 30)), if sel { base } else { Style::default().fg(rgb(t.dim)) }),
            Span::styled(format!(" last:{} runs:{} {}", last, task.run_count, task.last_status), if sel { base } else { Style::default().fg(rgb(t.dim)) }),
        ]))
    }).collect();
    f.render_widget(List::new(items).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(rgb(t.primary))).title(" Scheduler (Space toggle, Enter run now) ")), area);
}

fn draw_proxies(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let items: Vec<ListItem> = app.ws.proxies.iter().enumerate().map(|(i, p)| {
        let sel = i == app.selected_proxy;
        let style = if sel { Style::default().fg(rgb(t.bg)).bg(rgb(t.primary)).add_modifier(Modifier::BOLD) } else { Style::default().fg(rgb(t.text)) };
        ListItem::new(Line::from(vec![
            Span::styled(format!("{:<18}", p.name), style),
            Span::styled(format!(" {} ", p.display()), if sel { style } else { Style::default().fg(rgb(t.secondary)) }),
            Span::styled(format!(" id:{}", p.id), if sel { style } else { Style::default().fg(rgb(t.dim)) }),
        ]))
    }).collect();
    f.render_widget(List::new(items).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(rgb(t.primary))).title(" Proxies (a add, e edit, d delete) ")), area);
}

fn draw_aliases(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let items: Vec<ListItem> = app.ws.aliases.iter().enumerate().map(|(i, a)| {
        let sel = i == app.selected_alias;
        let style = if sel { Style::default().fg(rgb(t.bg)).bg(rgb(t.primary)).add_modifier(Modifier::BOLD) } else { Style::default().fg(rgb(t.text)) };
        ListItem::new(Line::from(vec![
            Span::styled(format!("{:<16}", a.name), style),
            Span::styled(format!(" {} ", crate::util::short(&a.command, 50)), if sel { style } else { Style::default().fg(rgb(t.secondary)) }),
            Span::styled(format!(" {}", a.description), if sel { style } else { Style::default().fg(rgb(t.dim)) }),
        ]))
    }).collect();
    f.render_widget(List::new(items).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(rgb(t.primary))).title(" Aliases (Enter run) ")), area);
}

fn draw_settings(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let lines = vec![
        Line::from(vec![Span::styled("Theme: ", Style::default().fg(rgb(t.dim))), Span::styled(&app.ws.config.theme, Style::default().fg(rgb(t.accent)).add_modifier(Modifier::BOLD)), Span::styled("  (t to change)", Style::default().fg(rgb(t.dim)))]),
        Line::from(vec![Span::styled("Effects: ", Style::default().fg(rgb(t.dim))), Span::styled(if app.ws.config.effects {"on"} else {"off"}, Style::default().fg(rgb(t.accent))), Span::styled("  (e toggle)", Style::default().fg(rgb(t.dim)))]),
        Line::from(vec![Span::styled("Effect mode: ", Style::default().fg(rgb(t.dim))), Span::styled(&app.ws.config.effect, Style::default().fg(rgb(t.accent))), Span::styled("  (m change: pulse/matrix/rain/none)", Style::default().fg(rgb(t.dim)))]),
        Line::from(vec![Span::styled("Default shell: ", Style::default().fg(rgb(t.dim))), Span::styled(&app.ws.config.default_shell, Style::default().fg(rgb(t.accent))), Span::styled("  (s change)", Style::default().fg(rgb(t.dim)))]),
        Line::from(vec![Span::styled("Log limit: ", Style::default().fg(rgb(t.dim))), Span::styled(app.ws.config.log_limit.to_string(), Style::default().fg(rgb(t.accent))), Span::styled("  (l change)", Style::default().fg(rgb(t.dim)))]),
        Line::from(""),
        Line::from(Span::styled("Themes: cyberpunk, dracula, matrix, nord, solarized, aurora, blood, vaporwave", Style::default().fg(rgb(t.secondary)))),
        Line::from(Span::styled(format!("Data file: {}", Workspace::data_file().unwrap_or_default().display()), Style::default().fg(rgb(t.dim)))),
        Line::from(Span::styled(format!("Scripts dir: {}", Workspace::scripts_dir().unwrap_or_default().display()), Style::default().fg(rgb(t.dim)))),
    ];
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(rgb(t.primary))).title(" Settings ")), area);
}
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default().direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ]).split(r);
    Layout::default().direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ]).split(popup_layout[1])[1]
}

fn draw_input(f: &mut Frame, app: &App, _kind: &InputKind) {
    let t = &app.theme;
    let area = centered_rect(70, 20, f.area());
    f.render_widget(Clear, area);
    let block = Block::default().borders(Borders::ALL).border_style(Style::default().fg(rgb(t.accent))).title(" Input ");
    let text = vec![
        Line::from(Span::styled(&app.input_prompt, Style::default().fg(rgb(t.secondary)))),
        Line::from(Span::styled(format!("{}█", app.input), Style::default().fg(rgb(t.text)).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled("Enter to confirm, Esc to cancel", Style::default().fg(rgb(t.dim)))),
    ];
    f.render_widget(Paragraph::new(text).block(block).wrap(Wrap { trim: false }), area);
}

fn draw_help(f: &mut Frame, app: &App) {
    let t = &app.theme;
    let area = centered_rect(80, 70, f.area());
    f.render_widget(Clear, area);
    let lines = vec![
        Line::from(Span::styled("LionSSH - hotkeys", Style::default().fg(rgb(t.accent)).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("Global: Tab/BackTab switch tabs, 1-8 jump tabs, Ctrl+S save, ? help, q quit"),
        Line::from("Terminal: c run command/alias, l clear, Up/Down scroll output"),
        Line::from("SSH: a add, e edit, d delete, Enter connect, t print ssh command"),
        Line::from("Scripts: a add, e edit, d delete, Enter run, x export file"),
        Line::from("Scheduler: a add, e edit, d delete, Space enable/disable, Enter run now"),
        Line::from("Proxies: a add, e edit, d delete"),
        Line::from("Aliases: a add, e edit, d delete, Enter run"),
        Line::from("Settings: t theme, e effects toggle, m effect mode, s default shell, l log limit"),
        Line::from(""),
        Line::from(Span::styled("Script/command notes:", Style::default().fg(rgb(t.secondary)))),
        Line::from("- Use ;; in form inputs to insert newlines in script/content fields."),
        Line::from("- SSH script kind runs script content as remote command over selected SSH profile."),
        Line::from("- Scheduler target kind: command (shell command/alias) or script (script id)."),
        Line::from("- SSH proxy kinds: jump (ProxyJump), socks5/http (ProxyCommand via nc)."),
        Line::from(""),
        Line::from(Span::styled("Press any key to close", Style::default().fg(rgb(t.dim)))),
    ];
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(rgb(t.primary))).title(" Help ")), area);
}

fn draw_confirm(f: &mut Frame, app: &App) {
    let t = &app.theme;
    let area = centered_rect(50, 20, f.area());
    f.render_widget(Clear, area);
    let text = vec![
        Line::from(Span::styled("Confirm delete?", Style::default().fg(rgb(t.warning)).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled("Press Y/Enter to delete, any other key to cancel.", Style::default().fg(rgb(t.text)))),
    ];
    f.render_widget(Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(rgb(t.error))).title(" Confirm ")), area);
}
