// src/tui/views/status_view.rs

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use git2::{Repository as GitRepo, StatusOptions};
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::git_utils::add_files;
use crate::tui_module::views::View;

pub struct StatusView {
    pub items: Vec<String>,
    pub input_mode: InputMode,
    pub input: String,
    pub selected: usize,
}

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    AddingFiles,
}

impl StatusView {
    pub fn new() -> StatusView {
        StatusView {
            items: vec![],
            input_mode: InputMode::Normal,
            input: String::new(),
            selected: 0,
        }
    }

    /// Fetches the current repository status and populates the items.
    pub fn fetch_status(&mut self) -> Result<()> {
        self.items.clear();
        let repo = GitRepo::open(".")?;

        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .renames_head_to_index(true)
            .renames_index_to_workdir(true);

        let statuses = repo.statuses(Some(&mut opts))?;

        if statuses.is_empty() {
            self.items
                .push("Nothing to commit, working tree clean.".to_string());
            return Ok(());
        }

        for entry in statuses.iter() {
            let status = entry.status();
            let file_path = entry.path().unwrap_or("Unknown");

            let status_str = match status {
                s if s.is_index_new() => "A",
                s if s.is_index_modified() => "M",
                s if s.is_index_deleted() => "D",
                s if s.is_wt_new() => "??",
                s if s.is_wt_modified() => "M",
                s if s.is_wt_deleted() => "D",
                _ => " ", //
            };

            self.items.push(format!("{} {}", status_str, file_path));
        }

        Ok(())
    }
}

impl View for StatusView {
    fn render<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        // Render the list of status items
        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let content = item.clone();
                let mut list_item = ListItem::new(content);
                if i == self.selected {
                    list_item = list_item.style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    );
                }
                list_item
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Status"))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");
        f.render_widget(list, area);
    }

    fn handle_input(&mut self, key: KeyEvent, messages: &mut Vec<String>) {
        match self.input_mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('a') => {
                    self.input_mode = InputMode::AddingFiles;
                    self.input.clear();
                    messages.push(
                        "Press 'Enter' to stage selected file or 'Esc' to cancel.".to_string(),
                    );
                }
                KeyCode::Down => {
                    if self.selected < self.items.len().saturating_sub(1) {
                        self.selected += 1;
                    }
                }
                KeyCode::Up => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    }
                }
                _ => {}
            },
            InputMode::AddingFiles => match key.code {
                KeyCode::Enter => {
                    if let Some(selected_item) = self.items.get(self.selected) {
                        // Extract the file path from the status line
                        if let Some((_, file_path)) = selected_item.split_once(' ') {
                            let file = file_path.to_string();
                            match add_files(".", &[file.clone()]) {
                                Ok(_) => {
                                    messages.push(format!("Staged file '{}'.", file));
                                    self.fetch_status().unwrap_or_else(|e| {
                                        messages.push(format!("Error fetching status: {}", e));
                                    });
                                }
                                Err(e) => {
                                    messages.push(format!("Failed to stage '{}': {}", file, e));
                                }
                            }
                        }
                    }
                    self.input_mode = InputMode::Normal;
                }
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    messages.push("Cancelled staging files.".to_string());
                }
                _ => {}
            },
        }
    }

    fn update(&mut self) {
        if let Err(e) = self.fetch_status() {
            self.items.push(format!("Error fetching status: {}", e));
        }
    }
}
