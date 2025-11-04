use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: u64,
    pub name: String,
    pub identifier: String,
    pub description: Option<String>,
    pub status: Option<u32>,
    pub parent: Option<IdName>,
    pub created_on: Option<DateTime<Utc>>,
    pub updated_on: Option<DateTime<Utc>>,
    #[serde(skip_deserializing, default)]
    pub last_issue_activity: Option<DateTime<Utc>>,
    #[serde(skip_deserializing, default)]
    pub last_issues_sync: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectsResponse {
    pub projects: Vec<Project>,
    pub total_count: Option<u32>,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: u64,
    pub project: IdName,
    pub tracker: IdName,
    pub status: IdName,
    pub priority: IdName,
    pub author: IdName,
    pub assigned_to: Option<IdName>,
    #[serde(default)]
    pub parent: Option<IdName>,
    #[serde(default)]
    pub category: Option<IdName>,
    #[serde(default)]
    pub fixed_version: Option<IdName>,
    pub subject: String,
    pub description: Option<String>,
    pub start_date: Option<String>,
    pub due_date: Option<String>,
    pub done_ratio: Option<u32>,
    pub is_private: Option<bool>,
    pub estimated_hours: Option<f32>,
    #[serde(default)]
    pub total_estimated_hours: Option<f32>,
    #[serde(default)]
    pub spent_hours: Option<f32>,
    #[serde(default)]
    pub total_spent_hours: Option<f32>,
    pub created_on: DateTime<Utc>,
    pub updated_on: DateTime<Utc>,
    pub closed_on: Option<DateTime<Utc>>,
    #[serde(default)]
    pub journals: Vec<Journal>,
    #[serde(default)]
    pub custom_fields: Vec<IssueCustomField>,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Journal {
    pub id: u64,
    pub user: IdName,
    pub notes: Option<String>,
    pub created_on: DateTime<Utc>,
    #[serde(default)]
    pub private_notes: bool,
    #[serde(default)]
    pub details: Vec<JournalDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalDetail {
    pub property: String,
    pub name: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: u64,
    pub filename: String,
    pub filesize: u64,
    pub content_type: Option<String>,
    #[serde(default)]
    pub description: String,
    pub content_url: String,
    #[serde(default)]
    pub thumbnail_url: Option<String>,
    pub author: IdName,
    pub created_on: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdName {
    pub id: u64,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuesResponse {
    pub issues: Vec<Issue>,
    pub total_count: Option<u32>,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueWrapper {
    pub issue: Issue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Upload {
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResponse {
    pub upload: UploadToken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadToken {
    pub id: u64,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIssue {
    pub project_id: u64,
    pub tracker_id: u64,
    pub status_id: u64,
    pub priority_id: u64,
    pub subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_to_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_hours: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done_ratio: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uploads: Option<Vec<Upload>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIssueWrapper {
    pub issue: CreateIssue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateIssue {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_to_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done_ratio: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_hours: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_notes: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uploads: Option<Vec<Upload>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateIssueWrapper {
    pub issue: UpdateIssue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub login: String,
    pub firstname: String,
    pub lastname: String,
    pub mail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tracker {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackersResponse {
    pub trackers: Vec<Tracker>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueStatus {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueStatusesResponse {
    pub issue_statuses: Vec<IssueStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Priority {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritiesResponse {
    pub issue_priorities: Vec<Priority>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserWrapper {
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersResponse {
    pub users: Vec<User>,
    pub total_count: Option<u32>,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMembership {
    pub id: u64,
    pub project: IdName,
    pub user: Option<IdName>,
    pub roles: Vec<IdName>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MembershipsResponse {
    pub memberships: Vec<ProjectMembership>,
    pub total_count: Option<u32>,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueCategory {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueCustomField {
    pub id: u64,
    pub name: String,
    #[serde(deserialize_with = "deserialize_null_to_empty_string")]
    pub value: String,
}

// Helper function to deserialize custom field value (can be null, string, or array)
fn deserialize_null_to_empty_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    use serde_json::Value;

    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Null => Ok(String::new()),
        Value::String(s) => Ok(s),
        Value::Array(arr) => {
            // Join array values with comma
            Ok(arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "))
        }
        _ => Ok(value.to_string()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomField {
    pub id: u64,
    pub name: String,
    #[serde(rename = "customized_type")]
    pub customized_type: Option<String>,
    #[serde(rename = "field_format")]
    pub field_format: Option<String>,
    pub possible_values: Option<Vec<CustomFieldValue>>,
    pub multiple: Option<bool>,
    pub required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFieldValue {
    pub value: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDetail {
    pub id: u64,
    pub name: String,
    pub identifier: String,
    pub description: Option<String>,
    pub status: Option<u32>,
    pub created_on: Option<DateTime<Utc>>,
    pub updated_on: Option<DateTime<Utc>>,
    #[serde(default)]
    pub trackers: Vec<Tracker>,
    #[serde(default)]
    pub issue_categories: Vec<IssueCategory>,
    #[serde(default)]
    pub custom_fields: Vec<CustomField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDetailWrapper {
    pub project: ProjectDetail,
}
