use crate::models::{ProxyProfile, SshProfile};

pub fn build_ssh_args(
    profile: &SshProfile,
    proxy: Option<&ProxyProfile>,
    remote_command: Option<&str>,
) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();
    let port = if profile.port == 0 { 22 } else { profile.port };
    args.push("-p".into());
    args.push(port.to_string());
    if !profile.identity.trim().is_empty() {
        args.push("-i".into());
        args.push(profile.identity.trim().to_string());
    }
    args.push("-o".into());
    args.push("StrictHostKeyChecking=accept-new".into());
    if let Some(p) = proxy {
        match p.kind.as_str() {
            "jump" => {
                let j = jump_spec(p);
                if !j.is_empty() {
                    args.push("-J".into());
                    args.push(j);
                }
            }
            "socks5" => {
                args.push("-o".into());
                args.push(format!(
                    "ProxyCommand=nc -X 5 -x {}:{} %h %p",
                    p.host, p.port
                ));
            }
            "http" => {
                args.push("-o".into());
                args.push(format!(
                    "ProxyCommand=nc -X connect -x {}:{} %h %p",
                    p.host, p.port
                ));
            }
            _ => {}
        }
    }
    let user = if profile.user.trim().is_empty() {
        "user"
    } else {
        profile.user.trim()
    };
    let host = if profile.host.trim().is_empty() {
        "host"
    } else {
        profile.host.trim()
    };
    args.push(format!("{user}@{host}"));
    if let Some(cmd) = remote_command {
        if !cmd.trim().is_empty() {
            args.push(cmd.to_string());
        }
    }
    args
}

fn jump_spec(p: &ProxyProfile) -> String {
    if p.host.trim().is_empty() || p.port == 0 {
        return String::new();
    }
    if p.user.trim().is_empty() {
        format!("{}:{}", p.host.trim(), p.port)
    } else {
        format!("{}@{}:{}", p.user.trim(), p.host.trim(), p.port)
    }
}

pub fn build_ssh_command_line(
    profile: &SshProfile,
    proxy: Option<&ProxyProfile>,
    remote_command: Option<&str>,
) -> String {
    let mut parts = vec!["ssh".to_string()];
    for a in build_ssh_args(profile, proxy, remote_command) {
        if a.contains(' ') || a.contains('"') || a.contains('\'') {
            parts.push(format!("\"{}\"", a.replace('"', "\\\"")));
        } else {
            parts.push(a);
        }
    }
    parts.join(" ")
}
