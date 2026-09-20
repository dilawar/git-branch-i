use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use git2::{Reference, Repository};
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

struct GitBranch<'a>(git2::Branch<'a>);

impl<'a> GitBranch<'a> {
    fn name(&self) -> Option<&str> {
        self.0.name().ok().flatten()
    }

    fn commit_time(&self) -> SystemTime {
        let branch = self.reference();
        UNIX_EPOCH
            + std::time::Duration::from_secs(
                branch
                    .peel_to_commit()
                    .expect("must be a valid commit")
                    .time()
                    .seconds() as u64,
            )
    }

    fn reference(&self) -> &Reference<'_> {
        self.0.get()
    }

    fn peel_to_commit(&self) -> anyhow::Result<git2::Commit<'_>> {
        Ok(self.reference().peel_to_commit()?)
    }
}

impl std::fmt::Display for GitBranch<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{} {:30} {:20} {} ago",
            if self.0.is_head() { '*' } else { ' ' },
            self.name().unwrap_or_default(),
            self.peel_to_commit()
                .expect("must be a valid commit")
                .author()
                .name()
                .unwrap_or("NA"),
            humantime::format_duration(
                SystemTime::now()
                    .duration_since(self.commit_time())
                    .expect("should not fail")
            ),
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
                // pre-checks.
                let repo_head = repo.head()?;
                anyhow::ensure!(
                    &repo_head != branch.reference(),
                    "can't delete current branch!"
                );
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

    // Return branches.
    //
    // - Ordered by modification time.
    // - Current user branch at the top.
    fn branches(&self) -> anyhow::Result<Vec<GitBranch<'_>>> {
        let mut result: Vec<_> = self
            .repo
            .branches(None)?
            .flatten()
            .map(|x| GitBranch(x.0))
            .collect();

        result.sort_by_key(|a| a.commit_time());

        Ok(result)
    }
}

fn main() -> anyhow::Result<()> {
    let mut app_state = AppState::new();
    // Run the main application loop
    run_app(&mut app_state)?;
    Ok(())
}

fn run_app(app_state: &AppState) -> anyhow::Result<()> {
    // select action on selected branch.
    let allowed_actions = BranchAction::iter().collect();
    let selected_action =
        Select::new("What do you want to do with a branch?", allowed_actions).prompt()?;

    let options = app_state.branches()?;
    let selected_branch =
        Select::new(&format!("Select a branch to {selected_action}"), options).prompt()?;

    selected_action.act_on(selected_branch, &app_state.repo)?;

    Ok(())
}
