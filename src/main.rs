fn main() -> anyhow::Result<()> {
    app()
}

fn app() -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let repo = git2::Repository::open(cwd)?;
    let branches = repo.branches(None)?;

    for branch in branches.flatten() {
        println!("{:?}", branch.0.name()?);
    }

    Ok(())
}
