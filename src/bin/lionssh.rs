use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use lionssh::executor::{run_shell, script_shell};
use lionssh::ssh::build_ssh_command_line;
use lionssh::*;
use std::time::Duration;

#[derive(Parser)]
#[command(
    name = "lionssh",
    version,
    about = "LionSSH CLI - SSH, scripts, proxies, scheduler, aliases"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Run {
        command: Vec<String>,
    },
    Ssh {
        #[command(subcommand)]
        cmd: SshCmd,
    },
    Script {
        #[command(subcommand)]
        cmd: ScriptCmd,
    },
    Task {
        #[command(subcommand)]
        cmd: TaskCmd,
    },
    Proxy {
        #[command(subcommand)]
        cmd: ProxyCmd,
    },
    Alias {
        #[command(subcommand)]
        cmd: AliasCmd,
    },
    History {
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    Config {
        #[command(subcommand)]
        cmd: Option<ConfigCmd>,
    },
    Paths,
}

#[derive(Subcommand)]
enum SshCmd {
    List,
    Connect {
        name: String,
    },
    Show {
        name: String,
    },
    Add {
        name: String,
        host: String,
        #[arg(long, default_value = "")]
        user: String,
        #[arg(long, default_value_t = 22)]
        port: u16,
        #[arg(long, default_value = "")]
        identity: String,
        #[arg(long, default_value = "")]
        proxy: String,
        #[arg(long, default_value = "")]
        startup: String,
    },
    Rm {
        name: String,
    },
}
#[derive(Subcommand)]
enum ScriptCmd {
    List,
    Show {
        name: String,
    },
    Run {
        name: String,
    },
    Add {
        name: String,
        #[arg(long, default_value = "shell")]
        kind: String,
        #[arg(long, default_value = "")]
        desc: String,
        #[arg(long, default_value = "")]
        profile: String,
        #[arg(long)]
        file: Option<String>,
        #[arg(long)]
        content: Option<String>,
    },
    Rm {
        name: String,
    },
    Export {
        name: String,
    },
}
#[derive(Subcommand)]
enum TaskCmd {
    List,
    Add {
        name: String,
        #[arg(long, default_value = "command")]
        kind: String,
        #[arg(long)]
        target: String,
        #[arg(long, default_value_t = 60)]
        interval: u64,
    },
    Run {
        name: String,
    },
    Toggle {
        name: String,
    },
    Rm {
        name: String,
    },
}
#[derive(Subcommand)]
enum ProxyCmd {
    List,
    Add {
        name: String,
        #[arg(long, default_value = "socks5")]
        kind: String,
        #[arg(long)]
        host: String,
        #[arg(long, default_value_t = 1080)]
        port: u16,
        #[arg(long, default_value = "")]
        user: String,
        #[arg(long, default_value = "")]
        password: String,
    },
    Rm {
        name: String,
    },
}
#[derive(Subcommand)]
enum AliasCmd {
    List,
    Add {
        name: String,
        command: Vec<String>,
        #[arg(long, default_value = "")]
        desc: String,
    },
    Run {
        name: String,
    },
    Rm {
        name: String,
    },
}
#[derive(Subcommand)]
enum ConfigCmd {
    SetShell { shell: String },
    SetLogLimit { limit: usize },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut ws = storage::load()?;
    match cli.cmd {
        Cmd::Paths => {
            println!("data_file: {}", Workspace::data_file()?.display());
            println!("scripts_dir: {}", Workspace::scripts_dir()?.display());
        }
        Cmd::Run { command } => {
            let cmd = command.join(" ");
            let resolved = ws
                .aliases
                .iter()
                .find(|a| a.name == cmd)
                .map(|a| a.command.clone())
                .unwrap_or(cmd.clone());
            let res = run_shell(
                &ws.config.default_shell,
                &resolved,
                Some(Duration::from_secs(300)),
            )?;
            print_run(&resolved, &res);
            push_hist(
                &mut ws,
                "command",
                resolved,
                if res.ok { "ok" } else { "error" },
                res.output,
            );
        }
        Cmd::History { limit } => {
            for h in ws.history.iter().rev().take(limit) {
                println!(
                    "{} [{:<9}] {:<5} {} :: {}",
                    h.at.format("%Y-%m-%d %H:%M:%S"),
                    h.kind,
                    h.status,
                    h.title,
                    util::short(&h.detail, 120)
                );
            }
        }
        Cmd::Config { cmd } => match cmd {
            None => {
                println!("theme={}", ws.config.theme);
                println!("default_shell={}", ws.config.default_shell);
                println!("log_limit={}", ws.config.log_limit);
            }
            Some(ConfigCmd::SetShell { shell }) => {
                ws.config.default_shell = shell;
                storage::save(&ws)?;
                println!("ok");
            }
            Some(ConfigCmd::SetLogLimit { limit }) => {
                ws.config.log_limit = limit.max(50);
                storage::save(&ws)?;
                println!("ok");
            }
        },
        Cmd::Ssh { cmd } => handle_ssh(&mut ws, cmd)?,
        Cmd::Script { cmd } => handle_script(&mut ws, cmd)?,
        Cmd::Task { cmd } => handle_task(&mut ws, cmd)?,
        Cmd::Proxy { cmd } => handle_proxy(&mut ws, cmd)?,
        Cmd::Alias { cmd } => handle_alias(&mut ws, cmd)?,
    }
    Ok(())
}

fn print_run(title: &str, res: &lionssh::executor::RunResult) {
    println!(
        "> {title}\nstatus={} ok={} {}ms\n{}",
        res.status, res.ok, res.duration_ms, res.output
    );
}
fn push_hist(ws: &mut Workspace, kind: &str, title: String, status: &str, detail: String) {
    util::push_history(ws, HistoryEntry::new(kind, title, status, detail));
    let _ = storage::save(ws);
}
fn find_by_name<'a, T>(items: &'a [T], name: &str, get: impl Fn(&T) -> &str) -> Option<&'a T> {
    items
        .iter()
        .find(|i| get(i).eq_ignore_ascii_case(name) || get(i) == name)
}

fn handle_ssh(ws: &mut Workspace, cmd: SshCmd) -> Result<()> {
    match cmd {
        SshCmd::List => {
            for p in &ws.ssh_profiles {
                let proxy = ws
                    .proxies
                    .iter()
                    .find(|x| x.id == p.proxy_id)
                    .map(|x| x.name.as_str())
                    .unwrap_or("direct");
                println!("{}\t{}\tproxy={}\tid={}", p.name, p.target(), proxy, p.id);
            }
        }
        SshCmd::Show { name } => {
            let p = find_by_name(&ws.ssh_profiles, &name, |p| &p.name)
                .ok_or_else(|| anyhow!("ssh profile not found"))?;
            let proxy = ws.proxies.iter().find(|x| x.id == p.proxy_id);
            println!("{}", build_ssh_command_line(p, proxy, None));
        }
        SshCmd::Connect { name } => {
            let p = find_by_name(&ws.ssh_profiles, &name, |p| &p.name)
                .ok_or_else(|| anyhow!("ssh profile not found"))?
                .clone();
            let proxy = ws.proxies.iter().find(|x| x.id == p.proxy_id).cloned();
            let startup = if p.startup_commands.is_empty() {
                None
            } else {
                Some(p.startup_commands.join(" && "))
            };
            let cmdline = build_ssh_command_line(&p, proxy.as_ref(), startup.as_deref());
            println!("{cmdline}");
            let status = std::process::Command::new("ssh")
                .args(lionssh::ssh::build_ssh_args(
                    &p,
                    proxy.as_ref(),
                    startup.as_deref(),
                ))
                .status();
            match status {
                Ok(s) => {
                    push_hist(
                        ws,
                        "ssh",
                        p.name,
                        if s.success() { "ok" } else { "error" },
                        cmdline,
                    );
                }
                Err(e) => {
                    push_hist(ws, "ssh", p.name, "error", e.to_string());
                    return Err(e.into());
                }
            }
        }
        SshCmd::Add {
            name,
            host,
            user,
            port,
            identity,
            proxy,
            startup,
        } => {
            let proxy_id = if proxy.is_empty() {
                String::new()
            } else {
                ws.proxies
                    .iter()
                    .find(|p| p.name == proxy || p.id == proxy)
                    .map(|p| p.id.clone())
                    .unwrap_or_default()
            };
            let startup_commands = if startup.is_empty() {
                vec![]
            } else {
                startup
                    .split(";;")
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            };
            ws.ssh_profiles.push(SshProfile {
                id: gen_id(),
                name,
                host,
                user,
                port,
                identity,
                proxy_id,
                startup_commands,
                notes: String::new(),
            });
            storage::save(ws)?;
            println!("ok");
        }
        SshCmd::Rm { name } => {
            let before = ws.ssh_profiles.len();
            ws.ssh_profiles.retain(|p| p.name != name && p.id != name);
            if ws.ssh_profiles.len() == before {
                return Err(anyhow!("not found"));
            }
            storage::save(ws)?;
            println!("ok");
        }
    }
    Ok(())
}

fn handle_script(ws: &mut Workspace, cmd: ScriptCmd) -> Result<()> {
    match cmd {
        ScriptCmd::List => {
            for s in &ws.scripts {
                println!(
                    "{}\t{}\t{}\t{}",
                    s.name,
                    s.kind,
                    s.id,
                    util::short(&s.description, 60)
                );
            }
        }
        ScriptCmd::Show { name } => {
            let s = find_by_name(&ws.scripts, &name, |s| &s.name)
                .ok_or_else(|| anyhow!("script not found"))?;
            println!(
                "name: {}\nkind: {}\ndesc: {}\n---\n{}",
                s.name, s.kind, s.description, s.content
            );
        }
        ScriptCmd::Run { name } => {
            let s = find_by_name(&ws.scripts, &name, |s| &s.name)
                .ok_or_else(|| anyhow!("script not found"))?
                .clone();
            if s.kind == "ssh" {
                let p = ws
                    .ssh_profiles
                    .iter()
                    .find(|p| p.id == s.profile_id)
                    .cloned()
                    .ok_or_else(|| anyhow!("ssh profile missing"))?;
                let proxy = ws.proxies.iter().find(|x| x.id == p.proxy_id).cloned();
                let cmdline = build_ssh_command_line(&p, proxy.as_ref(), Some(&s.content));
                let res = run_shell(
                    &ws.config.default_shell,
                    &cmdline,
                    Some(Duration::from_secs(300)),
                )?;
                print_run(&s.name, &res);
                push_hist(
                    ws,
                    "script",
                    s.name,
                    if res.ok { "ok" } else { "error" },
                    res.output,
                );
            } else {
                let shell = script_shell(&s.kind, &ws.config.default_shell).to_string();
                let res = run_shell(&shell, &s.content, Some(Duration::from_secs(300)))?;
                print_run(&s.name, &res);
                push_hist(
                    ws,
                    "script",
                    s.name,
                    if res.ok { "ok" } else { "error" },
                    res.output,
                );
            }
        }
        ScriptCmd::Add {
            name,
            kind,
            desc,
            profile,
            file,
            content,
        } => {
            let body = if let Some(f) = file {
                std::fs::read_to_string(f)?
            } else {
                content.unwrap_or_default()
            };
            let profile_id = if profile.is_empty() {
                String::new()
            } else {
                ws.ssh_profiles
                    .iter()
                    .find(|p| p.name == profile || p.id == profile)
                    .map(|p| p.id.clone())
                    .unwrap_or(profile)
            };
            let mut s = Script::default();
            s.name = name;
            s.kind = kind;
            s.description = desc;
            s.profile_id = profile_id;
            s.content = body;
            ws.scripts.push(s);
            storage::save(ws)?;
            println!("ok");
        }
        ScriptCmd::Rm { name } => {
            let before = ws.scripts.len();
            ws.scripts.retain(|s| s.name != name && s.id != name);
            if ws.scripts.len() == before {
                return Err(anyhow!("not found"));
            }
            storage::save(ws)?;
            println!("ok");
        }
        ScriptCmd::Export { name } => {
            let s = find_by_name(&ws.scripts, &name, |s| &s.name)
                .ok_or_else(|| anyhow!("script not found"))?;
            let dir = Workspace::scripts_dir()?;
            std::fs::create_dir_all(&dir)?;
            let ext = match s.kind.as_str() {
                "powershell" => "ps1",
                "bash" => "sh",
                "cmd" => "cmd",
                _ => "txt",
            };
            let path = dir.join(format!("{}.{}", util::sanitize_filename(&s.name), ext));
            std::fs::write(&path, &s.content)?;
            println!("{}", path.display());
        }
    }
    Ok(())
}

fn handle_task(ws: &mut Workspace, cmd: TaskCmd) -> Result<()> {
    match cmd {
        TaskCmd::List => {
            for t in &ws.tasks {
                println!(
                    "{}\t{}\t{}\tinterval={}s\truns={}\tenabled={}\tid={}",
                    t.name,
                    t.target_kind,
                    util::short(&t.target, 40),
                    t.interval_secs,
                    t.run_count,
                    t.enabled,
                    t.id
                );
            }
        }
        TaskCmd::Add {
            name,
            kind,
            target,
            interval,
        } => {
            let mut t = ScheduledTask::default();
            t.name = name;
            t.target_kind = kind;
            t.target = target;
            t.interval_secs = interval;
            ws.tasks.push(t);
            storage::save(ws)?;
            println!("ok");
        }
        TaskCmd::Run { name } => {
            let t = find_by_name(&ws.tasks, &name, |t| &t.name)
                .ok_or_else(|| anyhow!("task not found"))?
                .clone();
            if t.target_kind == "script" {
                handle_script(ws, ScriptCmd::Run { name: t.target })?;
            } else {
                let res = run_shell(
                    &ws.config.default_shell,
                    &t.target,
                    Some(Duration::from_secs(300)),
                )?;
                print_run(&t.name, &res);
                push_hist(
                    ws,
                    "scheduler",
                    t.name,
                    if res.ok { "ok" } else { "error" },
                    res.output,
                );
            }
        }
        TaskCmd::Toggle { name } => {
            let t = ws
                .tasks
                .iter_mut()
                .find(|t| t.name == name || t.id == name)
                .ok_or_else(|| anyhow!("not found"))?;
            t.enabled = !t.enabled;
            println!("{}", t.enabled);
            storage::save(ws)?;
        }
        TaskCmd::Rm { name } => {
            let before = ws.tasks.len();
            ws.tasks.retain(|t| t.name != name && t.id != name);
            if ws.tasks.len() == before {
                return Err(anyhow!("not found"));
            }
            storage::save(ws)?;
            println!("ok");
        }
    }
    Ok(())
}

fn handle_proxy(ws: &mut Workspace, cmd: ProxyCmd) -> Result<()> {
    match cmd {
        ProxyCmd::List => {
            for p in &ws.proxies {
                println!("{}\t{}\tid={}", p.name, p.display(), p.id);
            }
        }
        ProxyCmd::Add {
            name,
            kind,
            host,
            port,
            user,
            password,
        } => {
            ws.proxies.push(ProxyProfile {
                id: gen_id(),
                name,
                kind,
                host,
                port,
                user,
                password,
            });
            storage::save(ws)?;
            println!("ok");
        }
        ProxyCmd::Rm { name } => {
            let before = ws.proxies.len();
            ws.proxies.retain(|p| p.name != name && p.id != name);
            if ws.proxies.len() == before {
                return Err(anyhow!("not found"));
            }
            storage::save(ws)?;
            println!("ok");
        }
    }
    Ok(())
}

fn handle_alias(ws: &mut Workspace, cmd: AliasCmd) -> Result<()> {
    match cmd {
        AliasCmd::List => {
            for a in &ws.aliases {
                println!("{}\t{}\t{}", a.name, a.command, a.description);
            }
        }
        AliasCmd::Add {
            name,
            command,
            desc,
        } => {
            ws.aliases.push(CommandAlias {
                id: gen_id(),
                name,
                command: command.join(" "),
                description: desc,
            });
            storage::save(ws)?;
            println!("ok");
        }
        AliasCmd::Run { name } => {
            let a = find_by_name(&ws.aliases, &name, |a| &a.name)
                .ok_or_else(|| anyhow!("alias not found"))?
                .clone();
            let res = run_shell(
                &ws.config.default_shell,
                &a.command,
                Some(Duration::from_secs(300)),
            )?;
            print_run(&a.name, &res);
            push_hist(
                ws,
                "command",
                a.name,
                if res.ok { "ok" } else { "error" },
                res.output,
            );
        }
        AliasCmd::Rm { name } => {
            let before = ws.aliases.len();
            ws.aliases.retain(|a| a.name != name && a.id != name);
            if ws.aliases.len() == before {
                return Err(anyhow!("not found"));
            }
            storage::save(ws)?;
            println!("ok");
        }
    }
    Ok(())
}
