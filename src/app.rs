use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{backend::Backend, Terminal};
use crate::{
    ui, 
    edit_ui,
    launcher, 
    settings::*, 
    settings_editor::*,
    event::{Event, EventHandler}
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Search,
    Edit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusArea {
    Categories,
    Items,
}

#[derive(Debug, Clone)]
pub struct EditState {
    pub item_name: String,
    pub editor: Box<dyn SettingEditor>,
    pub editor_type: EditorType,
    pub current_value: SettingValue,
    pub pending_value: Option<SettingValue>,
    pub options: Vec<SettingOption>,
    pub selected_option_index: usize,
    pub scroll_offset: u16,
    pub error_message: Option<String>,
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
    pub edit_state: Option<EditState>,
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
            edit_state: None,
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
    
    fn enter_edit_mode(&mut self, item: &SettingsItem) -> Result<()> {
        if let Some(editor_key) = &item.editor_key {
            if let Some(editor) = create_editor(editor_key) {
                let current_value = editor.get_current_value()?;
                let options = editor.get_available_options()?;
                let editor_type = editor.get_editor_type();
                
                self.edit_state = Some(EditState {
                    item_name: item.name.clone(),
                    editor,
                    editor_type,
                    current_value: current_value.clone(),
                    pending_value: Some(current_value),
                    options,
                    selected_option_index: 0,
                    scroll_offset: 0,
                    error_message: None,
                });
                
                self.input_mode = InputMode::Edit;
                Ok(())
            } else {
                anyhow::bail!("No editor available for this setting")
            }
        } else {
            anyhow::bail!("This setting cannot be edited inline")
        }
    }
    
    fn save_edit(&mut self) -> Result<()> {
        if let Some(edit_state) = &self.edit_state {
            if let Some(pending_value) = &edit_state.pending_value {
                // Validate before saving
                if edit_state.editor.validate_value(pending_value)? {
                    edit_state.editor.set_value(pending_value.clone())?;
                    self.status_message = Some(format!("✓ {} updated successfully", edit_state.item_name));
                    self.input_mode = InputMode::Normal;
                    self.edit_state = None;
                    Ok(())
                } else {
                    anyhow::bail!("Invalid value")
                }
            } else {
                anyhow::bail!("No value to save")
            }
        } else {
            Ok(())
        }
    }
    
    fn cancel_edit(&mut self) {
        self.input_mode = InputMode::Normal;
        self.edit_state = None;
        self.status_message = Some("Edit cancelled".to_string());
    }
}

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    event_handler: EventHandler,
) -> Result<()> {
    loop {
        terminal.draw(|f| {
            if app.input_mode == InputMode::Edit {
                ui::draw(f, &app);
                // Draw edit overlay
                let area = centered_rect(80, 80, f.size());
                if let Some(edit_state) = &app.edit_state {
                    edit_ui::draw_edit_panel(f, area, edit_state);
                }
            } else {
                ui::draw(f, &app);
            }
        })?;

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
                                if let Some(item) = app.filtered_items.get(app.item_index).cloned() {
                                    if item.can_edit_inline {
                                        // Try to enter edit mode
                                        if let Err(e) = app.enter_edit_mode(&item) {
                                            app.status_message = Some(format!("Error: {}", e));
                                        }
                                    } else {
                                        // Fall back to launching the settings panel
                                        if let Err(e) = launcher::launch_setting(&item) {
                                            app.status_message = Some(format!("Error: {}", e));
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Char('e') => {
                            // Quick edit shortcut
                            if app.focus_area == FocusArea::Items {
                                if let Some(item) = app.filtered_items.get(app.item_index).cloned() {
                                    if item.can_edit_inline {
                                        if let Err(e) = app.enter_edit_mode(&item) {
                                            app.status_message = Some(format!("Error: {}", e));
                                        }
                                    } else {
                                        app.status_message = Some("This setting cannot be edited inline".to_string());
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
                    InputMode::Edit => {
                        if let Some(edit_state) = &mut app.edit_state {
                            match key.code {
                                KeyCode::Esc => {
                                    app.cancel_edit();
                                }
                                KeyCode::Char('s') | KeyCode::Char('S') => {
                                    if let Err(e) = app.save_edit() {
                                        app.status_message = Some(format!("Save failed: {}", e));
                                    }
                                }
                                KeyCode::Enter | KeyCode::Char(' ') => {
                                    match &edit_state.editor_type {
                                        EditorType::Toggle => {
                                            // Toggle the value
                                            let new_val = match &edit_state.pending_value {
                                                Some(SettingValue::Bool(b)) => SettingValue::Bool(!b),
                                                _ => SettingValue::Bool(true),
                                            };
                                            edit_state.pending_value = Some(new_val);
                                        }
                                        EditorType::Dropdown | EditorType::ResolutionPicker => {
                                            // Select current option
                                            if let Some(option) = edit_state.options.get(edit_state.selected_option_index) {
                                                edit_state.pending_value = Some(option.value.clone());
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                KeyCode::Up => {
                                    if edit_state.selected_option_index > 0 {
                                        edit_state.selected_option_index -= 1;
                                    }
                                }
                                KeyCode::Down => {
                                    if edit_state.selected_option_index < edit_state.options.len() - 1 {
                                        edit_state.selected_option_index += 1;
                                    }
                                }
                                KeyCode::Left => {
                                    if let EditorType::Slider { min, max: _, step } = &edit_state.editor_type {
                                        if let Some(SettingValue::Float(val)) = &mut edit_state.pending_value {
                                            let adjustment = if key.modifiers.contains(KeyModifiers::SHIFT) {
                                                step * 0.1
                                            } else {
                                                *step
                                            };
                                            *val = (*val - adjustment).max(*min);
                                        }
                                    }
                                }
                                KeyCode::Right => {
                                    if let EditorType::Slider { min: _, max, step } = &edit_state.editor_type {
                                        if let Some(SettingValue::Float(val)) = &mut edit_state.pending_value {
                                            let adjustment = if key.modifiers.contains(KeyModifiers::SHIFT) {
                                                step * 0.1
                                            } else {
                                                *step
                                            };
                                            *val = (*val + adjustment).min(*max);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
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

/// Helper function to create centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let popup_layout = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
            ratatui::layout::Constraint::Percentage(percent_y),
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
            ratatui::layout::Constraint::Percentage(percent_x),
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}