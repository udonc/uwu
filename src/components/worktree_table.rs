use color_eyre::{Result, eyre::Ok};
use ratatui::{
	style::{Color, Modifier, Style},
	widgets::{Block, Row, Table, TableState},
};

use crate::{
	action::Action,
	components::Component,
	git::{Worktree, get_worktrees},
};

#[derive(Debug)]
pub struct WorktreeTable {
	worktrees: Vec<Worktree>,
	table_state: TableState,
}

impl Default for WorktreeTable {
	fn default() -> Self {
		Self::new()
	}
}

impl WorktreeTable {
	pub fn new() -> Self {
		Self {
			worktrees: Vec::new(),
			table_state: TableState::default(),
		}
	}

	pub fn load_worktrees(&mut self) -> Result<()> {
		self.worktrees = get_worktrees(None)?;
		if !self.worktrees.is_empty() {
			self.table_state.select(Some(0));
		}
		Ok(())
	}

	pub fn next(&mut self) {
		let i = match self.table_state.selected() {
			Some(i) => {
				if i >= self.worktrees.len() - 1 {
					0
				} else {
					i + 1
				}
			}
			None => 0,
		};
		self.table_state.select(Some(i));
	}

	pub fn previous(&mut self) {
		let i = match self.table_state.selected() {
			Some(i) => {
				if i == 0 {
					self.worktrees.len() - 1
				} else {
					i - 1
				}
			}
			None => 0,
		};
		self.table_state.select(Some(i));
	}
}

impl Component for WorktreeTable {
	fn register_action_handler(
		&mut self,
		tx: tokio::sync::mpsc::UnboundedSender<Action>,
	) -> Result<()> {
		let _ = tx; // to appease clippy
		Ok(())
	}

	fn register_config_handler(&mut self, config: crate::config::Config) -> Result<()> {
		let _ = config; // to appease clippy
		Ok(())
	}

	fn init(&mut self, area: ratatui::prelude::Size) -> Result<()> {
		let _ = area; // to appease clippy
		self.load_worktrees()?;
		Ok(())
	}

	fn update(&mut self, action: Action) -> Result<Option<Action>> {
		match action {
			Action::Down => {
				self.next();
				Ok(None)
			}
			Action::Up => {
				self.previous();
				Ok(None)
			}
			_ => Ok(None),
		}
	}

	fn draw(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) -> Result<()> {
		let header = Row::new(vec!["Name", "Branch", "Path"])
			.style(Style::default().fg(Color::Yellow))
			.height(1);

		let rows: Vec<Row> = self
			.worktrees
			.iter()
			.map(|worktree| {
				let branch = worktree.branch.as_deref().unwrap_or("N/A");
				Row::new(vec![
					worktree.name.clone(),
					branch.to_string(),
					worktree.path.display().to_string(),
				])
				.height(1)
			})
			.collect();

		let table = Table::new(
			rows,
			[
				ratatui::layout::Constraint::Length(15),
				ratatui::layout::Constraint::Length(20),
				ratatui::layout::Constraint::Min(0),
			],
		)
		.header(header)
		.block(Block::default().title("Worktrees"))
		.row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
		.highlight_symbol(">> ");

		frame.render_stateful_widget(table, area, &mut self.table_state);
		Ok(())
	}
}
