# LionSSH

[![Build](https://github.com/AmazingCoderA/LionSSH/actions/workflows/build.yml/badge.svg)](https://github.com/AmazingCoderA/LionSSH/actions/workflows/build.yml)
[![Release](https://github.com/AmazingCoderA/LionSSH/actions/workflows/release.yml/badge.svg)](https://github.com/AmazingCoderA/LionSSH/actions/workflows/release.yml)
[![Latest Release](https://img.shields.io/github/v/release/AmazingCoderA/LionSSH?display_name=tag&label=release)](https://github.com/AmazingCoderA/LionSSH/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

**LionSSH** — command center для SSH, прокси, скриптов, планировщика, алиасов и истории.  

Две версии:
- **GUI** — десктопное приложение (`lionssh-gui`)
- **CLI** — терминальная утилита (`lionssh`)

Обе версии используют общий `workspace.json`.

## Скачать (без install-скриптов)

### Windows
Из [Latest Release](https://github.com/AmazingCoderA/LionSSH/releases/latest) скачай нужный файл:
- **GUI**: `lionssh-gui-x86_64-pc-windows-msvc.exe`
- **CLI**: `lionssh-x86_64-pc-windows-msvc.exe`

Запускай напрямую.

### Linux
Из [Latest Release](https://github.com/AmazingCoderA/LionSSH/releases/latest):
- **GUI**: `lionssh-gui-x86_64-unknown-linux-gnu`
- **CLI**: `lionssh-x86_64-unknown-linux-gnu`, `lionssh-x86_64-unknown-linux-musl`, `lionssh-aarch64-unknown-linux-gnu`

```bash
chmod +x ./lionssh-gui-x86_64-unknown-linux-gnu
./lionssh-gui-x86_64-unknown-linux-gnu
```

> GUI на Linux требует X11/Wayland + OpenGL. Для headless-серверов используй CLI.

## Возможности

- **SSH profiles** — host/user/port/identity, startup commands, proxy binding, connect/copy command.
- **Proxies** — `jump` / `socks5` / `http`.
- **Scripts** — `shell/powershell/bash/cmd/ssh`, теги, описание, run/export.
- **Scheduler** — command/script задачи, interval, enable/disable, run now, история.
- **Aliases** — короткие имена для команд.
- **Run/Terminal** — выполнение команд/алиасов, захват вывода, статус.
- **Shared storage** — `%LOCALAPPDATA%\LionSSH\workspace.json` (Windows) / data dir (Linux).

## CLI примеры

```bash
lionssh paths
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

## Сборка из исходников

```bash
# CLI only
cargo build --release --no-default-features --bin lionssh

# GUI
cargo build --release --bin lionssh-gui
```

## GitHub Actions / Releases

- `.github/workflows/build.yml` — сборка-проверка (Windows/Linux/macOS).
- `.github/workflows/release.yml` — релиз по тегу `v*`.

Релиз публикует:
- прямые бинарники: `lionssh-<target>[.exe]`, `lionssh-gui-<target>[.exe]` (где GUI поддерживается)
- архивы `.zip/.tar.gz`
- `SHA256SUMS.txt` и `.sha256`

## AUR

Шаблон: `packaging/aur/PKGBUILD`.

## License

MIT — см. [LICENSE](LICENSE).