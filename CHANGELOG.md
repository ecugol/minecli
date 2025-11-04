# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2024-11-04

### Initial Release

**MineCLI** - A fast, beautiful terminal interface for Redmine project management.

#### Features

- **Project Management**: Browse all Redmine projects with pagination and filtering
- **Issue Management**: Create, update, and view issues with full history
- **Bulk Operations**: Update multiple issues at once
- **Smart Filtering**: Filter by status, assignee, or search by keywords
- **Offline Mode**: SQLite caching for working without connection
- **Attachments**: View and download issue attachments
- **Vim Navigation**: Full keyboard navigation with hjkl keys
- **Mouse Support**: Click and scroll anywhere in the interface
- **Multiple Themes**: Choose from 5 color themes (Default + Catppuccin variants)
- **Context Help**: Press `?` for keyboard shortcuts

#### Installation Methods

**macOS (Apple Silicon)**
- Download: `minecli-aarch64-apple-darwin.tar.xz`
- Homebrew: `brew install ecugol/minecli/minecli`

**macOS (Intel)**
- Download: `minecli-x86_64-apple-darwin.tar.xz`
- Homebrew: `brew install ecugol/minecli/minecli`

**Linux (ARM64)**
- Download: `minecli-aarch64-unknown-linux-gnu.tar.xz`

**Linux (x64)**
- Download: `minecli-x86_64-unknown-linux-gnu.tar.xz`

**Windows (x64)**
- MSI Installer: `minecli-x86_64-pc-windows-msvc.msi`
- Zip Archive: `minecli-x86_64-pc-windows-msvc.zip`
- PowerShell: `irm https://github.com/ecugol/minecli/releases/latest/download/minecli-installer.ps1 | iex`

#### Requirements

- Redmine instance with REST API enabled
- Redmine API key (found in Account Settings)

---

**Full documentation**: https://github.com/ecugol/minecli
