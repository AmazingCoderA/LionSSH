<p align="center">
  <a href="README.md"><img alt="RU" src="https://img.shields.io/badge/RU-Русский-1f2937?style=for-the-badge"></a>
  <a href="README.en.md"><img alt="EN" src="https://img.shields.io/badge/EN-English-1f2937?style=for-the-badge"></a>
  <a href="README.uk.md"><img alt="UK" src="https://img.shields.io/badge/UK-Українська-0057B8?style=for-the-badge"></a>
  <a href="README.de.md"><img alt="DE" src="https://img.shields.io/badge/DE-Deutsch-9b6bff?style=for-the-badge"></a>
</p>

# LionSSH

[![Build](https://github.com/AmazingCoderA/LionSSH/actions/workflows/build.yml/badge.svg)](https://github.com/AmazingCoderA/LionSSH/actions/workflows/build.yml)
[![Release](https://github.com/AmazingCoderA/LionSSH/actions/workflows/release.yml/badge.svg)](https://github.com/AmazingCoderA/LionSSH/actions/workflows/release.yml)
[![Latest Release](https://img.shields.io/github/v/release/AmazingCoderA/LionSSH?display_name=tag&label=release)](https://github.com/AmazingCoderA/LionSSH/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

**LionSSH** — ein Command Center für SSH, Proxys, Skripte, Scheduler, Aliase und Verlauf.

Zwei Versionen:
- **GUI** — Desktop-App (`lionssh-gui`)
- **CLI** — Terminal-Tool (`lionssh`)

Beide nutzen dieselbe `workspace.json`.

## Neu

- **Aktives SSH-Profil pro Tab** in der GUI (gespeichert).
- **Mehrsprachige GUI**: RU / EN / UK / DE.
- Sprache oben oder in Settings wechseln; CLI: `lionssh config set-language ru|en|uk|de`.

## Download (ohne Install-Skripte)

### Windows
Aus dem [Latest Release](https://github.com/AmazingCoderA/LionSSH/releases/latest) herunterladen:
- **GUI**: `lionssh-gui-x86_64-pc-windows-msvc.exe`
- **CLI**: `lionssh-x86_64-pc-windows-msvc.exe`

Direkt starten.

### Linux
Aus dem [Latest Release](https://github.com/AmazingCoderA/LionSSH/releases/latest):
- **GUI**: `lionssh-gui-x86_64-unknown-linux-gnu`
- **CLI**: `lionssh-x86_64-unknown-linux-gnu`, `lionssh-x86_64-unknown-linux-musl`, `lionssh-aarch64-unknown-linux-gnu`

```bash
chmod +x ./lionssh-gui-x86_64-unknown-linux-gnu
./lionssh-gui-x86_64-unknown-linux-gnu
```

> Linux-GUI benötigt X11/Wayland + OpenGL. Auf Headless-Servern CLI verwenden.

## Funktionen

- **SSH-Profile** — Host/User/Port/Identity, Startup-Befehle, Proxy-Binding, Connect/Copy.
- **Proxys** — `jump` / `socks5` / `http`.
- **Skripte** — `shell/powershell/bash/cmd/ssh`, Tags, Beschreibung, Run/Export.
- **Scheduler** — Command/Script-Tasks, Interval, Enable/Disable, Run now, Verlauf.
- **Aliase** — kurze Befehlsnamen.
- **Run/Terminal** — Befehle/Aliase ausführen, Ausgabe, Status.
- **Aktives Profil pro Tab** — ausgewähltes SSH-Profil wird pro GUI-Tab gespeichert.
- **Mehrsprachige GUI** — Russisch, Englisch, Ukrainisch, Deutsch.
- **Gemeinsamer Speicher** — `%LOCALAPPDATA%\LionSSH\workspace.json` (Windows) / data dir (Linux).

## CLI-Beispiele

```bash
lionssh paths
lionssh config set-language de
lionssh run "echo hello"
lionssh ssh list
lionssh ssh connect my-server
```

## Build

```bash
cargo build --release --no-default-features --bin lionssh
cargo build --release --bin lionssh-gui
```

## Lizenz

MIT — siehe [LICENSE](LICENSE).
