# Contributing to Redmine TUI

Thank you for your interest in contributing! This document provides guidelines for contributing to this project.

## Development Setup

1. **Prerequisites:**
   - Rust 1.70+ (via rustup recommended)
   - A Redmine instance for testing

2. **Clone and Build:**
   ```bash
   git clone <repository-url>
   cd rs-redmine
   cargo build
   ```

3. **Run Tests:**
   ```bash
   cargo test
   ```

4. **Run the Application:**
   ```bash
   cargo run
   ```

## Code Style

- Follow the Rust style guide
- Run `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Keep functions focused and small
- Add comments for complex logic
- Write tests for new features

## Project Structure

```
src/
├── app/              # Application state and logic
│   ├── state.rs      # App struct and enums
│   ├── handlers.rs   # Event handling
│   ├── data_loader.rs # API operations
│   └── filters.rs    # Filtering and helpers
├── ui/               # User interface (to be refactored)
├── redmine/          # Redmine API client
│   ├── client.rs     # HTTP client
│   └── models.rs     # Data models
├── config.rs         # Configuration management
├── db.rs             # SQLite database layer
├── form_field.rs     # Form field abstractions
├── issue_form.rs     # Issue form builder
├── events.rs         # Event handling system
└── main.rs           # Entry point and event loop
```

## Architecture

- **State Management**: App state is centralized in `app/state.rs`
- **Event Handling**: Keyboard and mouse events in `app/handlers.rs`
- **Data Loading**: Async API operations in `app/data_loader.rs`
- **UI Rendering**: Ratatui-based UI in `ui.rs` (being refactored)
- **Caching**: SQLite for offline access via `db.rs`

## Submitting Changes

1. **Fork the repository**
2. **Create a feature branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```
3. **Make your changes:**
   - Write clear, descriptive commit messages
   - Add tests for new features
   - Update documentation as needed
4. **Test thoroughly:**
   ```bash
   cargo test
   cargo clippy
   cargo fmt -- --check
   cargo build --release
   ```
5. **Submit a pull request:**
   - Describe your changes clearly
   - Reference any related issues
   - Ensure CI passes (when set up)

## Reporting Issues

When reporting bugs, please include:
- Your OS and Rust version
- Steps to reproduce the issue
- Expected vs actual behavior
- Any error messages or logs
- Screenshots if relevant

## Feature Requests

Feature requests are welcome! Please:
- Check existing issues first to avoid duplicates
- Explain the use case and problem it solves
- Describe expected behavior
- Consider implementation complexity

## Development Tips

- Use `RUST_LOG=debug cargo run` for detailed logging (when logging is added)
- Test with different Redmine versions if possible
- Check the Redmine REST API documentation
- Consider offline functionality when adding features
- Maintain keyboard-first design philosophy

## Code of Conduct

- Be respectful and inclusive
- Welcome newcomers and help them learn
- Provide constructive feedback
- Keep discussions on-topic and professional
- Focus on the code, not the person

## Questions?

Open an issue for questions or clarifications. We're here to help!

Thank you for contributing! 🎉
