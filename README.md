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

## Что нового

- Выбор активного SSH-профиля **на каждой вкладке** (в GUI, с сохранением).
- Интерфейс GUI на двух языках: **Русский / English**.
- Язык можно переключать вверху окна и в Settings; в CLI: `lionssh config set-language ru|en`.

## Скачать (без install-скриптов)

Все файлы лежат в [Latest Release](https://github.com/AmazingCoderA/LionSSH/releases/latest) — просто скачай нужный бинарник.

| Платформа | GUI | CLI |
|---|---|---|
| Windows x64 | `lionssh-gui-windows-x64.exe` | `lionssh-cli-windows-x64.exe` |
| Linux x64 | `lionssh-gui-linux-x64` | `lionssh-cli-linux-x64` |
| Linux x64 (musl/static) | — | `lionssh-cli-linux-x64-musl` |
| Linux ARM64 | — | `lionssh-cli-linux-arm64` |
| Android ARM64 | — | `lionssh-cli-android-arm64` |

```bash
chmod +x ./lionssh-gui-linux-x64
./lionssh-gui-linux-x64
```

> GUI на Linux требует X11/Wayland + OpenGL. Для headless/Android используй CLI.

## Возможности

- **SSH profiles** — host/user/port/identity, startup commands, proxy binding, connect/copy command.
- **Proxies** — `jump` / `socks5` / `http`.
- **Scripts** — `shell/powershell/bash/cmd/ssh`, теги, описание, run/export.
- **Scheduler** — command/script задачи, interval, enable/disable, run now, история.
- **Aliases** — короткие имена для команд.
- **Run/Terminal** — выполнение команд/алиасов, захват вывода, статус.
- **Active profile per tab** — выбранный SSH-профиль сохраняется для каждой вкладки GUI.
- **GUI language** — Русский / English.
- **Shared storage** — `%LOCALAPPDATA%\LionSSH\workspace.json` (Windows) / data dir (Linux).

## CLI примеры

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
- прямые бинарники с понятными именами: `lionssh-gui-<platform>[.exe]`, `lionssh-cli-<platform>[.exe]`
- архивы `.zip/.tar.gz`
- `SHA256SUMS.txt` и `.sha256`

## AUR

Шаблон: `packaging/aur/PKGBUILD`.

## License

MIT — см. [LICENSE](LICENSE).
