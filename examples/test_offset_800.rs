use minecli::redmine::IssuesResponse;

fn main() {
    let json = std::fs::read_to_string("/tmp/offset_800.json").unwrap();
    match serde_json::from_str::<IssuesResponse>(&json) {
        Ok(response) => {
            println!("✓ Successfully parsed {} issues", response.issues.len());
        }
        Err(e) => {
            println!("✗ Failed to parse: {}", e);
            println!("\nTrying to find the problematic issue...");

            // Try to parse as generic JSON first
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json) {
                if let Some(issues) = value["issues"].as_array() {
                    println!("Found {} issues in JSON", issues.len());
                    for (idx, issue) in issues.iter().enumerate() {
                        if let Err(e) = serde_json::from_value::<minecli::redmine::Issue>(issue.clone()) {
                            println!("\nIssue #{} failed to parse:", idx);
                            println!("Error: {}", e);
                            println!(
                                "Issue data: {}",
                                serde_json::to_string_pretty(issue).unwrap_or_default()
                            );
                            break;
                        }
                    }
                }
            }
        }
    }
}
