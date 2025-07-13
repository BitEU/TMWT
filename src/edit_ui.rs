use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use crate::{
    app::EditState,
    settings_editor::{EditorType, SettingOption, SettingValue},
};

pub fn draw_edit_panel(f: &mut Frame, area: Rect, edit_state: &EditState) {
    // Clear the area first
    f.render_widget(Clear, area);
    
    // Create the main block
    let block = Block::default()
        .title(format!(" Editing: {} ", edit_state.item_name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    
    let inner_area = block.inner(area);
    f.render_widget(block, area);
    
    // Split the inner area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Current value
            Constraint::Min(5),     // Editor area
            Constraint::Length(4),  // Help text
        ])
        .split(inner_area);
    
    // Draw current value
    draw_current_value(f, chunks[0], &edit_state.current_value);
    
    // Draw the editor based on type
    match &edit_state.editor_type {
        EditorType::Toggle => draw_toggle_editor(f, chunks[1], edit_state),
        EditorType::Dropdown => draw_dropdown_editor(f, chunks[1], edit_state),
        EditorType::ResolutionPicker => draw_resolution_picker(f, chunks[1], edit_state),
        EditorType::Slider { min, max, step } => {
            draw_slider_editor(f, chunks[1], edit_state, *min, *max, *step)
        }
        _ => draw_unsupported_editor(f, chunks[1]),
    }
    
    // Draw help text
    draw_edit_help(f, chunks[2], &edit_state.editor_type);
}

fn draw_current_value(f: &mut Frame, area: Rect, value: &SettingValue) {
    let current = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Current: "),
            Span::styled(
                value.to_string(),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    
    f.render_widget(current, area);
}

fn draw_toggle_editor(f: &mut Frame, area: Rect, edit_state: &EditState) {
    let is_enabled = matches!(&edit_state.pending_value, Some(SettingValue::Bool(true)));
    
    let toggle_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            if is_enabled {
                Span::styled("● ", Style::default().fg(Color::Green))
            } else {
                Span::raw("○ ")
            },
            Span::styled(
                "Enabled",
                if is_enabled {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
            Span::raw("    "),
            if !is_enabled {
                Span::styled("● ", Style::default().fg(Color::Red))
            } else {
                Span::raw("○ ")
            },
            Span::styled(
                "Disabled",
                if !is_enabled {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
        ]),
    ];
    
    let paragraph = Paragraph::new(toggle_text)
        .alignment(Alignment::Left);
    
    f.render_widget(paragraph, area);
}

fn draw_dropdown_editor(f: &mut Frame, area: Rect, edit_state: &EditState) {
    let items: Vec<ListItem> = edit_state
        .options
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let is_selected = i == edit_state.selected_option_index;
            let is_current = Some(&opt.value) == edit_state.pending_value.as_ref();
            
            let mut spans = vec![
                if is_current {
                    Span::styled("► ", Style::default().fg(Color::Green))
                } else {
                    Span::raw("  ")
                },
                Span::raw(&opt.label),
            ];
            
            if let Some(desc) = &opt.description {
                spans.push(Span::styled(
                    format!(" - {}", desc),
                    Style::default().fg(Color::DarkGray),
                ));
            }
            
            let style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if is_current {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };
            
            ListItem::new(Line::from(spans)).style(style)
        })
        .collect();
    
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::NONE)
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("→ ");
    
    f.render_stateful_widget(
        list,
        area,
        &mut ratatui::widgets::ListState::default()
            .with_selected(Some(edit_state.selected_option_index)),
    );
}

fn draw_resolution_picker(f: &mut Frame, area: Rect, edit_state: &EditState) {
    // Group resolutions by aspect ratio
    let mut grouped: std::collections::BTreeMap<String, Vec<&SettingOption>> = 
        std::collections::BTreeMap::new();
    
    for opt in &edit_state.options {
        if let SettingValue::Resolution { width, height } = &opt.value {
            let ratio = gcd(*width, *height);
            let aspect = format!("{}:{}", width / ratio, height / ratio);
            grouped.entry(aspect).or_insert_with(Vec::new).push(opt);
        }
    }
    
    let mut lines = vec![];
    let mut item_index = 0;
    
    for (aspect, resolutions) in grouped {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {} ", aspect),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED),
            ),
        ]));
        
        for res in resolutions {
            let is_selected = item_index == edit_state.selected_option_index;
            let is_current = Some(&res.value) == edit_state.pending_value.as_ref();
            
            let prefix = if is_current { "► " } else { "  " };
            let style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if is_current {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };
            
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(prefix, style),
                Span::styled(&res.label, style),
                if let Some(desc) = &res.description {
                    Span::styled(format!(" ({})", desc), Style::default().fg(Color::DarkGray))
                } else {
                    Span::raw("")
                },
            ]));
            
            item_index += 1;
        }
        
        lines.push(Line::from("")); // Empty line between groups
    }
    
    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: true })
        .scroll((edit_state.scroll_offset, 0));
    
    f.render_widget(paragraph, area);
}

fn draw_slider_editor(f: &mut Frame, area: Rect, edit_state: &EditState, min: f64, max: f64, _step: f64) {
    let current_val = match &edit_state.pending_value {
        Some(SettingValue::Float(v)) => *v,
        Some(SettingValue::Integer(v)) => *v as f64,
        _ => min,
    };
    
    let percentage = ((current_val - min) / (max - min) * 100.0) as u16;
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(area);
    
    // Value display
    let value_text = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Value: "),
            Span::styled(
                format!("{:.1}", current_val),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
        ]),
    ])
    .alignment(Alignment::Center);
    
    f.render_widget(value_text, chunks[0]);
    
    // Slider bar
    let slider_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(6),
            Constraint::Min(10),
            Constraint::Length(6),
        ])
        .split(chunks[1]);
    
    let min_label = Paragraph::new(format!("{:.0}", min))
        .alignment(Alignment::Right);
    let max_label = Paragraph::new(format!("{:.0}", max))
        .alignment(Alignment::Left);
    
    f.render_widget(min_label, slider_chunks[0]);
    f.render_widget(max_label, slider_chunks[2]);
    
    // Draw the slider bar
    let bar_width = slider_chunks[1].width as usize;
    let filled_width = (bar_width as f64 * (percentage as f64 / 100.0)) as usize;
    
    let mut bar = String::new();
    bar.push('▐');
    for i in 0..bar_width - 2 {
        if i < filled_width {
            bar.push('█');
        } else {
            bar.push('─');
        }
    }
    bar.push('▌');
    
    let slider_bar = Paragraph::new(bar)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);
    
    f.render_widget(slider_bar, slider_chunks[1]);
}

fn draw_unsupported_editor(f: &mut Frame, area: Rect) {
    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "This setting type is not yet supported for inline editing.",
                Style::default().fg(Color::Red),
            ),
        ]),
        Line::from(""),
        Line::from("Press Enter to open in Windows Settings instead."),
    ];
    
    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    
    f.render_widget(paragraph, area);
}

fn draw_edit_help(f: &mut Frame, area: Rect, editor_type: &EditorType) {
    let help_text = match editor_type {
        EditorType::Toggle => {
            "[Space/Enter] Toggle  [Esc] Cancel  [S] Save"
        }
        EditorType::Dropdown | EditorType::ResolutionPicker => {
            "[↑↓] Navigate  [Enter] Select  [Esc] Cancel  [S] Save"
        }
        EditorType::Slider { .. } => {
            "[←→] Adjust  [Shift+←→] Fine  [Esc] Cancel  [S] Save"
        }
        _ => {
            "[Enter] Open Windows Settings  [Esc] Cancel"
        }
    };
    
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .alignment(Alignment::Center);
    
    f.render_widget(help, area);
}

// Helper function for GCD calculation
fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}