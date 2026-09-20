use inquire::Select;
use strum::IntoEnumIterator;

struct AppState {
    repo: git2::Repository,
    branches: Vec<String>,
}

#[derive(Default, strum::EnumString, strum::EnumIter, strum::Display)]
#[strum(serialize_all = "lowercase")]
enum BranchAction {
    #[default]
    Checkout,
    Delete,
}

impl BranchAction {
    fn act_on(self, branch: &str) -> anyhow::Result<()> {
        // act on given BranchAction
        match self {
            Self::Checkout => {
                println!("checking out {branch}");
            }
            Self::Delete => {
                println!("Deleting given branch {branch}");
            }
        }

        Ok(())
    }
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
}

fn main() -> anyhow::Result<()> {
    let mut app_state = AppState::new();
    // Run the main application loop
    run_app(&mut app_state)?;
    Ok(())
}

fn run_app(app_state: &mut AppState) -> anyhow::Result<()> {
    app_state.load_branches()?;

    let options = app_state.branches().collect();
    let selected_branch = Select::new("Select a branch?", options).prompt()?;

    // select action on selected branch.
    let allowed_actions = BranchAction::iter().collect();
    let selected_action = Select::new("select action on this branch", allowed_actions).prompt()?;

    selected_action.act_on(selected_branch)?;

    Ok(())
}
