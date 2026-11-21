<p align="center">
  <a href="README.md"><img alt="RU" src="https://img.shields.io/badge/RU-Русский-1f2937?style=for-the-badge"></a>
  <a href="README.en.md"><img alt="EN" src="https://img.shields.io/badge/EN-English-9b6bff?style=for-the-badge"></a>
  <a href="README.uk.md"><img alt="UK" src="https://img.shields.io/badge/UK-Українська-0057B8?style=for-the-badge"></a>
  <a href="README.de.md"><img alt="DE" src="https://img.shields.io/badge/DE-Deutsch-333333?style=for-the-badge"></a>
</p>

# LionSSH

[![Build](https://github.com/AmazingCoderA/LionSSH/actions/workflows/build.yml/badge.svg)](https://github.com/AmazingCoderA/LionSSH/actions/workflows/build.yml)
[![Release](https://github.com/AmazingCoderA/LionSSH/actions/workflows/release.yml/badge.svg)](https://github.com/AmazingCoderA/LionSSH/actions/workflows/release.yml)
[![Latest Release](https://img.shields.io/github/v/release/AmazingCoderA/LionSSH?display_name=tag&label=release)](https://github.com/AmazingCoderA/LionSSH/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

**LionSSH** is a command center for SSH, proxies, scripts, scheduler, aliases, and history.

Two versions:
- **GUI** — desktop app (`lionssh-gui`)
- **CLI** — terminal tool (`lionssh`)

Both share the same `workspace.json`.

## What's new

- **Active SSH profile per tab** in GUI (persisted).
- **Multilanguage GUI**: RU / EN / UK / DE.
- Switch language from the top bar or Settings; CLI: `lionssh config set-language ru|en|uk|de`.

## Download (no install scripts)

### Windows
Download from [Latest Release](https://github.com/AmazingCoderA/LionSSH/releases/latest):
- **GUI**: `lionssh-gui-x86_64-pc-windows-msvc.exe`
- **CLI**: `lionssh-x86_64-pc-windows-msvc.exe`

Run directly.

### Linux
Download from [Latest Release](https://github.com/AmazingCoderA/LionSSH/releases/latest):
- **GUI**: `lionssh-gui-x86_64-unknown-linux-gnu`
- **CLI**: `lionssh-x86_64-unknown-linux-gnu`, `lionssh-x86_64-unknown-linux-musl`, `lionssh-aarch64-unknown-linux-gnu`

```bash
chmod +x ./lionssh-gui-x86_64-unknown-linux-gnu
./lionssh-gui-x86_64-unknown-linux-gnu
```

> Linux GUI requires X11/Wayland + OpenGL. Use CLI on headless servers.

## Features

- **SSH profiles** — host/user/port/identity, startup commands, proxy binding, connect/copy command.
- **Proxies** — `jump` / `socks5` / `http`.
- **Scripts** — `shell/powershell/bash/cmd/ssh`, tags, description, run/export.
- **Scheduler** — command/script tasks, interval, enable/disable, run now, history.
- **Aliases** — short command names.
- **Run/Terminal** — execute commands/aliases, capture output, status.
- **Active profile per tab** — selected SSH profile is saved per GUI tab.
- **Multilanguage GUI** — Russian, English, Ukrainian, German.
- **Shared storage** — `%LOCALAPPDATA%\LionSSH\workspace.json` (Windows) / data dir (Linux).

## CLI examples

```bash
lionssh paths
lionssh config set-language en
lionssh run "echo hello"
lionssh ssh list
lionssh ssh add my-server 1.2.3.4 --user root --port 22
lionssh ssh connect my-server
lionssh proxy add home --kind socks5 --host 127.0.0.1 --port 1080
lionssh script add backup --kind bash --content "tar -czf backup.tgz ./data"
lionssh script run backup
lionssh task add hourly-backup --kind script --target backup --interval 3600
lionssh alias add ll "ls -la"
lionssh alias run ll
lionssh history --limit 20
```

## Build from source

```bash
# CLI only
cargo build --release --no-default-features --bin lionssh

# GUI
cargo build --release --bin lionssh-gui
```

## License

MIT — see [LICENSE](LICENSE).
