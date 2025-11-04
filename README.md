<div align="center">

```
 ███╗   ███╗██╗███╗   ██╗███████╗   ██████╗ ██╗     ██╗
 ████╗ ████║██║████╗  ██║██╔════╝   ██╔════╝██║     ██║
 ██╔████╔██║██║██╔██╗ ██║█████╗     ██║     ██║     ██║
 ██║╚██╔╝██║██║██║╚██╗██║██╔══╝     ██║     ██║     ██║
 ██║ ╚═╝ ██║██║██║ ╚████║███████╗   ╚██████╗███████╗██║
 ╚═╝     ╚═╝╚═╝╚═╝  ╚═══╝╚══════╝    ╚═════╝╚══════╝╚═╝
```

**A fast, beautiful terminal interface for Redmine**

Built with Rust • Powered by Ratatui

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

[Installation](#installation) • [Features](#features) • [Usage](#usage) • [Configuration](#configuration)

</div>

---

## Why MineCLI?

Work with Redmine projects and issues directly from your terminal. No browser needed. Fast, keyboard-driven, and works offline with smart caching.

## Features

### Core Functionality
- **Browse Projects** - View all your Redmine projects with real-time sync
- **Manage Issues** - Create, update, and view issues with full history
- **Smart Filtering** - Filter by status, assignee, or search by keywords
- **Bulk Operations** - Update multiple issues at once
- **Offline Mode** - SQLite caching for working without connection
- **Attachments** - View and download issue attachments

### Interface
- **Vim Navigation** - Use `hjkl` or arrow keys
- **Mouse Support** - Click and scroll anywhere
- **Multiple Themes** - Default, plus 4 Catppuccin variants (Mocha, Macchiato, Frappe, Latte)
- **Context Help** - Press `?` for keyboard shortcuts
- **Responsive** - Adapts to your terminal size

### Workflow
- Sort by status, priority, or recent updates
- Group issues by status with collapse/expand
- "My Issues" filter for assigned issues
- Open issues in browser with one key
- Add public or private comments

## Installation

### Homebrew (macOS/Linux)

```bash
brew install ecugol/minecli/minecli
```

### Shell Script (macOS/Linux)

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/ecugol/minecli/releases/latest/download/minecli-installer.sh | sh
```

### PowerShell (Windows)

```powershell
powershell -c "irm https://github.com/ecugol/minecli/releases/latest/download/minecli-installer.ps1 | iex"
```

### MSI Installer (Windows)

Download from [latest release](https://github.com/ecugol/minecli/releases/latest)

### From Source

```bash
git clone https://github.com/ecugol/minecli.git
cd minecli
cargo build --release
```

## Quick Start

1. **Launch MineCLI**
   ```bash
   minecli
   ```

2. **Configure** (first run only)
   - Enter your Redmine URL (e.g., `https://redmine.example.com`)
   - Enter your API key (find in Account Settings)
   - Choose a theme
   - Press `ESC`

3. **Start Working**
   - Press `P` to sync projects
   - Use `j/k` to navigate
   - Press `Enter` to open
   - Press `?` for help

## Usage

### Essential Keys

| Key | Action |
|-----|--------|
| `h` / `l` | Switch panes (projects ↔ issues) |
| `j` / `k` | Navigate up/down |
| `Enter` | Select/Open |
| `/` | Search |
| `?` | Show help |
| `q` | Quit |

### Projects

| Key | Action |
|-----|--------|
| `P` | Sync projects from server |

### Issues

| Key | Action |
|-----|--------|
| `I` | Sync issues from server |
| `n` | Create new issue |
| `s` | Cycle sort order |
| `g` | Toggle status grouping |
| `m` | Toggle "My Issues" filter |
| `b` | Bulk edit selected issues |

### Issue Details

| Key | Action |
|-----|--------|
| `j` / `k` | Scroll |
| `g` / `G` | Top / Bottom |
| `r` | Reply/Update issue |
| `O` | Open in browser |
| `1-9` | Open attachment |
| `[` / `]` | Previous/Next attachment page |
| `J` / `K` | Next/Previous issue |
| `ESC` | Close |

### Forms

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Next/Previous field |
| `Ctrl+S` | Submit |
| `/` | Search in dropdown |
| `ESC` | Cancel |

## Configuration

Press `c` to open configuration screen, or edit manually:

**Config Location:**
- macOS: `~/Library/Application Support/redmine-tui/config.toml`
- Linux: `~/.config/redmine-tui/config.toml`
- Windows: `%APPDATA%\redmine-tui\config.toml`

```toml
redmine_url = "https://your-redmine.com"
api_key = "your_api_key"
theme = "CatppuccinMocha"
```

**Available Themes:**
- `Default`
- `CatppuccinMocha` (dark, warm)
- `CatppuccinMacchiato` (dark, cool)
- `CatppuccinFrappe` (dark, neutral)
- `CatppuccinLatte` (light)

## Requirements

- Redmine instance with REST API enabled
- Redmine API key (Account → Settings → Show API key)

## Building

```bash
# Run
cargo run

# Test
cargo test

# Build release
cargo build --release
```

## License

MIT License - see [LICENSE](LICENSE)

## Built With

- [Ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [Crossterm](https://github.com/crossterm-rs/crossterm) - Terminal manipulation
- [Tokio](https://tokio.rs/) - Async runtime
- [Rusqlite](https://github.com/rusqlite/rusqlite) - SQLite bindings

---

<div align="center">

**Star this project if you find it useful!** ⭐

</div>
