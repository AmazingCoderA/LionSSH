<p align="center">
  <a href="README.md"><img alt="RU" src="https://img.shields.io/badge/RU-Русский-1f2937?style=for-the-badge"></a>
  <a href="README.en.md"><img alt="EN" src="https://img.shields.io/badge/EN-English-1f2937?style=for-the-badge"></a>
  <a href="README.uk.md"><img alt="UK" src="https://img.shields.io/badge/UK-Українська-9b6bff?style=for-the-badge"></a>
  <a href="README.de.md"><img alt="DE" src="https://img.shields.io/badge/DE-Deutsch-333333?style=for-the-badge"></a>
</p>

# LionSSH

[![Build](https://github.com/AmazingCoderA/LionSSH/actions/workflows/build.yml/badge.svg)](https://github.com/AmazingCoderA/LionSSH/actions/workflows/build.yml)
[![Release](https://github.com/AmazingCoderA/LionSSH/actions/workflows/release.yml/badge.svg)](https://github.com/AmazingCoderA/LionSSH/actions/workflows/release.yml)
[![Latest Release](https://img.shields.io/github/v/release/AmazingCoderA/LionSSH?display_name=tag&label=release)](https://github.com/AmazingCoderA/LionSSH/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

**LionSSH** — командний центр для SSH, проксі, скриптів, планувальника, аліасів та історії.

Дві версії:
- **GUI** — десктопний застосунок (`lionssh-gui`)
- **CLI** — термінальна утиліта (`lionssh`)

Обидві версії використовують спільний `workspace.json`.

## Що нового

- **Вибір активного SSH-профілю на кожній вкладці** в GUI (зберігається).
- **Багатомовний GUI**: RU / EN / UK / DE.
- Перемикання мови у верхній панелі або в Settings; CLI: `lionssh config set-language ru|en|uk|de`.

## Завантаження (без install-скриптів)

### Windows
Завантаж з [Latest Release](https://github.com/AmazingCoderA/LionSSH/releases/latest):
- **GUI**: `lionssh-gui-x86_64-pc-windows-msvc.exe`
- **CLI**: `lionssh-x86_64-pc-windows-msvc.exe`

Запускай напряму.

### Linux
Завантаж з [Latest Release](https://github.com/AmazingCoderA/LionSSH/releases/latest):
- **GUI**: `lionssh-gui-x86_64-unknown-linux-gnu`
- **CLI**: `lionssh-x86_64-unknown-linux-gnu`, `lionssh-x86_64-unknown-linux-musl`, `lionssh-aarch64-unknown-linux-gnu`

```bash
chmod +x ./lionssh-gui-x86_64-unknown-linux-gnu
./lionssh-gui-x86_64-unknown-linux-gnu
```

> GUI на Linux потребує X11/Wayland + OpenGL. Для headless-серверів використовуй CLI.

## Можливості

- **SSH profiles** — host/user/port/identity, startup commands, proxy binding, connect/copy command.
- **Proxies** — `jump` / `socks5` / `http`.
- **Scripts** — `shell/powershell/bash/cmd/ssh`, теги, опис, run/export.
- **Scheduler** — command/script задачі, interval, enable/disable, run now, історія.
- **Aliases** — короткі імена для команд.
- **Run/Terminal** — виконання команд/аліасів, вивід, статус.
- **Активний профіль на вкладці** — обраний SSH-профіль зберігається для кожної вкладки GUI.
- **Багатомовний GUI** — Русский, English, Українська, Deutsch.
- **Shared storage** — `%LOCALAPPDATA%\LionSSH\workspace.json` (Windows) / data dir (Linux).

## CLI приклади

```bash
lionssh paths
lionssh config set-language uk
lionssh run "echo hello"
lionssh ssh list
lionssh ssh connect my-server
```

## Збірка

```bash
cargo build --release --no-default-features --bin lionssh
cargo build --release --bin lionssh-gui
```

## License

MIT — див. [LICENSE](LICENSE).
