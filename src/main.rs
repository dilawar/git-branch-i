use std::option;

use git2::Branch;
use inquire::Select;

struct AppState {
    repo: git2::Repository,
    branches: Vec<String>,
}

impl AppState {
    fn new() -> Self {
        Self {
            repo: git2::Repository::open(".").expect("could not open repo"),
            branches: vec![],
        }
    }

    fn load_branches(&mut self) -> anyhow::Result<()> {
        self.branches.clear();

        let branches = self.repo.branches(None)?;
        for b in branches.flatten() {
            if let Some(name) = b.0.name()? {
                self.branches.push(name.into());
            }
        }

        Ok(())
    }

    fn branches(&self) -> impl Iterator<Item = &str> {
        self.branches.iter().map(|x| x.as_str())
    }

    fn print(&self) {
        let mut i = 0;
        for branch in self.branches() {
            println!("{i}: {branch}");
            i += 1;
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut app_state = AppState::new();
    // Run the main application loop
    let _app_result = run_app(&mut app_state)?;
    Ok(())
}

fn run_app(app_state: &mut AppState) -> anyhow::Result<()> {
    app_state.load_branches()?;

    let options = app_state.branches().collect();
    let selected_branch = Select::new("Select a branch?", options).prompt();
    anyhow::ensure!(selected_branch.is_ok(), "no branch is selected");

    // select action on selected branch.
    let allowed_actions = vec!["checkout", "delete"];
    let selected_action = Select::new("select action on this branch", allowed_actions).prompt();
    anyhow::ensure!(selected_action.is_ok(), "no action is selected");

    Ok(())
}
