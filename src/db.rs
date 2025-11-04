use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;

use crate::app::IssueSortOrder;
use crate::redmine::{Issue, Journal, Project};

/// Helper function to safely parse datetime from database
/// Returns a proper error instead of panicking
fn parse_datetime_from_db(s: &str) -> Result<DateTime<Utc>, rusqlite::Error> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(db_path).context("Failed to open database")?;

        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        // Projects table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                identifier TEXT NOT NULL,
                description TEXT,
                status INTEGER,
                parent_id INTEGER,
                parent_name TEXT,
                created_on TEXT,
                updated_on TEXT,
                last_issue_activity TEXT,
                last_issues_sync TEXT
            )",
            [],
        )?;

        // Migration: Add last_issue_activity column if it doesn't exist
        let _ = self
            .conn
            .execute("ALTER TABLE projects ADD COLUMN last_issue_activity TEXT", []);

        // Migration: Add last_issues_sync column if it doesn't exist
        let _ = self
            .conn
            .execute("ALTER TABLE projects ADD COLUMN last_issues_sync TEXT", []);

        // Migration: Add parent_id column if it doesn't exist
        let _ = self
            .conn
            .execute("ALTER TABLE projects ADD COLUMN parent_id INTEGER", []);

        // Migration: Add parent_name column if it doesn't exist
        let _ = self
            .conn
            .execute("ALTER TABLE projects ADD COLUMN parent_name TEXT", []);

        // Issues table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS issues (
                id INTEGER PRIMARY KEY,
                project_id INTEGER NOT NULL,
                tracker_id INTEGER NOT NULL,
                tracker_name TEXT NOT NULL,
                status_id INTEGER NOT NULL,
                status_name TEXT NOT NULL,
                priority_id INTEGER NOT NULL,
                priority_name TEXT NOT NULL,
                author_id INTEGER NOT NULL,
                author_name TEXT NOT NULL,
                assigned_to_id INTEGER,
                assigned_to_name TEXT,
                subject TEXT NOT NULL,
                description TEXT,
                created_on TEXT NOT NULL,
                updated_on TEXT NOT NULL,
                due_date TEXT,
                done_ratio INTEGER
            )",
            [],
        )?;

        // Users table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY,
                login TEXT NOT NULL,
                firstname TEXT NOT NULL,
                lastname TEXT NOT NULL,
                mail TEXT,
                cached_at TEXT NOT NULL
            )",
            [],
        )?;

        // Journals table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS journals (
                id INTEGER PRIMARY KEY,
                issue_id INTEGER NOT NULL,
                user_id INTEGER NOT NULL,
                user_name TEXT NOT NULL,
                notes TEXT,
                created_on TEXT NOT NULL,
                FOREIGN KEY(issue_id) REFERENCES issues(id)
            )",
            [],
        )?;

        // Journal details table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS journal_details (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                journal_id INTEGER NOT NULL,
                property TEXT NOT NULL,
                name TEXT NOT NULL,
                old_value TEXT,
                new_value TEXT,
                FOREIGN KEY(journal_id) REFERENCES journals(id)
            )",
            [],
        )?;

        // Metadata table for tracking last sync times
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        // Create indexes for performance
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_projects_updated ON projects(updated_on DESC)",
            [],
        )?;
        self.conn
            .execute("CREATE INDEX IF NOT EXISTS idx_projects_name ON projects(name)", [])?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_issues_project ON issues(project_id)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_issues_updated ON issues(updated_on DESC)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_issues_status ON issues(status_name)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_issues_priority ON issues(priority_id)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_issues_assigned ON issues(assigned_to_id)",
            [],
        )?;
        self.conn
            .execute("CREATE INDEX IF NOT EXISTS idx_issues_subject ON issues(subject)", [])?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_issues_created ON issues(created_on DESC)",
            [],
        )?;
        // Composite index for common query pattern (project + updated)
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_issues_project_updated ON issues(project_id, updated_on DESC)",
            [],
        )?;
        // Composite index for filtering by assignee within project
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_issues_project_assigned ON issues(project_id, assigned_to_id)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_journals_issue ON journals(issue_id)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_journal_details_journal ON journal_details(journal_id)",
            [],
        )?;
        self.conn
            .execute("CREATE INDEX IF NOT EXISTS idx_users_login ON users(login)", [])?;

        Ok(())
    }

    // Projects
    pub fn insert_projects(&self, projects: &[Project]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;

        for project in projects {
            // First, get existing last_issue_activity and last_issues_sync to preserve them
            let existing: Option<(Option<String>, Option<String>)> = tx
                .query_row(
                    "SELECT last_issue_activity, last_issues_sync FROM projects WHERE id = ?1",
                    params![project.id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()?;

            let (preserve_activity, preserve_sync) = existing.unwrap_or((None, None));

            tx.execute(
                "INSERT OR REPLACE INTO projects
                (id, name, identifier, description, status, parent_id, parent_name, created_on, updated_on, last_issue_activity, last_issues_sync)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    project.id,
                    &project.name,
                    &project.identifier,
                    &project.description,
                    project.status,
                    project.parent.as_ref().map(|p| p.id),
                    project.parent.as_ref().map(|p| &p.name),
                    project.created_on.as_ref().map(|d| d.to_rfc3339()),
                    project.updated_on.as_ref().map(|d| d.to_rfc3339()),
                    preserve_activity,  // Preserve existing value
                    preserve_sync,      // Preserve existing value
                ],
            )?;
        }

        // Record sync timestamp
        let now = Utc::now().to_rfc3339();
        tx.execute(
            "INSERT OR REPLACE INTO metadata (key, value) VALUES ('projects_last_synced', ?1)",
            params![now],
        )?;

        // After inserting projects, update last_issue_activity for all projects based on their issues
        // This ensures projects are sorted correctly even if issues were loaded before projects were refreshed
        tx.execute(
            "UPDATE projects SET last_issue_activity = (
                SELECT MAX(updated_on) FROM issues WHERE issues.project_id = projects.id
            )",
            [],
        )?;

        tx.commit()?;
        Ok(())
    }

    // Users
    pub fn insert_users(&self, users: &[crate::redmine::User]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;

        for user in users {
            tx.execute(
                "INSERT OR REPLACE INTO users
                (id, login, firstname, lastname, mail, cached_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    user.id,
                    &user.login,
                    &user.firstname,
                    &user.lastname,
                    &user.mail,
                    Utc::now().to_rfc3339(),
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn get_users(&self) -> Result<Vec<crate::redmine::User>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, login, firstname, lastname, mail FROM users ORDER BY lastname, firstname")?;

        let users = stmt
            .query_map([], |row| {
                Ok(crate::redmine::User {
                    id: row.get(0)?,
                    login: row.get(1)?,
                    firstname: row.get(2)?,
                    lastname: row.get(3)?,
                    mail: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(users)
    }

    pub fn get_projects(&self, filter: Option<&str>) -> Result<Vec<Project>> {
        let mut query = String::from(
            "SELECT id, name, identifier, description, status, parent_id, parent_name, created_on, updated_on, last_issue_activity, last_issues_sync
             FROM projects"
        );

        let mut params: Vec<String> = Vec::new();

        if let Some(f) = filter {
            if !f.is_empty() {
                query.push_str(" WHERE (name LIKE ?1 OR identifier LIKE ?1 OR description LIKE ?1)");
                params.push(format!("%{}%", f));
            }
        }

        // Sort by last issue activity (most recent first), fallback to updated_on, then created_on
        // Use '1970-01-01T00:00:00Z' for proper RFC3339 format comparison
        query.push_str(" ORDER BY COALESCE(last_issue_activity, updated_on, created_on, '1970-01-01T00:00:00Z') DESC");

        let mut stmt = self.conn.prepare(&query)?;

        let project_iter = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                identifier: row.get(2)?,
                description: row.get(3)?,
                status: row.get(4)?,
                parent: {
                    let parent_id: Option<u64> = row.get(5)?;
                    let parent_name: Option<String> = row.get(6)?;
                    match (parent_id, parent_name) {
                        (Some(id), Some(name)) => Some(crate::redmine::IdName { id, name }),
                        _ => None,
                    }
                },
                created_on: row
                    .get::<_, Option<String>>(7)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                updated_on: row
                    .get::<_, Option<String>>(8)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                last_issue_activity: row
                    .get::<_, Option<String>>(9)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                last_issues_sync: row
                    .get::<_, Option<String>>(10)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
            })
        })?;

        let mut projects = Vec::new();
        for project in project_iter {
            projects.push(project?);
        }

        Ok(projects)
    }

    // Issues
    pub fn clear_project_issues(&self, project_id: u64) -> Result<()> {
        // Delete journals and their details first
        self.conn.execute(
            "DELETE FROM journal_details WHERE journal_id IN (
                SELECT j.id FROM journals j
                JOIN issues i ON j.issue_id = i.id
                WHERE i.project_id = ?1
            )",
            params![project_id],
        )?;

        self.conn.execute(
            "DELETE FROM journals WHERE issue_id IN (
                SELECT id FROM issues WHERE project_id = ?1
            )",
            params![project_id],
        )?;

        self.conn
            .execute("DELETE FROM issues WHERE project_id = ?1", params![project_id])?;

        Ok(())
    }

    pub fn insert_issues(&self, issues: &[Issue]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;

        for issue in issues {
            tx.execute(
                "INSERT OR REPLACE INTO issues
                (id, project_id, tracker_id, tracker_name, status_id, status_name,
                 priority_id, priority_name, author_id, author_name,
                 assigned_to_id, assigned_to_name, subject, description,
                 created_on, updated_on, due_date, done_ratio)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                params![
                    issue.id,
                    issue.project.id,
                    issue.tracker.id,
                    &issue.tracker.name,
                    issue.status.id,
                    &issue.status.name,
                    issue.priority.id,
                    &issue.priority.name,
                    issue.author.id,
                    &issue.author.name,
                    issue.assigned_to.as_ref().map(|a| a.id),
                    issue.assigned_to.as_ref().map(|a| &a.name),
                    &issue.subject,
                    &issue.description,
                    issue.created_on.to_rfc3339(),
                    issue.updated_on.to_rfc3339(),
                    &issue.due_date,
                    issue.done_ratio,
                ],
            )?;
        }

        // Update project's last_issue_activity based on the most recent issue from ALL issues (not just this batch)
        // Get unique project IDs from this batch
        let project_ids: std::collections::HashSet<u64> = issues.iter().map(|i| i.project.id).collect();
        
        for project_id in &project_ids {
            // Find the most recently updated issue for this project in the database
            let max_updated: Option<String> = tx.query_row(
                "SELECT MAX(updated_on) FROM issues WHERE project_id = ?1",
                params![project_id],
                |row| row.get(0),
            ).optional()?;

            if let Some(max_updated_str) = max_updated {
                tx.execute(
                    "UPDATE projects SET last_issue_activity = ?1 WHERE id = ?2",
                    params![max_updated_str, project_id],
                )?;
            }
        }

        // Record per-project sync timestamp
        let now = Utc::now().to_rfc3339();
        for project_id in &project_ids {
            tx.execute(
                "UPDATE projects SET last_issues_sync = ?1 WHERE id = ?2",
                params![now, project_id],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn get_issues(
        &self,
        project_id: Option<u64>,
        sort_order: IssueSortOrder,
        filter: Option<&str>,
        assigned_to_id: Option<u64>,
    ) -> Result<Vec<Issue>> {
        let mut query = String::from(
            "SELECT id, project_id, tracker_id, tracker_name, status_id, status_name,
             priority_id, priority_name, author_id, author_name,
             assigned_to_id, assigned_to_name, subject, description,
             created_on, updated_on, due_date, done_ratio
             FROM issues WHERE 1=1",
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(pid) = project_id {
            query.push_str(" AND project_id = ?");
            params.push(Box::new(pid));
        }

        if let Some(assigned_id) = assigned_to_id {
            query.push_str(" AND assigned_to_id = ?");
            params.push(Box::new(assigned_id));
        }

        if let Some(f) = filter {
            if !f.is_empty() {
                query.push_str(" AND (subject LIKE ? OR description LIKE ? OR CAST(id AS TEXT) LIKE ?)");
                let filter_param = format!("%{}%", f);
                params.push(Box::new(filter_param.clone()));
                params.push(Box::new(filter_param.clone()));
                params.push(Box::new(filter_param));
            }
        }

        // Add ORDER BY based on sort order
        match sort_order {
            IssueSortOrder::UpdatedDesc => query.push_str(" ORDER BY updated_on DESC"),
            IssueSortOrder::StatusAsc => query.push_str(" ORDER BY status_name ASC"),
            IssueSortOrder::StatusDesc => query.push_str(" ORDER BY status_name DESC"),
            IssueSortOrder::PriorityAsc => query.push_str(" ORDER BY priority_id ASC"),
            IssueSortOrder::PriorityDesc => query.push_str(" ORDER BY priority_id DESC"),
        }

        let mut stmt = self.conn.prepare(&query)?;

        let params_ref: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();

        let issue_iter = stmt.query_map(params_ref.as_slice(), |row| {
            Ok(Issue {
                id: row.get(0)?,
                project: crate::redmine::IdName {
                    id: row.get(1)?,
                    name: String::new(), // Will be populated if needed
                },
                tracker: crate::redmine::IdName {
                    id: row.get(2)?,
                    name: row.get(3)?,
                },
                status: crate::redmine::IdName {
                    id: row.get(4)?,
                    name: row.get(5)?,
                },
                priority: crate::redmine::IdName {
                    id: row.get(6)?,
                    name: row.get(7)?,
                },
                author: crate::redmine::IdName {
                    id: row.get(8)?,
                    name: row.get(9)?,
                },
                assigned_to: {
                    let id: Option<u64> = row.get(10)?;
                    let name: Option<String> = row.get(11)?;
                    match (id, name) {
                        (Some(id), Some(name)) => Some(crate::redmine::IdName { id, name }),
                        _ => None,
                    }
                },
                parent: None, // Not stored in DB yet
                category: None, // Not stored in DB yet
                fixed_version: None, // Not stored in DB yet
                subject: row.get(12)?,
                description: row.get(13)?,
                start_date: None, // Not stored in DB yet
                due_date: row.get(16)?,
                done_ratio: row.get(17)?,
                is_private: None,      // Not stored in DB yet
                estimated_hours: None, // Not stored in DB yet
                total_estimated_hours: None, // Not stored in DB yet
                spent_hours: None, // Not stored in DB yet
                total_spent_hours: None, // Not stored in DB yet
                created_on: parse_datetime_from_db(&row.get::<_, String>(14)?)?,
                updated_on: parse_datetime_from_db(&row.get::<_, String>(15)?)?,
                closed_on: None,           // Not stored in DB yet
                journals: Vec::new(),      // Loaded separately
                custom_fields: Vec::new(), // Not stored in DB
                attachments: Vec::new(),   // Not stored in DB
            })
        })?;

        let mut issues = Vec::new();
        for issue in issue_iter {
            issues.push(issue?);
        }

        Ok(issues)
    }

    // Issue with journals
    pub fn insert_issue_with_journals(&self, issue: &Issue) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;

        // Insert the issue
        tx.execute(
            "INSERT OR REPLACE INTO issues
            (id, project_id, tracker_id, tracker_name, status_id, status_name,
             priority_id, priority_name, author_id, author_name,
             assigned_to_id, assigned_to_name, subject, description,
             created_on, updated_on, due_date, done_ratio)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
            params![
                issue.id,
                issue.project.id,
                issue.tracker.id,
                &issue.tracker.name,
                issue.status.id,
                &issue.status.name,
                issue.priority.id,
                &issue.priority.name,
                issue.author.id,
                &issue.author.name,
                issue.assigned_to.as_ref().map(|a| a.id),
                issue.assigned_to.as_ref().map(|a| &a.name),
                &issue.subject,
                &issue.description,
                issue.created_on.to_rfc3339(),
                issue.updated_on.to_rfc3339(),
                &issue.due_date,
                issue.done_ratio,
            ],
        )?;

        // Delete old journals for this issue
        tx.execute(
            "DELETE FROM journal_details WHERE journal_id IN (
                SELECT id FROM journals WHERE issue_id = ?1
            )",
            params![issue.id],
        )?;
        tx.execute("DELETE FROM journals WHERE issue_id = ?1", params![issue.id])?;

        // Insert journals
        for journal in &issue.journals {
            tx.execute(
                "INSERT INTO journals (id, issue_id, user_id, user_name, notes, created_on)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    journal.id,
                    issue.id,
                    journal.user.id,
                    &journal.user.name,
                    &journal.notes,
                    journal.created_on.to_rfc3339(),
                ],
            )?;

            // Insert journal details
            for detail in &journal.details {
                tx.execute(
                    "INSERT INTO journal_details (journal_id, property, name, old_value, new_value)
                    VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        journal.id,
                        &detail.property,
                        &detail.name,
                        &detail.old_value,
                        &detail.new_value,
                    ],
                )?;
            }
        }

        // Update project's last_issue_activity and last_issues_sync
        let now = Utc::now().to_rfc3339();
        tx.execute(
            "UPDATE projects SET last_issue_activity = ?1, last_issues_sync = ?2 WHERE id = ?3",
            params![issue.updated_on.to_rfc3339(), now, issue.project.id],
        )?;

        tx.commit()?;
        Ok(())
    }

    pub fn get_issue_with_journals(&self, issue_id: u64) -> Result<Option<Issue>> {
        // Get the issue
        let issue = self
            .conn
            .query_row(
                "SELECT id, project_id, tracker_id, tracker_name, status_id, status_name,
             priority_id, priority_name, author_id, author_name,
             assigned_to_id, assigned_to_name, subject, description,
             created_on, updated_on, due_date, done_ratio
             FROM issues WHERE id = ?1",
                params![issue_id],
                |row| {
                    Ok(Issue {
                        id: row.get(0)?,
                        project: crate::redmine::IdName {
                            id: row.get(1)?,
                            name: String::new(),
                        },
                        tracker: crate::redmine::IdName {
                            id: row.get(2)?,
                            name: row.get(3)?,
                        },
                        status: crate::redmine::IdName {
                            id: row.get(4)?,
                            name: row.get(5)?,
                        },
                        priority: crate::redmine::IdName {
                            id: row.get(6)?,
                            name: row.get(7)?,
                        },
                        author: crate::redmine::IdName {
                            id: row.get(8)?,
                            name: row.get(9)?,
                        },
                        assigned_to: {
                            let id: Option<u64> = row.get(10)?;
                            let name: Option<String> = row.get(11)?;
                            match (id, name) {
                                (Some(id), Some(name)) => Some(crate::redmine::IdName { id, name }),
                                _ => None,
                            }
                        },
                        parent: None, // Not stored in DB yet
                        category: None, // Not stored in DB yet
                        fixed_version: None, // Not stored in DB yet
                        subject: row.get(12)?,
                        description: row.get(13)?,
                        start_date: None, // Not stored in DB yet
                        due_date: row.get(16)?,
                        done_ratio: row.get(17)?,
                        is_private: None,      // Not stored in DB yet
                        estimated_hours: None, // Not stored in DB yet
                        total_estimated_hours: None, // Not stored in DB yet
                        spent_hours: None, // Not stored in DB yet
                        total_spent_hours: None, // Not stored in DB yet
                        created_on: parse_datetime_from_db(&row.get::<_, String>(14)?)?,
                        updated_on: parse_datetime_from_db(&row.get::<_, String>(15)?)?,
                        closed_on: None, // Not stored in DB yet
                        journals: Vec::new(),
                        custom_fields: Vec::new(), // Not stored in DB
                        attachments: Vec::new(),   // Not stored in DB
                    })
                },
            )
            .optional()?;

        if let Some(mut issue) = issue {
            // Get journals
            let mut stmt = self.conn.prepare(
                "SELECT id, user_id, user_name, notes, created_on
                 FROM journals WHERE issue_id = ?1
                 ORDER BY created_on ASC",
            )?;

            let journal_iter = stmt.query_map(params![issue_id], |row| {
                Ok((
                    row.get::<_, u64>(0)?,
                    Journal {
                        id: row.get(0)?,
                        user: crate::redmine::IdName {
                            id: row.get(1)?,
                            name: row.get(2)?,
                        },
                        notes: row.get(3)?,
                        created_on: parse_datetime_from_db(&row.get::<_, String>(4)?)?,
                        private_notes: false, // Not stored in DB yet
                        details: Vec::new(),
                    },
                ))
            })?;

            for journal_result in journal_iter {
                let (journal_id, mut journal) = journal_result?;

                // Get journal details
                let mut detail_stmt = self.conn.prepare(
                    "SELECT property, name, old_value, new_value
                     FROM journal_details WHERE journal_id = ?1",
                )?;

                let detail_iter = detail_stmt.query_map(params![journal_id], |row| {
                    Ok(crate::redmine::JournalDetail {
                        property: row.get(0)?,
                        name: row.get(1)?,
                        old_value: row.get(2)?,
                        new_value: row.get(3)?,
                    })
                })?;

                for detail in detail_iter {
                    journal.details.push(detail?);
                }

                issue.journals.push(journal);
            }

            Ok(Some(issue))
        } else {
            Ok(None)
        }
    }

    pub fn get_project_name(&self, project_id: u64) -> Result<Option<String>> {
        self.conn
            .query_row("SELECT name FROM projects WHERE id = ?1", params![project_id], |row| {
                row.get(0)
            })
            .optional()
            .map_err(Into::into)
    }

    // Metadata
    pub fn get_last_projects_sync(&self) -> Result<Option<DateTime<Utc>>> {
        let result: Option<String> = self
            .conn
            .query_row(
                "SELECT value FROM metadata WHERE key = 'projects_last_synced'",
                [],
                |row| row.get(0),
            )
            .optional()?;

        Ok(result
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc)))
    }

    pub fn get_last_issues_sync(&self) -> Result<Option<DateTime<Utc>>> {
        let result: Option<String> = self
            .conn
            .query_row(
                "SELECT value FROM metadata WHERE key = 'issues_last_synced'",
                [],
                |row| row.get(0),
            )
            .optional()?;

        Ok(result
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc)))
    }

    pub fn update_project_last_activity(&self, project_id: u64, last_activity: DateTime<Utc>) -> Result<()> {
        self.conn.execute(
            "UPDATE projects SET last_issue_activity = ?1 WHERE id = ?2",
            params![last_activity.to_rfc3339(), project_id],
        )?;
        Ok(())
    }
}
