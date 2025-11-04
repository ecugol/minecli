use minecli::redmine::IssuesResponse;

fn main() {
    let json = std::fs::read_to_string("/tmp/test_issues.json").unwrap();
    match serde_json::from_str::<IssuesResponse>(&json) {
        Ok(response) => {
            println!("✓ Successfully parsed {} issues", response.issues.len());
            for issue in &response.issues {
                println!("  - Issue #{}: {}", issue.id, issue.subject);
            }
        }
        Err(e) => {
            println!("✗ Failed to parse: {}", e);
        }
    }
}
