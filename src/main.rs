use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use git2::{Reference, Repository};
use inquire::Select;
use strum::IntoEnumIterator;

struct AppState {
    repo: Repository,
}

#[derive(Clone, Copy, Default, strum::EnumString, strum::EnumIter, strum::Display)]
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
        let _current_branch_marker = if self.0.is_head() { '*' } else { ' ' };
        let branch_name = self.name().unwrap_or_default();

        // Format the time to X days Y hours Z mins, X secs ago"
        let commit_time = timeago::format_5chars(
            SystemTime::now()
                .duration_since(self.commit_time())
                .expect("should not fail"),
        );
        let prefix = format!(
            "{commit_time} ago by {}",
            self.peel_to_commit()
                .expect("must be a valid commit")
                .author()
                .name()
                .unwrap_or("NA")
        );
        write!(f, "({prefix:20}) {branch_name}")
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
                println!("checking out {}", branch.name().unwrap_or_default());
                repo.checkout_tree(&object, None)?;

                match reference {
                    Some(gref) => repo.set_head(gref.name().expect("must have a valid name")),
                    // this is a commit, not a reference
                    None => repo.set_head_detached(object.id()),
                }
                .expect("failed to set HEAD");
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

    // List branches depending on action
    //
    // Ordered by modification time.
    fn branches_order_by_ctime(&self, action: BranchAction) -> anyhow::Result<Vec<GitBranch<'_>>> {
        // For delete action, we only show local branches. For checkout, show remote branches
        // that do not have remote counterpart.
        let filter = match action {
            BranchAction::Delete => Some(git2::BranchType::Local),
            BranchAction::Checkout => None,
        };

        let mut result: Vec<_> = self
            .repo
            .branches(filter)?
            .flatten()
            .map(|x| GitBranch(x.0))
            .collect();

        // newest first
        result.sort_by_key(|a| a.commit_time());
        result.reverse();

        // filter brnach depending on action.
        let result = match action {
            BranchAction::Delete => result,
            BranchAction::Checkout => result,
        };

        Ok(result)
    }
}

fn main() -> anyhow::Result<()> {
    let app_state = AppState::new();
    run_app(&app_state)?;
    Ok(())
}

fn run_app(app_state: &AppState) -> anyhow::Result<()> {
    // select action on selected branch.
    let allowed_actions = BranchAction::iter().collect();
    let selected_action =
        Select::new("What do you want to do with a branch?", allowed_actions).prompt()?;

    let options = app_state.branches_order_by_ctime(selected_action)?;
    let selected_branch = Select::new(&format!("Select a branch to {selected_action}"), options)
        .with_page_size(20)
        .with_vim_mode(true)
        .prompt()?;

    selected_action.act_on(selected_branch, &app_state.repo)?;

    Ok(())
}
