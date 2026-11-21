# LionSSH

[![CI](https://github.com/AmazingCoderA/LionSSH/actions/workflows/ci.yml/badge.svg)](https://github.com/AmazingCoderA/LionSSH/actions/workflows/ci.yml)
[![Release](https://github.com/AmazingCoderA/LionSSH/actions/workflows/release.yml/badge.svg)](https://github.com/AmazingCoderA/LionSSH/actions/workflows/release.yml)
[![Latest Release](https://img.shields.io/github/v/release/AmazingCoderA/LionSSH?display_name=tag&label=release)](https://github.com/AmazingCoderA/LionSSH/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-stable-orange?logo=rust)](https://www.rust-lang.org/)

**Мега TUI command-center на Rust**: SSH, прокси, скрипты, планировщик, алиасы, терминал, темы и эффекты.

<p>
  <a href="https://github.com/AmazingCoderA/LionSSH/releases/latest"><img alt="Download Release" src="https://img.shields.io/badge/Download-Latest%20Release-2ea44f?style=for-the-badge"></a>
  <a href="https://github.com/AmazingCoderA/LionSSH/actions"><img alt="Actions" src="https://img.shields.io/badge/GitHub-Actions-2088FF?style=for-the-badge&logo=github-actions&logoColor=white"></a>
  <a href="https://github.com/AmazingCoderA/LionSSH/issues"><img alt="Issues" src="https://img.shields.io/badge/Report-Issue-f85149?style=for-the-badge&logo=github"></a>
</p>

> Windows/Linux бинарники, автосборка GitHub Actions, релизы, установщики и AUR packaging.

## Быстрые ссылки

- [Возможности](#возможности)
- [Установка](#установка)
- [Сборка из исходников](#сборка-из-исходников)
- [Hotkeys](#hotkeys)
- [Конфиг и данные](#конфиг-и-данные)
- [GitHub Actions / Releases](#github-actions--releases)
- [AUR](#aur)

## Возможности

- **SSH Manager** - профили, identity, startup-команды, заметки, быстрый connect.
- **Proxy Profiles** - `jump` / `socks5` / `http`, привязка к SSH.
- **Script Runner** - `powershell/bash/cmd/shell/ssh`, теги, экспорт, запуск.
- **Scheduler** - периодические/одноразовые задачи, pause/resume, run-now, история.
- **Aliases** - короткие команды для частых действий.
- **Terminal** - запуск команд/алиасов, вывод, скролл, статус выполнения.
- **Themes & Effects** - 8 тем, эффекты, настраиваемый shell/лимит истории.
- **Persistence** - автосохранение workspace в JSON.

## Установка

### Windows (просто exe)

1. Открой [Latest Release](https://github.com/AmazingCoderA/LionSSH/releases/latest)
2. Скачай **`lionssh-x86_64-pc-windows-msvc.exe`** (или `.zip`, если удобнее)
3. Запусти `lionssh.exe`

> Никаких PowerShell install-скриптов не нужно.

### Linux (просто бинарник)

1. Открой [Latest Release](https://github.com/AmazingCoderA/LionSSH/releases/latest)
2. Скачай нужный файл:
   - `lionssh-x86_64-unknown-linux-gnu`
   - `lionssh-x86_64-unknown-linux-musl`
   - `lionssh-aarch64-unknown-linux-gnu`
3. Сделай исполняемым и запусти:

```bash
chmod +x ./lionssh-x86_64-unknown-linux-gnu
./lionssh-x86_64-unknown-linux-gnu
```

Архивы `.tar.gz` тоже остаются в релизе, если удобнее скачать пакет с README/LICENSE.

### AUR

В репозитории есть шаблон `packaging/aur/PKGBUILD`. Для публикации в AUR создай пакет `lionssh-git` (или `lionssh`) и замени `AmazingCoderA/LionSSH` на реальный репозиторий.

```bash
cd packaging/aur
makepkg -si
```

## Сборка из исходников

```bash
cargo build --release
```

Запуск dev:

```bash
cargo run
```

## Hotkeys

| Область | Клавиши |
|---|---|
| Global | `Tab/BackTab` вкладки, `1-8` переход, `Ctrl+S` сохранить, `?` help, `q` выход |
| Terminal | `c` команда/алиас, `l` очистить, `Up/Down` скролл |
| SSH | `a` add, `e` edit, `d` delete, `Enter/c` connect, `t` show command |
| Scripts | `a/e/d`, `Enter/r` run, `x` export |
| Scheduler | `a/e/d`, `Space` pause/resume, `Enter/r` run now |
| Proxies | `a/e/d` |
| Aliases | `a/e/d`, `Enter/r` run |
| Settings | `t` theme, `e` effects, `m` mode, `s` shell, `l` log limit |

## Формат ввода

- Для многострочного текста в формах используй `;;` как разделитель строк.
- `Scheduler target kind`: `command` = shell команда/алиас, `script` = id скрипта.
- `SSH script`: content выполняется как remote command через выбранный SSH профиль.
- Proxy kinds: `jump` (ProxyJump), `socks5/http` (ProxyCommand через `nc`).

## Конфиг и данные

- Workspace: `%LOCALAPPDATA%\LionSSH\workspace.json` (Windows) / data_local/home на Linux.
- Export scripts: `%LOCALAPPDATA%\LionSSH\scripts\...`

## GitHub Actions / Releases

В проект добавлены workflow:

- `.github/workflows/ci.yml` - проверка на push/PR.
- `.github/workflows/release.yml` - автосборка бинарников и публикация GitHub Release по тегу `v*`.

### Как выпустить релиз

```bash
git add .
git commit -m "feat: initial LionSSH mega release"
git tag v1.0.0
git push origin main --tags
```

После пуша тега Release workflow соберет:

- Windows: `x86_64-pc-windows-msvc` (.zip)
- Linux: `x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-gnu` (.tar.gz)
- `SHA256SUMS.txt` и `.sha256` для артефактов

## AUR

Файл: `packaging/aur/PKGBUILD` (шаблон `lionssh-git`). Перед публикацией:

1. Замени `AmazingCoderA/LionSSH` на реальный GitHub repo.
2. Проверь `pkgver()`, зависимости, license.
3. Сгенерируй `.SRCINFO`: `makepkg --printsrcinfo > .SRCINFO`.
4. Запушь в AUR repo.

## Зависимости

- Runtime: `ssh` в PATH; для socks/http proxy - `nc` (netcat).
- Build: Rust stable toolchain.

## Лицензия

MIT - см. [LICENSE](LICENSE).

