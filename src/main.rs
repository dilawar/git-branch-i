use anyhow::Context;
use git2::Repository;
use inquire::Select;
use strum::IntoEnumIterator;

struct AppState {
    repo: Repository,
}

#[derive(Default, strum::EnumString, strum::EnumIter, strum::Display)]
#[strum(serialize_all = "lowercase")]
enum BranchAction {
    #[default]
    Checkout,
    Delete,
}

struct GitBranch<'a>(pub git2::Branch<'a>);

impl<'a> GitBranch<'a> {
    fn name(&self) -> Option<&str> {
        self.0.name().ok().flatten()
    }
}

impl std::fmt::Display for GitBranch<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            self.0
                .name()
                .map_err(|_e| std::fmt::Error)?
                .unwrap_or_default()
        )
    }
}

impl BranchAction {
    fn act_on(&self, branch: GitBranch<'_>, repo: &Repository) -> anyhow::Result<()> {
        // act on given BranchAction
        let branch_name = branch.name().context("empty name")?;
        match self {
            Self::Checkout => {
                let (object, reference) = repo.revparse_ext(branch_name)?;
                anyhow::ensure!(
                    reference.is_some(),
                    "Can't checkout tree without a valid reference"
                );
                let refname = reference.expect("must be set");
                anyhow::ensure!(refname.is_branch(), "reference is not a branch");
                println!("checking out {branch}");
                repo.checkout_tree(&object, None)?;
                repo.set_head(refname.name().expect("must have a valid name"))?;
            }
            Self::Delete => {
                println!("Deleting given branch {branch}");
                let mut branch = branch.0;
                branch.delete()?;
            }
        }

        Ok(())
    }
}

impl AppState {
    fn new() -> Self {
        Self {
            repo: Repository::open(".").expect("could not open repo"),
        }
    }

    fn branches(&self) -> anyhow::Result<Vec<GitBranch<'_>>> {
        Ok(self
            .repo
            .branches(None)?
            .flatten()
            .map(|x| GitBranch(x.0))
            .collect())
    }
}

fn main() -> anyhow::Result<()> {
    let mut app_state = AppState::new();
    // Run the main application loop
    run_app(&mut app_state)?;
    Ok(())
}

fn run_app(app_state: &AppState) -> anyhow::Result<()> {
    let options = app_state.branches()?;
    let selected_branch = Select::new("Select a branch?", options).prompt()?;

    // select action on selected branch.
    let allowed_actions = BranchAction::iter().collect();
    let selected_action = Select::new("select action on this branch", allowed_actions).prompt()?;

    selected_action.act_on(selected_branch, &app_state.repo)?;

    Ok(())
}
