use anyhow::{anyhow, Context, Result};
use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct RunResult {
    pub ok: bool,
    pub status: String,
    pub output: String,
    pub duration_ms: u128,
}

fn shell_command(shell: &str) -> Command {
    match shell {
        "bash" | "sh" => {
            let mut c = Command::new("bash");
            c.arg("-lc");
            c
        }
        "cmd" => {
            let mut c = Command::new("cmd");
            c.arg("/C");
            c
        }
        _ => {
            let mut c = Command::new("powershell");
            c.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command"]);
            c
        }
    }
}

pub fn run_shell(shell: &str, command: &str, timeout: Option<Duration>) -> Result<RunResult> {
    let mut cmd = shell_command(shell);
    cmd.arg(command);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let started = Instant::now();
    let mut child = cmd.spawn().with_context(|| format!("spawn {shell}"))?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("stdout missing"))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("stderr missing"))?;
    let out_h = std::thread::spawn(move || {
        let mut b = Vec::new();
        let _ = stdout.read_to_end(&mut b);
        b
    });
    let err_h = std::thread::spawn(move || {
        let mut b = Vec::new();
        let _ = stderr.read_to_end(&mut b);
        b
    });

    let status = if let Some(t) = timeout {
        let deadline = Instant::now() + t;
        loop {
            if let Some(s) = child.try_wait()? {
                break s;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                let out = String::from_utf8_lossy(&out_h.join().unwrap_or_default()).to_string();
                let err = String::from_utf8_lossy(&err_h.join().unwrap_or_default()).to_string();
                return Ok(RunResult {
                    ok: false,
                    status: "timeout".into(),
                    output: trim_output(merge(out, err)),
                    duration_ms: started.elapsed().as_millis(),
                });
            }
            std::thread::sleep(Duration::from_millis(30));
        }
    } else {
        child.wait()?
    };

    let out = String::from_utf8_lossy(&out_h.join().unwrap_or_default()).to_string();
    let err = String::from_utf8_lossy(&err_h.join().unwrap_or_default()).to_string();
    Ok(RunResult {
        ok: status.success(),
        status: status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "signal".into()),
        output: trim_output(merge(out, err)),
        duration_ms: started.elapsed().as_millis(),
    })
}

fn merge(out: String, err: String) -> String {
    let o = out.trim();
    let e = err.trim();
    match (o.is_empty(), e.is_empty()) {
        (true, true) => String::new(),
        (false, true) => o.to_string(),
        (true, false) => e.to_string(),
        (false, false) => format!("{o}\n--- stderr ---\n{e}"),
    }
}

fn trim_output(mut s: String) -> String {
    const MAX: usize = 20000;
    if s.len() > MAX {
        s.truncate(MAX);
        s.push_str("\n... [truncated]");
    }
    s
}

pub fn script_shell<'a>(kind: &'a str, default_shell: &'a str) -> &'a str {
    match kind {
        "powershell" | "bash" | "cmd" => kind,
        "shell" => default_shell,
        _ => default_shell,
    }
}
