use super::state::App;
use crate::redmine::Project;
use std::collections::HashMap;

impl App {
    /// Get the project at the current cursor position in the tree view
    pub fn get_project_at_cursor(&self) -> Option<&Project> {
        // Build the same tree structure as the UI
        let mut root_projects = Vec::new();
        let mut child_map: HashMap<u64, Vec<&Project>> = HashMap::new();

        for project in &self.filtered_projects {
            if let Some(parent) = &project.parent {
                child_map.entry(parent.id).or_default().push(project);
            } else {
                root_projects.push(project);
            }
        }

        let mut display_list = Vec::new();
        let mut display_index = 0;

        fn add_project_tree<'a>(
            project: &'a Project,
            child_map: &HashMap<u64, Vec<&'a Project>>,
            collapsed_map: &HashMap<u64, bool>,
            display_list: &mut Vec<&'a Project>,
            display_index: &mut usize,
        ) {
            display_list.push(project);
            *display_index += 1;

            let is_collapsed = collapsed_map.get(&project.id).copied().unwrap_or(false);
            if !is_collapsed {
                if let Some(children) = child_map.get(&project.id) {
                    for child in children {
                        add_project_tree(child, child_map, collapsed_map, display_list, display_index);
                    }
                }
            }
        }

        for project in root_projects {
            add_project_tree(
                project,
                &child_map,
                &self.projects_collapsed,
                &mut display_list,
                &mut display_index,
            );
        }

        display_list.get(self.projects_list_state).copied()
    }
}
