use serde::{Deserialize, Serialize};

/// Represents a form field type
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    Text,               // Single line text input
    TextArea,           // Multi-line text input
    Dropdown,           // Select from list
    SearchableDropdown, // Dropdown with search (like assignee)
    Date,               // Date input (YYYY-MM-DD)
    #[allow(dead_code)]
    Number, // Integer input
    Float,              // Decimal input
    Checkbox,           // Boolean toggle
    Progress,           // Progress bar (0-100%)
}

/// Represents a single option in a dropdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldOption {
    pub id: u64,
    pub name: String,
}

/// Represents the current value of a field
#[derive(Debug, Clone)]
pub enum FieldValue {
    Text(String),
    Number(Option<u32>),
    Float(Option<f32>),
    Boolean(bool),
    OptionId(Option<u64>), // For dropdowns - stores the selected ID
}

impl FieldValue {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            FieldValue::Text(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<u32> {
        match self {
            FieldValue::Number(n) => *n,
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f32> {
        match self {
            FieldValue::Float(f) => *f,
            _ => None,
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            FieldValue::Boolean(b) => *b,
            _ => false,
        }
    }

    pub fn as_option_id(&self) -> Option<u64> {
        match self {
            FieldValue::OptionId(id) => *id,
            _ => None,
        }
    }
}

/// Represents a form field definition
#[derive(Debug, Clone)]
pub struct FormField {
    pub key: String,   // Field identifier (e.g., "subject", "tracker_id")
    pub label: String, // Display label
    pub field_type: FieldType,
    pub required: bool,
    pub options: Vec<FieldOption>, // For dropdown fields
    pub default_value: Option<FieldValue>,
    pub help_text: Option<String>,
}

impl FormField {
    pub fn new_text(key: &str, label: &str, required: bool) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            field_type: FieldType::Text,
            required,
            options: Vec::new(),
            default_value: Some(FieldValue::Text(String::new())),
            help_text: None,
        }
    }

    pub fn new_textarea(key: &str, label: &str, required: bool) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            field_type: FieldType::TextArea,
            required,
            options: Vec::new(),
            default_value: Some(FieldValue::Text(String::new())),
            help_text: None,
        }
    }

    pub fn new_dropdown(key: &str, label: &str, options: Vec<FieldOption>, required: bool) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            field_type: FieldType::Dropdown,
            required,
            options,
            default_value: Some(FieldValue::OptionId(None)),
            help_text: None,
        }
    }

    pub fn new_searchable_dropdown(key: &str, label: &str, options: Vec<FieldOption>, required: bool) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            field_type: FieldType::SearchableDropdown,
            required,
            options,
            default_value: Some(FieldValue::OptionId(None)),
            help_text: None,
        }
    }

    pub fn new_date(key: &str, label: &str, required: bool) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            field_type: FieldType::Date,
            required,
            options: Vec::new(),
            default_value: Some(FieldValue::Text(String::new())),
            help_text: Some("Format: YYYY-MM-DD".to_string()),
        }
    }

    pub fn new_number(key: &str, label: &str, required: bool) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            field_type: FieldType::Number,
            required,
            options: Vec::new(),
            default_value: Some(FieldValue::Number(None)),
            help_text: None,
        }
    }

    pub fn new_float(key: &str, label: &str, required: bool) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            field_type: FieldType::Float,
            required,
            options: Vec::new(),
            default_value: Some(FieldValue::Float(None)),
            help_text: None,
        }
    }

    pub fn new_checkbox(key: &str, label: &str) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            field_type: FieldType::Checkbox,
            required: false,
            options: Vec::new(),
            default_value: Some(FieldValue::Boolean(false)),
            help_text: None,
        }
    }

    pub fn new_progress(key: &str, label: &str) -> Self {
        Self {
            key: key.to_string(),
            label: label.to_string(),
            field_type: FieldType::Progress,
            required: false,
            options: Vec::new(),
            default_value: Some(FieldValue::Number(Some(0))),
            help_text: Some("0-100%".to_string()),
        }
    }

    pub fn with_help_text(mut self, help: &str) -> Self {
        self.help_text = Some(help.to_string());
        self
    }

    pub fn with_default(mut self, value: FieldValue) -> Self {
        self.default_value = Some(value);
        self
    }
}
