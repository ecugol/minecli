//! Form field input handlers for issue creation and update forms.
//!
//! This module handles keyboard input for various form field types including:
//! - Text and textarea fields
//! - Dropdown menus (simple and searchable)
//! - Progress bars
//! - Checkboxes
//!
//! The handlers support form navigation, search functionality, and validation.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;
use crate::form_field::{FieldType, FieldValue};

impl App {
    /// Route form field input to the appropriate handler
    pub(crate) fn handle_form_input(&mut self, key: KeyEvent, is_update: bool) {
        let form = if is_update {
            self.update_issue_form.as_mut()
        } else {
            self.create_issue_form.as_mut()
        };

        // Extract field info before calling handler methods to avoid borrow conflicts
        let field_info = form.and_then(|f| {
            f.get_current_field().map(|field| {
                let old_tracker_id = if field.key == "tracker_id" && !is_update {
                    f.get_value("tracker_id").and_then(|v| v.as_option_id())
                } else {
                    None
                };
                (field.key.clone(), field.field_type.clone(), old_tracker_id)
            })
        });

        if let Some((field_key, field_type, old_tracker_id)) = field_info {
            let is_tracker_field = field_key == "tracker_id";

            match field_type {
                FieldType::Text | FieldType::TextArea | FieldType::Date | FieldType::Float => {
                    self.handle_text_field_input(key, &field_key, field_type, is_update);
                }
                FieldType::Dropdown => {
                    self.handle_dropdown_input(key, &field_key, is_update);
                }
                FieldType::SearchableDropdown => {
                    self.handle_searchable_dropdown_input(key, &field_key, is_update);
                }
                FieldType::Progress => {
                    self.handle_progress_input(key, &field_key, is_update);
                }
                FieldType::Checkbox => {
                    self.handle_checkbox_input(key, &field_key, is_update);
                }
                _ => {}
            }

            // Check if tracker changed - if so, rebuild form with preserved values
            if is_tracker_field && !is_update {
                if let Some(form) = self.create_issue_form.as_ref() {
                    let new_tracker_id = form.get_value("tracker_id").and_then(|v| v.as_option_id());
                    if old_tracker_id != new_tracker_id && new_tracker_id.is_some() {
                        self.rebuild_create_issue_form_preserving_values();
                    }
                }
            }
        }
    }

    pub(crate) fn handle_text_field_input(
        &mut self,
        key: KeyEvent,
        field_key: &str,
        field_type: FieldType,
        is_update: bool,
    ) {
        let form = if is_update {
            self.update_issue_form.as_mut()
        } else {
            self.create_issue_form.as_mut()
        };

        if let Some(form) = form {
            match key.code {
                KeyCode::Char(c) => {
                    let mut text = form
                        .get_value(field_key)
                        .and_then(|v| v.as_text())
                        .unwrap_or("")
                        .to_string();
                    text.push(c);
                    form.set_value(field_key.to_string(), FieldValue::Text(text));
                }
                KeyCode::Backspace => {
                    let mut text = form
                        .get_value(field_key)
                        .and_then(|v| v.as_text())
                        .unwrap_or("")
                        .to_string();
                    text.pop();
                    form.set_value(field_key.to_string(), FieldValue::Text(text));
                }
                KeyCode::Enter if field_type == FieldType::TextArea => {
                    let mut text = form
                        .get_value(field_key)
                        .and_then(|v| v.as_text())
                        .unwrap_or("")
                        .to_string();
                    text.push('\n');
                    form.set_value(field_key.to_string(), FieldValue::Text(text));
                }
                _ => {}
            }
        }
    }

    pub(crate) fn handle_dropdown_input(&mut self, key: KeyEvent, field_key: &str, is_update: bool) {
        let form = if is_update {
            self.update_issue_form.as_mut()
        } else {
            self.create_issue_form.as_mut()
        };

        if let Some(form) = form {
            let field = form.get_current_field();
            if let Some(field) = field {
                let options = &field.options;
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if !options.is_empty() {
                            let current_id = form.get_value(field_key).and_then(|v| v.as_option_id());
                            let current_idx = current_id
                                .and_then(|id| options.iter().position(|opt| opt.id == id))
                                .unwrap_or(0);
                            let new_idx = if current_idx > 0 {
                                current_idx - 1
                            } else {
                                options.len() - 1
                            };
                            form.set_value(field_key.to_string(), FieldValue::OptionId(Some(options[new_idx].id)));
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if !options.is_empty() {
                            let current_id = form.get_value(field_key).and_then(|v| v.as_option_id());
                            let current_idx = current_id
                                .and_then(|id| options.iter().position(|opt| opt.id == id))
                                .unwrap_or(0);
                            let new_idx = (current_idx + 1) % options.len();
                            form.set_value(field_key.to_string(), FieldValue::OptionId(Some(options[new_idx].id)));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub(crate) fn handle_searchable_dropdown_input(&mut self, key: KeyEvent, field_key: &str, is_update: bool) {
        let form = if is_update {
            self.update_issue_form.as_mut()
        } else {
            self.create_issue_form.as_mut()
        };

        if let Some(form) = form {
            let is_search_mode = form.is_search_mode(field_key);
            let _has_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

            match key.code {
                KeyCode::Char('/') if !is_search_mode => {
                    form.set_search_mode(field_key.to_string(), true);
                    form.set_search_text(field_key.to_string(), String::new());
                }
                KeyCode::Esc if is_search_mode => {
                    form.clear_search(field_key);
                }
                KeyCode::Tab if is_search_mode => {
                    navigate_dropdown(form, field_key, 1);
                }
                KeyCode::BackTab if is_search_mode => {
                    navigate_dropdown(form, field_key, -1);
                }
                KeyCode::Enter if is_search_mode => {
                    handle_dropdown_enter(form, field_key, &mut self.status_message);
                }
                KeyCode::Up if is_search_mode => {
                    navigate_dropdown(form, field_key, -1);
                }
                KeyCode::Down if is_search_mode => {
                    navigate_dropdown(form, field_key, 1);
                }
                KeyCode::Up if !is_search_mode => {
                    navigate_dropdown(form, field_key, -1);
                }
                KeyCode::Down if !is_search_mode => {
                    navigate_dropdown(form, field_key, 1);
                }
                KeyCode::Char('k') if !is_search_mode => {
                    navigate_dropdown(form, field_key, -1);
                }
                KeyCode::Char('j') if !is_search_mode => {
                    navigate_dropdown(form, field_key, 1);
                }
                KeyCode::Char(c) if is_search_mode && !c.is_control() => {
                    let mut search = form.get_search_text(field_key);
                    search.push(c);
                    form.set_search_text(field_key.to_string(), search);
                }
                KeyCode::Backspace if is_search_mode => {
                    let mut search = form.get_search_text(field_key);
                    search.pop();
                    form.set_search_text(field_key.to_string(), search);
                }
                _ => {}
            }
        }
    }

    pub(crate) fn handle_progress_input(&mut self, key: KeyEvent, field_key: &str, is_update: bool) {
        let form = if is_update {
            self.update_issue_form.as_mut()
        } else {
            self.create_issue_form.as_mut()
        };

        if let Some(form) = form {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    let current = form.get_value(field_key).and_then(|v| v.as_number()).unwrap_or(0);
                    let new_val = (current + 10).min(100);
                    form.set_value(field_key.to_string(), FieldValue::Number(Some(new_val)));
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let current = form.get_value(field_key).and_then(|v| v.as_number()).unwrap_or(0);
                    let new_val = current.saturating_sub(10);
                    form.set_value(field_key.to_string(), FieldValue::Number(Some(new_val)));
                }
                _ => {}
            }
        }
    }

    pub(crate) fn handle_checkbox_input(&mut self, key: KeyEvent, field_key: &str, is_update: bool) {
        let form = if is_update {
            self.update_issue_form.as_mut()
        } else {
            self.create_issue_form.as_mut()
        };

        if let Some(form) = form {
            match key.code {
                KeyCode::Char(' ') | KeyCode::Char('x') | KeyCode::Enter => {
                    let current = form.get_value(field_key).map(|v| v.as_bool()).unwrap_or(false);
                    form.set_value(field_key.to_string(), FieldValue::Boolean(!current));
                }
                _ => {}
            }
        }
    }
}

/// Helper: Navigate through dropdown options
fn navigate_dropdown(form: &mut crate::issue_form::IssueForm, field_key: &str, direction: i32) {
    let filtered_options = form.get_filtered_options(field_key);
    if !filtered_options.is_empty() {
        let current_id = form.get_value(field_key).and_then(|v| v.as_option_id());
        let current_idx = current_id
            .and_then(|id| filtered_options.iter().position(|opt| opt.id == id))
            .unwrap_or(0);

        let new_idx = if direction > 0 {
            (current_idx + 1) % filtered_options.len()
        } else if current_idx > 0 {
            current_idx - 1
        } else {
            filtered_options.len() - 1
        };

        let new_id = filtered_options[new_idx].id;
        form.set_value(field_key.to_string(), FieldValue::OptionId(Some(new_id)));
    }
}

/// Helper: Handle Enter key in searchable dropdown
fn handle_dropdown_enter(
    form: &mut crate::issue_form::IssueForm,
    field_key: &str,
    status_message: &mut Option<String>,
) {
    let filtered_options = form.get_filtered_options(field_key);
    let matches_count = filtered_options.len();
    let current_id = form.get_value(field_key).and_then(|v| v.as_option_id());

    if matches_count == 1 {
        let selected_id = filtered_options[0].id;
        let selected_name = filtered_options[0].name.clone();
        form.set_value(field_key.to_string(), FieldValue::OptionId(Some(selected_id)));
        form.clear_search(field_key);
        *status_message = Some(format!("Selected: {}", selected_name));
    } else if matches_count > 1 && current_id.is_some() {
        let selected_name = filtered_options
            .iter()
            .find(|opt| Some(opt.id) == current_id)
            .map(|opt| opt.name.clone());

        if let Some(name) = selected_name {
            form.clear_search(field_key);
            *status_message = Some(format!("Selected: {}", name));
        } else {
            *status_message = Some(format!(
                "{} matches - ↑↓ or Tab to navigate, Enter to select",
                matches_count
            ));
        }
    } else {
        *status_message = Some(format!(
            "{} matches - ↑↓ or Tab to navigate, Enter to select",
            matches_count
        ));
    }
}
