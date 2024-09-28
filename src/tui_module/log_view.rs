use crate::utils::{print_error, print_info};
use anyhow::{Context, Result};
use chrono::{NaiveDateTime, Utc};
use crossterm::event::{KeyCode, KeyEvent};
use git2::Repository as GitRepo;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub struct LogView {
    pub items: Vec<CommitItem>,
    pub selected: usize,

    pub detailed_commit: Option<CommitDetail>,
}

#[derive(Clone)]
pub struct CommitItem {
    pub id: String,
    pub author: String,
    pub date: String, // New field for commit date
    pub message: String,
}

pub struct CommitDetail {
    pub id: String,
    pub author: String,
    pub date: String,
    pub message: String,
    pub parents: Vec<String>,
}

impl LogView {
    pub fn new() -> LogView {
        LogView {
            items: vec![],
            selected: 0,

            detailed_commit: None,
        }
    }

    pub fn render<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        if let Some(detail) = &self.detailed_commit {
            let block = Block::default()
                .borders(Borders::ALL)
                .title("Commit Details")
                .style(Style::default().fg(Color::Green));
            let content = vec![
                format!("Commit ID: {}", detail.id),
                format!("Author: {}", detail.author),
                format!("Date: {}", detail.date),
                "".to_string(),
                "Message:".to_string(),
                detail.message.clone(),
                "".to_string(),
                "Parents:".to_string(),
                detail.parents.join(", "),
            ];
            let paragraph = Paragraph::new(content.join("\n"))
                .block(block)
                .style(Style::default().fg(Color::White))
                .alignment(tui::layout::Alignment::Left)
                .wrap(tui::widgets::Wrap { trim: true });
            f.render_widget(paragraph, area);
            return;
        }

        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, commit)| {
                let content = format!(
                    "{} {} [{}] - {}",
                    commit.id, commit.author, commit.date, commit.message
                );
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
            .block(Block::default().borders(Borders::ALL).title("Commit Log"))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");
        f.render_widget(list, area);
    }

    pub fn handle_input(&mut self, key: KeyEvent, messages: &mut Vec<String>) -> Result<()> {
        match key.code {
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
                    let commit = &self.items[self.selected];
                    self.detailed_commit = Some(self.get_commit_detail(&commit.id)?);
                }
            }
            KeyCode::Char('r') => {
                self.update();
                messages.push("Commit logs refreshed.".to_string());
            }
            KeyCode::Esc => {
                if self.detailed_commit.is_some() {
                    self.detailed_commit = None;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn update(&mut self) {
        self.items.clear();
        self.detailed_commit = None;
        match GitRepo::open(".") {
            Ok(repo) => {
                let mut revwalk = match repo.revwalk() {
                    Ok(rw) => rw,
                    Err(e) => {
                        self.items.push(CommitItem {
                            id: "Error".to_string(),
                            author: "Error".to_string(),
                            date: "".to_string(),
                            message: format!("Error creating revwalk: {}", e),
                        });
                        return;
                    }
                };

                if let Err(e) = revwalk.push_head() {
                    self.items.push(CommitItem {
                        id: "Error".to_string(),
                        author: "Error".to_string(),
                        date: "".to_string(),
                        message: format!("Error pushing HEAD: {}", e),
                    });
                    return;
                }

                revwalk
                    .set_sorting(git2::Sort::TIME | git2::Sort::REVERSE)
                    .unwrap();
                use anyhow::Context;
                for oid_result in revwalk {
                    match oid_result {
                        Ok(oid) => match repo.find_commit(oid) {
                            Ok(commit) => {
                                let author =
                                    commit.author().name().unwrap_or("Unknown").to_string();
                                let message = commit
                                    .message()
                                    .unwrap_or("")
                                    .split('\n')
                                    .next()
                                    .unwrap_or("");

                                // Extract and format the commit date
                                let time = commit.time();
                                let timestamp = time.seconds();
                                let naive = NaiveDateTime::from_timestamp_opt(timestamp, 0)
                                    .unwrap_or_else(|| NaiveDateTime::from_timestamp(0, 0));
                                let datetime = naive.format("%Y-%m-%d %H:%M:%S").to_string();

                                self.items.push(CommitItem {
                                    id: commit.id().to_string(),
                                    author: author.to_string(),
                                    date: datetime, // Assign formatted date
                                    message: message.to_string(),
                                });
                            }
                            Err(e) => {
                                self.items.push(CommitItem {
                                    id: "Error".to_string(),
                                    author: "Error".to_string(),
                                    date: "".to_string(),
                                    message: format!("Error finding commit {}: {}", oid, e),
                                });
                            }
                        },
                        Err(e) => {
                            self.items.push(CommitItem {
                                id: "Error".to_string(),
                                author: "Error".to_string(),
                                date: "".to_string(),
                                message: format!("Error iterating oid: {}", e),
                            });
                        }
                    }
                }
            }
            Err(e) => {
                self.items.push(CommitItem {
                    id: "Error".to_string(),
                    author: "Error".to_string(),
                    date: "".to_string(),
                    message: format!("Error opening repository: {}", e),
                });
            }
        }

        // Reset selection if necessary
        if self.selected >= self.items.len() && self.selected > 0 {
            self.selected = self.items.len() - 1;
        }
    }

    fn get_commit_detail(&self, commit_id: &str) -> Result<CommitDetail> {
        let repo = GitRepo::open(".").context("Failed to open repository")?;
        let oid = commit_id.parse()?;
        let commit = repo
            .find_commit(oid)
            .with_context(|| format!("Failed to find commit '{}'", commit_id))?;

        let parents = commit
            .parents()
            .map(|parent| parent.id().to_string())
            .collect();

        // Format the commit date
        let time = commit.time();
        let timestamp = time.seconds();
        let naive = NaiveDateTime::from_timestamp_opt(timestamp, 0)
            .unwrap_or_else(|| NaiveDateTime::from_timestamp(0, 0));
        let datetime = naive.format("%Y-%m-%d %H:%M:%S").to_string();

        let detail = CommitDetail {
            id: commit.id().to_string(),
            author: commit.author().name().unwrap_or("Unknown").to_string(),
            date: datetime, // Assign formatted date
            message: commit.message().unwrap_or("").to_string(),
            parents,
        };

        Ok(detail)
    }
}
