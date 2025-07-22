use color_eyre::Result;
use git2::Repository;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worktree {
	pub name: String,
	pub path: PathBuf,
	pub branch: Option<String>,
}

impl Worktree {
	pub fn new(name: String, path: PathBuf, branch: Option<String>) -> Self {
		Self { name, path, branch }
	}
}

pub fn get_worktrees(repo_path: Option<PathBuf>) -> Result<Vec<Worktree>> {
	let repo = if let Some(path) = repo_path {
		Repository::open(path)?
	} else {
		Repository::open_from_env()?
	};

	let mut worktrees = Vec::new();

	if let Some(workdir) = repo.workdir() {
		let branch = match repo.head() {
			Ok(head) => head.shorthand().map(|s| s.to_string()),
			Err(_) => None,
		};

		let main_worktree = Worktree::new("MAIN".to_string(), workdir.to_path_buf(), branch);

		worktrees.push(main_worktree);
	}

	let worktree_names = repo.worktrees()?;

	for worktree_name in worktree_names.iter() {
		let Some(name) = worktree_name else { continue };

		let worktree = repo.find_worktree(name)?;
		let worktree_path = worktree.path();

		let branch = match Repository::open(worktree_path) {
			Ok(worktree_repo) => match worktree_repo.head() {
				Ok(head) => head.shorthand().map(|s| s.to_string()),
				Err(_) => None,
			},
			Err(_) => None,
		};

		let worktree_info = Worktree::new(name.to_string(), worktree_path.to_path_buf(), branch);

		worktrees.push(worktree_info);
	}

	Ok(worktrees)
}
