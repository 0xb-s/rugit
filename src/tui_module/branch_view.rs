

use crate::git_utils::{create_branch, delete_branch, switch_branch};
use crate::utils::{print_error, print_info};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use git2::{BranchType, Error as GitError, Repository as GitRepo};
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub struct BranchView {
    pub items: Vec<String>,
    pub input_mode: InputMode,
    pub input: String,
    pub selected: usize, // Index of the selected branch
}

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    CreatingBranch,
    DeletingBranch,
}

impl BranchView {
    pub fn new() -> BranchView {
        BranchView {
            items: vec![],
            input_mode: InputMode::Normal,
            input: String::new(),
            selected: 0,
        }
    }

    pub fn render<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        // If in input mode, render the input prompt
        if self.input_mode != InputMode::Normal {
            let block = Block::default()
                .borders(Borders::ALL)
                .title(match self.input_mode {
                    InputMode::CreatingBranch => "Create New Branch",
                    InputMode::DeletingBranch => "Delete Branch",
                    _ => "",
                });
            let paragraph =
                Paragraph::new(&self.input[..])
                    .block(block)
                    .style(match self.input_mode {
                        InputMode::CreatingBranch => Style::default().fg(Color::Green),
                        InputMode::DeletingBranch => Style::default().fg(Color::Red),
                        _ => Style::default(),
                    });
            f.render_widget(Clear, area); // Clear the area before rendering the input
            f.render_widget(paragraph, area);
            return;
        }

        // Render the list of branches with the selected item highlighted
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
            .block(Block::default().borders(Borders::ALL).title("Branches"))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");
        f.render_widget(list, area);
    }

    pub fn handle_input(&mut self, key: KeyEvent, messages: &mut Vec<String>) -> Result<()> {
        match self.input_mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('c') => {
                    self.input_mode = InputMode::CreatingBranch;
                    self.input.clear();
                    messages.push("Enter new branch name:".to_string());
                }
                KeyCode::Char('d') => {
                    if !self.items.is_empty() {
                        self.input_mode = InputMode::DeletingBranch;
                        self.input.clear();
                        messages.push("Enter branch name to delete:".to_string());
                    } else {
                        messages.push("No branches available to delete.".to_string());
                    }
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
                KeyCode::Enter => {
                    if !self.items.is_empty() {
                        let branch_name = self.items[self.selected].trim_start_matches("* ").trim();
                        match switch_branch(".", branch_name) {
                            Ok(_) => {
                                messages.push(format!("Switched to branch '{}'.", branch_name))
                            }
                            Err(e) => messages.push(format!("Failed to switch branch: {}", e)),
                        }
                        self.update(); // Refresh the branch list
                    }
                }
                _ => {}
            },
            InputMode::CreatingBranch => match key.code {
                KeyCode::Enter => {
                    let branch_name = self.input.trim();
                    if branch_name.is_empty() {
                        messages.push("Branch name cannot be empty.".to_string());
                    } else {
                        match create_branch(".", branch_name) {
                            Ok(_) => messages.push(format!("Branch '{}' created.", branch_name)),
                            Err(e) => messages.push(format!("Failed to create branch: {}", e)),
                        }
                        self.update(); // Refresh the branch list
                    }
                    self.input_mode = InputMode::Normal;
                    self.input.clear();
                }
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.input.clear();
                    messages.push("Branch creation cancelled.".to_string());
                }
                KeyCode::Char(c) => {
                    self.input.push(c);
                }
                KeyCode::Backspace => {
                    self.input.pop();
                }
                _ => {}
            },
            InputMode::DeletingBranch => match key.code {
                KeyCode::Enter => {
                    let branch_name = self.input.trim();
                    if branch_name.is_empty() {
                        messages.push("Branch name cannot be empty.".to_string());
                    } else {
                        match delete_branch(".", branch_name) {
                            Ok(_) => messages.push(format!("Branch '{}' deleted.", branch_name)),
                            Err(e) => messages.push(format!("Failed to delete branch: {}", e)),
                        }
                        self.update(); // Refresh the branch list
                    }
                    self.input_mode = InputMode::Normal;
                    self.input.clear();
                }
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.input.clear();
                    messages.push("Branch deletion cancelled.".to_string());
                }
                KeyCode::Char(c) => {
                    self.input.push(c);
                }
                KeyCode::Backspace => {
                    self.input.pop();
                }
                _ => {}
            },
        }
        Ok(())
    }

    pub fn update(&mut self) {
        self.items.clear();
        match GitRepo::open(".") {
            Ok(repo) => match repo.branches(Some(BranchType::Local)) {
                Ok(branches) => {
                    for branch in branches {
                        match branch {
                            Ok((b, _)) => {
                                let name = match b.name() {
                                    Ok(Some(n)) => n.to_string(),
                                    _ => "Unnamed".to_string(),
                                };
                                if b.is_head() {
                                    self.items.push(format!("* {}", name));
                                } else {
                                    self.items.push(format!("  {}", name));
                                }
                            }
                            Err(e) => {
                                self.items.push(format!("Error iterating branches: {}", e));
                            }
                        }
                    }
                }
                Err(e) => {
                    self.items.push(format!("Error retrieving branches: {}", e));
                }
            },
            Err(e) => {
                self.items.push(format!("Error opening repository: {}", e));
            }
        }
    }
}
