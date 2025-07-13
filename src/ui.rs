use crate::app::{App, FocusArea, InputMode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(f.size());
    
    draw_header(f, app, chunks[0]);
    draw_main_content(f, app, chunks[1]);
    draw_status_bar(f, app, chunks[2]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let header_text = if app.input_mode == InputMode::Search {
        vec![
            Span::raw("Windows System Settings TUI - "),
            Span::styled("Search: ", Style::default().fg(Color::Yellow)),
            Span::styled(&app.search_query, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]
    } else if app.input_mode == InputMode::Edit {
        vec![
            Span::styled(
                "Windows System Settings TUI - Edit Mode",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]
    } else {
        vec![Span::styled(
            "Windows System Settings TUI",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]
    };
    
    let header = Paragraph::new(Line::from(header_text))
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White)),
        )
        .alignment(Alignment::Center);
    
    f.render_widget(header, area);
}

fn draw_main_content(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);
    
    draw_categories(f, app, chunks[0]);
    draw_items(f, app, chunks[1]);
}

fn draw_categories(f: &mut Frame, app: &App, area: Rect) {
    let categories: Vec<ListItem> = app
        .categories
        .iter()
        .enumerate()
        .map(|(i, category)| {
            let prefix = if i < 9 {
                format!("{}. ", i + 1)
            } else {
                "   ".to_string()
            };
            
            let content = if i == app.category_index {
                Line::from(vec![
                    Span::raw(prefix),
                    Span::styled(
                        category.display_name(),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::raw(prefix),
                    Span::raw(category.display_name()),
                ])
            };
            
            ListItem::new(content)
        })
        .collect();
    
    let categories_block = Block::default()
        .borders(Borders::ALL)
        .title("Categories")
        .border_style(
            if app.focus_area == FocusArea::Categories && app.input_mode != InputMode::Edit {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            },
        );
    
    let categories_list = List::new(categories)
        .block(categories_block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
    
    f.render_widget(categories_list, area);
}

fn draw_items(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .filtered_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let icon = item.icon.unwrap_or('•');
            let admin_indicator = if item.requires_admin { " [Admin]" } else { "" };
            let edit_indicator = if item.can_edit_inline { " ✏" } else { "" };
            
            let style = if i == app.item_index && app.focus_area == FocusArea::Items {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            let content = vec![
                Line::from(vec![
                    Span::raw(format!("{} ", icon)),
                    Span::styled(&item.name, style),
                    Span::styled(admin_indicator, Style::default().fg(Color::Red)),
                    Span::styled(edit_indicator, Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        item.description.as_deref().unwrap_or(""),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
            ];
            
            ListItem::new(content)
        })
        .collect();
    
    let items_block = Block::default()
        .borders(Borders::ALL)
        .title(if app.search_query.is_empty() {
            "Settings Items"
        } else {
            "Search Results"
        })
        .border_style(
            if app.focus_area == FocusArea::Items && app.input_mode != InputMode::Edit {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            },
        );
    
    if items.is_empty() {
        let empty_message = Paragraph::new("No items found")
            .style(Style::default().fg(Color::DarkGray))
            .block(items_block)
            .alignment(Alignment::Center);
        f.render_widget(empty_message, area);
    } else {
        let items_list = List::new(items)
            .block(items_block)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("→ ");
        
        f.render_stateful_widget(
            items_list,
            area,
            &mut ratatui::widgets::ListState::default().with_selected(Some(app.item_index)),
        );
    }
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status_text = if let Some(msg) = &app.status_message {
        msg.clone()
    } else {
        let help_text = match app.input_mode {
            InputMode::Normal => {
                if app.focus_area == FocusArea::Items {
                    "[Enter] Open/Edit  [e] Edit  [Tab] Switch  [/] Search  [q] Quit"
                } else {
                    "[Enter] Select  [Tab] Switch  [/] Search  [q] Quit"
                }
            },
            InputMode::Search => "[Enter] Confirm  [Esc] Cancel  Type to search...",
            InputMode::Edit => "Edit Mode Active - See edit panel for controls",
        };
        help_text.to_string()
    };
    
    let items_count = format!("Items: {}", app.filtered_items.len());
    let filter_status = if app.search_query.is_empty() {
        "Filter: None".to_string()
    } else {
        format!("Filter: '{}'", app.search_query)
    };
    
    let editable_count = app.filtered_items.iter().filter(|i| i.can_edit_inline).count();
    let edit_info = if editable_count > 0 {
        format!(" | Editable: {} (✏)", editable_count)
    } else {
        String::new()
    };
    
    let status_line = format!("Status: Ready | {} | {}{}", filter_status, items_count, edit_info);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);
    
    let help = Paragraph::new(status_text)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);
    
    let status = Paragraph::new(status_line)
        .style(Style::default().fg(Color::DarkGray))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .alignment(Alignment::Left);
    
    f.render_widget(help, chunks[0]);
    f.render_widget(status, chunks[1]);
}