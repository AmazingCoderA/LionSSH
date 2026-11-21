# LionSSH

Мега TUI command-center на Rust: SSH, прокси, скрипты, планировщик, алиасы, терминал, темы и эффекты.

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

### Windows (Release binary)

Скачай `lionssh-*-x86_64-pc-windows-msvc.zip` из Releases, распакуй и запусти `lionssh.exe`.

Или PowerShell installer (после публикации релиза, замени `AmazingCoderA/LionSSH`):

```powershell
irm https://raw.githubusercontent.com/AmazingCoderA/LionSSH/main/packaging/install/install-windows.ps1 | iex
```

### Linux (Release binary)

Скачай архив под свой target из Releases:

- `x86_64-unknown-linux-gnu`
- `x86_64-unknown-linux-musl`
- `aarch64-unknown-linux-gnu`

Или installer script:

```bash
curl -fsSL https://raw.githubusercontent.com/AmazingCoderA/LionSSH/main/packaging/install/install-linux.sh | bash
```

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

