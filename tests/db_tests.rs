use chrono::Utc;
use minecli::db::Database;
use minecli::redmine::{IdName, Issue, Journal, JournalDetail, Project};
use tempfile::TempDir;

fn create_test_db() -> (Database, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(db_path).unwrap();
    (db, temp_dir)
}

fn create_test_project(id: u64, name: &str) -> Project {
    Project {
        id,
        name: name.to_string(),
        identifier: format!("proj-{}", id),
        description: Some("Test project".to_string()),
        status: Some(1),
        parent: None,
        created_on: Some(Utc::now()),
        updated_on: Some(Utc::now()),
        last_issue_activity: None,
        last_issues_sync: None,
    }
}

fn create_test_issue(id: u64, project_id: u64, subject: &str) -> Issue {
    Issue {
        id,
        project: IdName {
            id: project_id,
            name: "Test Project".to_string(),
        },
        tracker: IdName {
            id: 1,
            name: "Bug".to_string(),
        },
        status: IdName {
            id: 1,
            name: "New".to_string(),
        },
        priority: IdName {
            id: 2,
            name: "Normal".to_string(),
        },
        author: IdName {
            id: 1,
            name: "Test User".to_string(),
        },
        assigned_to: None,
        subject: subject.to_string(),
        description: Some("Test description".to_string()),
        start_date: None,
        created_on: Utc::now(),
        updated_on: Utc::now(),
        closed_on: None,
        due_date: None,
        done_ratio: Some(0),
        is_private: None,
        estimated_hours: None,
        journals: vec![],
        custom_fields: vec![],
        attachments: vec![],
    }
}

#[test]
fn test_insert_and_get_projects() {
    let (db, _temp) = create_test_db();

    let projects = vec![
        create_test_project(1, "Project Alpha"),
        create_test_project(2, "Project Beta"),
        create_test_project(3, "Project Gamma"),
    ];

    db.insert_projects(&projects).unwrap();

    let retrieved = db.get_projects(None).unwrap();
    assert_eq!(retrieved.len(), 3);

    // Check all projects are present (order may vary due to sorting)
    let names: Vec<_> = retrieved.iter().map(|p| p.name.as_str()).collect();
    assert!(names.contains(&"Project Alpha"));
    assert!(names.contains(&"Project Beta"));
    assert!(names.contains(&"Project Gamma"));
}

#[test]
fn test_filter_projects() {
    let (db, _temp) = create_test_db();

    let projects = vec![
        create_test_project(1, "Alpha Project"),
        create_test_project(2, "Beta Project"),
        create_test_project(3, "Gamma Project"),
    ];

    db.insert_projects(&projects).unwrap();

    // Filter by name
    let filtered = db.get_projects(Some("Alpha")).unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "Alpha Project");

    // Filter with partial match
    let filtered = db.get_projects(Some("project")).unwrap();
    assert_eq!(filtered.len(), 3); // All contain "project"
}

#[test]
fn test_insert_and_get_issues() {
    let (db, _temp) = create_test_db();

    // Insert project first
    let project = create_test_project(1, "Test Project");
    db.insert_projects(&vec![project]).unwrap();

    // Insert issues
    let issues = vec![
        create_test_issue(1, 1, "First Issue"),
        create_test_issue(2, 1, "Second Issue"),
    ];

    db.insert_issues(&issues).unwrap();

    let retrieved = db
        .get_issues(Some(1), minecli::app::IssueSortOrder::UpdatedDesc, None, None)
        .unwrap();
    assert_eq!(retrieved.len(), 2);
}

#[test]
fn test_clear_project_issues() {
    let (db, _temp) = create_test_db();

    // Insert project
    let project = create_test_project(1, "Test Project");
    db.insert_projects(&vec![project]).unwrap();

    // Insert issues
    let issues = vec![create_test_issue(1, 1, "Issue 1"), create_test_issue(2, 1, "Issue 2")];
    db.insert_issues(&issues).unwrap();

    // Verify inserted
    let retrieved = db
        .get_issues(Some(1), minecli::app::IssueSortOrder::UpdatedDesc, None, None)
        .unwrap();
    assert_eq!(retrieved.len(), 2);

    // Clear issues for project
    db.clear_project_issues(1).unwrap();

    // Verify cleared
    let retrieved = db
        .get_issues(Some(1), minecli::app::IssueSortOrder::UpdatedDesc, None, None)
        .unwrap();
    assert_eq!(retrieved.len(), 0);
}

#[test]
fn test_issue_with_journals() {
    let (db, _temp) = create_test_db();

    // Insert project
    let project = create_test_project(1, "Test Project");
    db.insert_projects(&vec![project]).unwrap();

    // Create issue with journals
    let mut issue = create_test_issue(1, 1, "Issue with Notes");
    issue.journals = vec![Journal {
        id: 1,
        user: IdName {
            id: 1,
            name: "Test User".to_string(),
        },
        notes: Some("This is a comment".to_string()),
        created_on: Utc::now(),
        details: vec![JournalDetail {
            property: "attr".to_string(),
            name: "status_id".to_string(),
            old_value: Some("1".to_string()),
            new_value: Some("2".to_string()),
        }],
    }];

    db.insert_issue_with_journals(&issue).unwrap();

    // Retrieve with journals
    let retrieved = db.get_issue_with_journals(1).unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.journals.len(), 1);
    assert_eq!(retrieved.journals[0].notes.as_ref().unwrap(), "This is a comment");
    assert_eq!(retrieved.journals[0].details.len(), 1);
}

#[test]
fn test_issue_filtering() {
    let (db, _temp) = create_test_db();

    // Insert project
    let project = create_test_project(1, "Test Project");
    db.insert_projects(&vec![project]).unwrap();

    // Insert issues with different properties
    let mut issue1 = create_test_issue(1, 1, "Bug in login");
    issue1.assigned_to = Some(IdName {
        id: 5,
        name: "User A".to_string(),
    });

    let mut issue2 = create_test_issue(2, 1, "Feature request");
    issue2.assigned_to = Some(IdName {
        id: 6,
        name: "User B".to_string(),
    });

    let issue3 = create_test_issue(3, 1, "Another bug");

    db.insert_issues(&vec![issue1, issue2, issue3]).unwrap();

    // Filter by subject
    let filtered = db
        .get_issues(
            Some(1),
            minecli::app::IssueSortOrder::UpdatedDesc,
            Some("bug"),
            None,
        )
        .unwrap();
    assert_eq!(filtered.len(), 2); // "Bug in login" and "Another bug"

    // Filter by assignee
    let filtered = db
        .get_issues(Some(1), minecli::app::IssueSortOrder::UpdatedDesc, None, Some(5))
        .unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].subject, "Bug in login");
}

#[test]
fn test_sync_timestamps() {
    let (db, _temp) = create_test_db();

    // Initially no sync
    let last_sync = db.get_last_projects_sync().unwrap();
    assert!(last_sync.is_none());

    // Insert projects (updates sync time)
    let projects = vec![create_test_project(1, "Test Project")];
    db.insert_projects(&projects).unwrap();

    // Should have sync time now
    let last_sync = db.get_last_projects_sync().unwrap();
    assert!(last_sync.is_some());
}
