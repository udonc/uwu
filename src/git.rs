use color_eyre::Result;
use git2::Repository;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worktree {
	pub path: PathBuf,
	pub branch: Option<String>,
	pub commit_hash: Option<String>,
	pub is_main: bool,
}

impl Worktree {
	pub fn new(
		path: PathBuf,
		branch: Option<String>,
		commit_hash: Option<String>,
		is_main: bool,
	) -> Self {
		Self {
			path,
			branch,
			commit_hash,
			is_main,
		}
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
		let (branch, commit_hash) = match repo.head() {
			Ok(head) => (
				head.shorthand().map(|s| s.to_string()),
				head.target().map(|t| t.to_string()[..8].to_string()),
			),
			Err(_) => (None, None),
		};

		let main_worktree = Worktree::new(workdir.to_path_buf(), branch, commit_hash, true);

		worktrees.push(main_worktree);
	}

	let worktree_names = repo.worktrees()?;

	for worktree_name in worktree_names.iter() {
		let Some(name) = worktree_name else { continue };

		let worktree = repo.find_worktree(name)?;
		let worktree_path = worktree.path();

		let (branch, commit_hash) = match Repository::open(worktree_path) {
			Ok(worktree_repo) => match worktree_repo.head() {
				Ok(head) => (
					head.shorthand().map(|s| s.to_string()),
					head.target().map(|t| t.to_string()[..8].to_string()),
				),
				Err(_) => (None, None),
			},
			Err(_) => (None, None),
		};

		let worktree_info = Worktree::new(worktree_path.to_path_buf(), branch, commit_hash, false);

		worktrees.push(worktree_info);
	}

	Ok(worktrees)
}
