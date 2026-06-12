//! Ratatui settings submenu.
//!
//! Allows the user to configure:
//! - Overlay mode (None / Minimal / Standard)
//! - Emergency unlock hold duration
//! - Startup integration (run on startup, start minimized)

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use cloakey_settings::{CloaKeyConfig, OverlayMode};

/// Which settings item is currently selected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsItem {
    OverlayMode,
    UnlockHoldDuration,
    RunOnStartup,
    StartMinimized,
    BackToMenu,
}

impl SettingsItem {
    pub fn all() -> Vec<SettingsItem> {
        vec![
            SettingsItem::OverlayMode,
            SettingsItem::UnlockHoldDuration,
            SettingsItem::RunOnStartup,
            SettingsItem::StartMinimized,
            SettingsItem::BackToMenu,
        ]
    }

    pub fn label(&self, config: &CloaKeyConfig) -> String {
        match self {
            SettingsItem::OverlayMode => {
                format!("Overlay Mode          [ {} ]", config.general.overlay)
            }
            SettingsItem::UnlockHoldDuration => format!(
                "Unlock Hold Duration  [ {}ms ]",
                config.shortcuts.emergency_unlock_hold_ms
            ),
            SettingsItem::RunOnStartup => format!(
                "Run On Startup        [ {} ]",
                if config.startup.run_on_startup {
                    "ON "
                } else {
                    "OFF"
                }
            ),
            SettingsItem::StartMinimized => format!(
                "Start Minimized       [ {} ]",
                if config.startup.start_minimized {
                    "ON "
                } else {
                    "OFF"
                }
            ),
            SettingsItem::BackToMenu => "← Back to Main Menu".to_string(),
        }
    }
}

/// Render the settings submenu.
pub fn draw_settings_menu(frame: &mut Frame, config: &CloaKeyConfig, list_state: &mut ListState) {
    let area = frame.area();

    let block = Block::default()
        .title(Span::styled(
            " ⚙  CloaKey Settings ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Blue))
        .style(Style::default().bg(Color::Black));

    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // header
            Constraint::Min(1),    // list
            Constraint::Length(3), // footer
        ])
        .margin(2)
        .split(area);

    let header = Paragraph::new("Use ↑↓ to navigate, Enter to toggle/select, Esc to go back")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(header, chunks[0]);

    let items: Vec<ListItem> = SettingsItem::all()
        .iter()
        .map(|item| {
            let label = item.label(config);
            let is_back = *item == SettingsItem::BackToMenu;
            ListItem::new(Line::from(Span::styled(
                format!("  {}", label),
                if is_back {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
                },
            )))
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, chunks[1], list_state);

    let footer = Paragraph::new("Settings are saved automatically")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, chunks[2]);
}

/// Cycle the overlay mode to the next option.
pub fn cycle_overlay_mode(config: &mut CloaKeyConfig) {
    config.general.overlay = match config.general.overlay {
        OverlayMode::None => OverlayMode::Minimal,
        OverlayMode::Minimal => OverlayMode::Standard,
        OverlayMode::Standard | OverlayMode::Prominent => OverlayMode::None,
    };
}
