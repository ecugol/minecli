use anyhow::{Context, Result};
use reqwest::Client;
use serde::de::DeserializeOwned;

use super::models::*;

#[derive(Clone)]
pub struct RedmineClient {
    base_url: String,
    api_key: String,
    client: Client,
}

impl RedmineClient {
    pub fn new(base_url: String, api_key: String) -> Self {
        let client = Client::new();
        Self {
            base_url,
            api_key,
            client,
        }
    }

    fn build_url(&self, path: &str) -> String {
        format!("{}/{}", self.base_url.trim_end_matches('/'), path)
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = self.build_url(path);
        let response = self
            .client
            .get(&url)
            .header("X-Redmine-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .send()
            .await
            .context("Failed to send GET request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, error_text);
        }

        // Try to parse JSON, but if it fails, show the actual response
        let response_text = response.text().await.context("Failed to read response body")?;
        serde_json::from_str(&response_text).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse JSON response from {}\nSerde error: {}\nResponse body (first 2000 chars): {}",
                path,
                e,
                &response_text.chars().take(2000).collect::<String>()
            )
        })
    }

    async fn post<T: DeserializeOwned>(&self, path: &str, body: &impl serde::Serialize) -> Result<T> {
        let url = self.build_url(path);
        let response = self
            .client
            .post(&url)
            .header("X-Redmine-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .context("Failed to send POST request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, error_text);
        }

        // Try to parse JSON, but if it fails, show the actual response
        let response_text = response.text().await.context("Failed to read response body")?;
        serde_json::from_str(&response_text).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse JSON response from POST {}\nSerde error: {}\nResponse body (first 2000 chars): {}",
                path,
                e,
                &response_text.chars().take(2000).collect::<String>()
            )
        })
    }

    async fn put(&self, path: &str, body: &impl serde::Serialize) -> Result<()> {
        let url = self.build_url(path);
        let response = self
            .client
            .put(&url)
            .header("X-Redmine-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .context("Failed to send PUT request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, error_text);
        }

        Ok(())
    }

    pub async fn get_projects(&self, limit: u32, offset: u32) -> Result<ProjectsResponse> {
        self.get(&format!("projects.json?limit={}&offset={}", limit, offset))
            .await
    }

    pub async fn get_issues(
        &self,
        project_id: Option<u64>,
        status_id: Option<&str>,
        limit: u32,
        offset: u32,
        exclude_subprojects: bool,
    ) -> Result<IssuesResponse> {
        let mut query = format!("limit={}&offset={}", limit, offset);

        if let Some(pid) = project_id {
            query.push_str(&format!("&project_id={}", pid));
            
            // Exclude subproject issues if requested (only get issues from this project)
            if exclude_subprojects {
                query.push_str("&subproject_id=!*");
            }
        }

        if let Some(sid) = status_id {
            query.push_str(&format!("&status_id={}", sid));
        }

        // Note: attachments are not available in the issues list endpoint
        // They can only be fetched when getting a single issue

        self.get(&format!("issues.json?{}", query)).await
    }

    pub async fn get_issue(&self, issue_id: u64) -> Result<IssueWrapper> {
        self.get(&format!("issues/{}.json?include=journals,attachments", issue_id))
            .await
    }

    pub async fn create_issue(&self, issue: CreateIssue) -> Result<IssueWrapper> {
        let wrapper = CreateIssueWrapper { issue };
        self.post("issues.json", &wrapper).await
    }

    pub async fn update_issue(&self, issue_id: u64, issue: UpdateIssue) -> Result<()> {
        let wrapper = UpdateIssueWrapper { issue };
        self.put(&format!("issues/{}.json", issue_id), &wrapper).await
    }

    pub async fn get_trackers(&self) -> Result<TrackersResponse> {
        self.get("trackers.json").await
    }

    pub async fn get_issue_statuses(&self) -> Result<IssueStatusesResponse> {
        self.get("issue_statuses.json").await
    }

    pub async fn get_priorities(&self) -> Result<PrioritiesResponse> {
        self.get("enumerations/issue_priorities.json").await
    }

    pub async fn test_connection(&self) -> Result<()> {
        self.get_projects(1, 0).await?;
        Ok(())
    }

    pub async fn get_current_user(&self) -> Result<UserWrapper> {
        self.get("users/current.json").await
    }

    pub async fn get_users(&self, limit: u32, offset: u32) -> Result<UsersResponse> {
        self.get(&format!("users.json?limit={}&offset={}", limit, offset)).await
    }

    pub async fn get_project_memberships(&self, project_id: u64) -> Result<MembershipsResponse> {
        // Fetch all memberships with pagination to ensure we get all assignees
        let limit = 100;
        let mut offset = 0;
        let mut all_memberships = Vec::new();
        
        loop {
            let response: MembershipsResponse = self
                .get(&format!(
                    "projects/{}/memberships.json?limit={}&offset={}",
                    project_id, limit, offset
                ))
                .await?;
            
            let count = response.memberships.len();
            all_memberships.extend(response.memberships);
            
            // Check if we've fetched all memberships
            if count < limit as usize {
                break;
            }
            
            offset += limit;
        }
        
        Ok(MembershipsResponse {
            memberships: all_memberships,
            total_count: None,
            offset: None,
            limit: None,
        })
    }

    pub async fn update_issue_with_comment(&self, issue_id: u64, update: UpdateIssue) -> Result<()> {
        let wrapper = UpdateIssueWrapper { issue: update };
        self.put(&format!("issues/{}.json", issue_id), &wrapper).await
    }

    pub async fn get_project_detail(&self, project_id: u64) -> Result<ProjectDetailWrapper> {
        self.get(&format!(
            "projects/{}.json?include=trackers,issue_categories,issue_custom_fields",
            project_id
        ))
        .await
    }

    /// Fetch a sample issue for a specific tracker to discover its custom fields
    pub async fn get_sample_issue_for_tracker(&self, tracker_id: u64) -> Result<Option<Issue>> {
        let response: IssuesResponse = self
            .get(&format!("issues.json?tracker_id={}&limit=1", tracker_id))
            .await?;

        Ok(response.issues.into_iter().next())
    }

    /// Download an attachment from Redmine
    pub async fn download_attachment(&self, url: &str) -> Result<Vec<u8>> {
        let response = self
            .client
            .get(url)
            .header("X-Redmine-API-Key", &self.api_key)
            .send()
            .await
            .context("Failed to download attachment")?;

        if !response.status().is_success() {
            let status = response.status();
            anyhow::bail!("Failed to download attachment with status {}", status);
        }

        let bytes = response.bytes().await.context("Failed to read attachment bytes")?;

        Ok(bytes.to_vec())
    }

    /// Get the latest updated issue for a project
    pub async fn get_latest_issue_for_project(&self, project_id: u64) -> Result<Option<Issue>> {
        let response: IssuesResponse = self
            .get(&format!(
                "issues.json?project_id={}&limit=1&sort=updated_on:desc",
                project_id
            ))
            .await?;

        Ok(response.issues.into_iter().next())
    }

    /// Get recently updated issues across all projects (sorted by updated_on descending)
    pub async fn get_recent_issues(&self, limit: u32, offset: u32) -> Result<IssuesResponse> {
        self.get(&format!(
            "issues.json?limit={}&offset={}&sort=updated_on:desc",
            limit, offset
        ))
        .await
    }

    /// Upload a file and get an upload token
    pub async fn upload_file(&self, filename: &str, content: Vec<u8>) -> Result<UploadResponse> {
        let url = self.build_url("uploads.json");
        
        let response = self
            .client
            .post(&url)
            .header("X-Redmine-API-Key", &self.api_key)
            .header("Content-Type", "application/octet-stream")
            .query(&[("filename", filename)])
            .body(content)
            .send()
            .await
            .context("Failed to upload file")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("File upload failed with status {}: {}", status, error_text);
        }

        let response_text = response.text().await.context("Failed to read upload response")?;
        serde_json::from_str(&response_text).with_context(|| {
            format!(
                "Failed to parse upload response\nResponse: {}",
                &response_text.chars().take(500).collect::<String>()
            )
        })
    }
}
