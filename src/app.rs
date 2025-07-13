use anyhow::Result;
use crossterm::event::KeyCode;
use ratatui::{backend::Backend, Terminal};
use crate::{ui, launcher, settings::*, event::{Event, EventHandler}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Search,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusArea {
    Categories,
    Items,
}

#[derive(Debug, Clone)]
pub struct App {
    pub categories: Vec<Category>,
    pub items: Vec<SettingsItem>,
    pub filtered_items: Vec<SettingsItem>,
    pub category_index: usize,
    pub item_index: usize,
    pub focus_area: FocusArea,
    pub input_mode: InputMode,
    pub search_query: String,
    pub status_message: Option<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let categories = Category::all();
        let items = SETTINGS_ITEMS.clone();
        let mut app = Self {
            categories,
            items,
            filtered_items: vec![],
            category_index: 0,
            item_index: 0,
            focus_area: FocusArea::Categories,
            input_mode: InputMode::Normal,
            search_query: String::new(),
            status_message: None,
            should_quit: false,
        };
        app.filter_items();
        app
    }

    pub fn filter_items(&mut self) {
        let selected_category = &self.categories[self.category_index];
        self.filtered_items = self
            .items
            .iter()
            .filter(|item| {
                let category_match = &item.category == selected_category;
                let search_match = if self.search_query.is_empty() {
                    true
                } else {
                    item.name
                        .to_lowercase()
                        .contains(&self.search_query.to_lowercase())
                        || item
                            .description
                            .as_deref()
                            .unwrap_or("")
                            .to_lowercase()
                            .contains(&self.search_query.to_lowercase())
                        || item
                            .keywords
                            .iter()
                            .any(|k| k.to_lowercase().contains(&self.search_query.to_lowercase()))
                };
                category_match && search_match
            })
            .cloned()
            .collect();
        self.item_index = 0;
    }
}

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    event_handler: EventHandler,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        match event_handler.next()? {
            Event::Tick => {}
            Event::Key(key) => {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => {
                            app.should_quit = true;
                        }
                        KeyCode::Char('/') => {
                            app.input_mode = InputMode::Search;
                            app.search_query.clear();
                        }
                        KeyCode::Tab => {
                            app.focus_area = match app.focus_area {
                                FocusArea::Categories => FocusArea::Items,
                                FocusArea::Items => FocusArea::Categories,
                            };
                        }
                        KeyCode::Down => match app.focus_area {
                            FocusArea::Categories => {
                                if app.category_index < app.categories.len() - 1 {
                                    app.category_index += 1;
                                    app.filter_items();
                                }
                            }
                            FocusArea::Items => {
                                if app.item_index < app.filtered_items.len() - 1 {
                                    app.item_index += 1;
                                }
                            }
                        },
                        KeyCode::Up => match app.focus_area {
                            FocusArea::Categories => {
                                if app.category_index > 0 {
                                    app.category_index -= 1;
                                    app.filter_items();
                                }
                            }
                            FocusArea::Items => {
                                if app.item_index > 0 {
                                    app.item_index -= 1;
                                }
                            }
                        },
                        KeyCode::Enter => {
                            if app.focus_area == FocusArea::Items {
                                if let Some(item) = app.filtered_items.get(app.item_index) {
                                    if let Err(e) = launcher::launch_setting(item) {
                                        app.status_message = Some(format!("Error: {}", e));
                                    }
                                }
                            }
                        }
                        _ => {}
                    },
                    InputMode::Search => match key.code {
                        KeyCode::Enter => {
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                            app.search_query.clear();
                            app.filter_items();
                        }
                        KeyCode::Char(c) => {
                            app.search_query.push(c);
                            app.filter_items();
                        }
                        KeyCode::Backspace => {
                            app.search_query.pop();
                            app.filter_items();
                        }
                        _ => {}
                    },
                }
            }
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
