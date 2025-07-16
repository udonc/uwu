use std::path::PathBuf;

use color_eyre::{Result, eyre::eyre, owo_colors::OwoColorize};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use git2::Repository;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    symbols,
    text::Line,
    widgets::{Block, List, ListItem, ListState, Paragraph},
};

#[derive(Debug, Clone)]
pub struct Worktree {
    name: String,
    path: PathBuf,
    is_current: bool,
}

/// The current screen of the application.
#[derive(Debug)]
enum CurrentScreen {
    WorktreeList,
}

/// The main application which holds the state and logic of the application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    running: bool,
    /// The current screen of the application.
    current_screen: CurrentScreen,
    /// List of worktrees. (dummy for now)
    worktrees: Vec<Worktree>,
    /// List state for navigation.
    list_state: ListState,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Result<Self> {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let worktrees = Self::load_worktrees()?;

        if !worktrees.is_empty() {
            list_state.select(Some(0));
        }

        Ok(Self {
            running: false,
            current_screen: CurrentScreen::WorktreeList,
            worktrees,
            list_state,
        })
    }

    fn load_worktrees() -> Result<Vec<Worktree>> {
        let repo = Repository::open_from_env().or_else(|_| Repository::discover("."))?;

        let mut worktrees = Vec::new();

        // Get the main worktree (repo root)
        let workdir = repo
            .workdir()
            .ok_or_else(|| eyre!("Reoisitory has no workdir"))?;

        let current_branch = repo
            .head()
            .ok()
            .and_then(|head| head.shorthand().map(|s| s.to_string()))
            .unwrap_or_else(|| "HEAD".to_string());

        worktrees.push(Worktree {
            name: current_branch.clone(),
            path: workdir.to_path_buf(),
            is_current: true,
        });

        // Get all other worktrees
        let worktree_list = repo.worktrees()?;
        for worktree_name in worktree_list.iter() {
            let Some(name) = worktree_name else { continue };
            if let Ok(worktree) = repo.find_worktree(name) {
                worktrees.push(Worktree {
                    name: name.to_string(),
                    path: worktree.path().to_path_buf(),
                    is_current: false,
                });
            }
        }

        Ok(worktrees)
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    ///
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/main/ratatui-widgets/examples>
    fn render(&mut self, frame: &mut Frame) {
        let title = Line::from("uwu - Unified Worktree Utility")
            .bold()
            .magenta();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // List
                Constraint::Length(3), // Footer
            ])
            .split(frame.area());

        // Header
        frame.render_widget(
            Paragraph::new("Git Worktrees")
                .block(
                    Block::bordered()
                        .border_set(symbols::border::ROUNDED)
                        .title(title.clone()),
                )
                .centered(),
            chunks[0],
        );

        // List of worktrees
        let items = self
            .worktrees
            .iter()
            .map(|worktree| {
                let display_text = if worktree.is_current {
                    format!("* {} ({})", worktree.name, worktree.path.display())
                } else {
                    format!("  {} ({})", worktree.name, worktree.path.display())
                };
                ListItem::new(display_text)
            })
            .collect::<Vec<_>>();

        let list = List::new(items)
            .block(Block::bordered().title("Worktrees"))
            .highlight_style(Style::default().bg(Color::Magenta).fg(Color::Black))
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, chunks[1], &mut self.list_state);

        // Footer
        frame.render_widget(
            Paragraph::new("Use arrow keys to navigate and Enter to select, Esc/q to quit.")
                .block(Block::bordered())
                .centered(),
            chunks[2],
        );
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Moves to the previous item in the list.
    fn prev_item(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.worktrees.len() - 1 // wrap around to the last item
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Moves to the next item in the list.
    fn next_item(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.worktrees.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// Selects the currently highlighted item in the list.
    fn select_item(&mut self) {
        unimplemented!();
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Up) => self.prev_item(),
            (_, KeyCode::Down) => self.next_item(),
            (_, KeyCode::Enter) => self.select_item(),
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}
