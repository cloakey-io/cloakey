//! Ratatui interactive main menu.
//!
//! Displayed when `cloak` is run with no arguments.
//! Provides a numbered menu with keyboard navigation.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
};

use cloakey_core::LockState;

/// The main menu items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuItem {
    LockKeyboard,
    LockMouse,
    LockBoth,
    GhostMode,
    TimedLock,
    Settings,
    Help,
    About,
    Quit,
}

impl MenuItem {
    pub fn all() -> Vec<MenuItem> {
        vec![
            MenuItem::LockKeyboard,
            MenuItem::LockMouse,
            MenuItem::LockBoth,
            MenuItem::GhostMode,
            MenuItem::TimedLock,
            MenuItem::Settings,
            MenuItem::Help,
            MenuItem::About,
            MenuItem::Quit,
        ]
    }

    pub fn number(&self) -> &'static str {
        match self {
            MenuItem::LockKeyboard => "1",
            MenuItem::LockMouse => "2",
            MenuItem::LockBoth => "3",
            MenuItem::GhostMode => "4",
            MenuItem::TimedLock => "5",
            MenuItem::Settings => "6",
            MenuItem::Help => "7",
            MenuItem::About => "8",
            MenuItem::Quit => "Q",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            MenuItem::LockKeyboard => "Lock Keyboard",
            MenuItem::LockMouse => "Lock Mouse",
            MenuItem::LockBoth => "Lock Keyboard & Mouse",
            MenuItem::GhostMode => "Ghost Mode",
            MenuItem::TimedLock => "Timed Lock",
            MenuItem::Settings => "Settings",
            MenuItem::Help => "Help",
            MenuItem::About => "About",
            MenuItem::Quit => "Exit",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            MenuItem::LockKeyboard => "Block all keyboard input. Mouse stays active.",
            MenuItem::LockMouse => "Block mouse clicks, movement, scroll. Keyboard stays active.",
            MenuItem::LockBoth => "Block all keyboard and mouse input.",
            MenuItem::GhostMode => "Cursor moves freely. Clicks and keyboard blocked.",
            MenuItem::TimedLock => "Lock for a set duration, then auto-unlock.",
            MenuItem::Settings => "Configure overlay, shortcuts, and startup.",
            MenuItem::Help => "Show all commands and keyboard shortcuts.",
            MenuItem::About => "Version, license, and links.",
            MenuItem::Quit => "Exit CloaKey.",
        }
    }

    /// Try to match a key character to a menu item.
    pub fn from_key(ch: char) -> Option<MenuItem> {
        match ch.to_ascii_lowercase() {
            '1' => Some(MenuItem::LockKeyboard),
            '2' => Some(MenuItem::LockMouse),
            '3' => Some(MenuItem::LockBoth),
            '4' => Some(MenuItem::GhostMode),
            '5' => Some(MenuItem::TimedLock),
            '6' => Some(MenuItem::Settings),
            '7' => Some(MenuItem::Help),
            '8' => Some(MenuItem::About),
            'q' => Some(MenuItem::Quit),
            _ => None,
        }
    }

    pub fn index_from_key(ch: char) -> Option<usize> {
        Self::all().iter().position(|m| {
            m.number()
                .chars()
                .next()
                .map(|c| c.to_ascii_lowercase() == ch.to_ascii_lowercase())
                .unwrap_or(false)
        })
    }
}

/// Render the main interactive menu.
pub fn draw_main_menu(
    frame: &mut Frame,
    list_state: &mut ListState,
    current_state: &LockState,
    version: &str,
) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Header / logo area
            Constraint::Min(1),     // Menu items
            Constraint::Length(4),  // Status bar
        ])
        .split(area);

    // --- Header ---
    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    let header_text = vec![
        Line::from(Span::styled(
            "  🔑  CloaKey",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  v{}  ·  Protect the workflow. Don't stop the workflow.", version),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Emergency: ", Style::default().fg(Color::Yellow)),
            Span::styled("CTRL + ALT + SHIFT (hold 2s)", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("  ·  Uncloak: ", Style::default().fg(Color::Cyan)),
            Span::styled("CTRL + ALT + U", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
    ];

    let header = Paragraph::new(header_text)
        .block(header_block)
        .alignment(Alignment::Left);
    frame.render_widget(header, chunks[0]);

    // --- Menu Items ---
    let menu_block = Block::default()
        .title(Span::styled(
            " Select a protection mode ",
            Style::default().fg(Color::White),
        ))
        .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
        .border_style(Style::default().fg(Color::Cyan));

    let items: Vec<ListItem> = MenuItem::all()
        .iter()
        .map(|item| {
            let is_quit = *item == MenuItem::Quit;
            let style = if is_quit {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("  [{}]  ", item.number()),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(item.label(), style.add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("  — {}", item.description()),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(menu_block)
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, chunks[1], list_state);

    // --- Status Bar ---
    let status_color = match current_state {
        LockState::Unlocked => Color::Green,
        _ => Color::Red,
    };

    let status_text = vec![
        Line::from(vec![
            Span::styled("  Status: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                current_state.to_string(),
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(Span::styled(
            "  Use ↑↓ arrows or number keys to navigate, Enter to select",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let status_bar = Paragraph::new(status_text).block(
        Block::default()
            .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(status_bar, chunks[2]);
}

/// Render the help screen.
pub fn draw_help_screen(frame: &mut Frame) {
    let area = frame.area();

    let help_text = vec![
        Line::from(Span::styled(
            " CloaKey — Command Reference",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled("COMMANDS:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  cloak                    Open interactive menu"),
        Line::from("  cloak keyboard           Lock keyboard only"),
        Line::from("  cloak mouse              Lock mouse only"),
        Line::from("  cloak full               Lock keyboard and mouse"),
        Line::from("  cloak ghost              Ghost Mode"),
        Line::from("  cloak timer <duration>   Timed lock (e.g. 60, 5m, 1h)"),
        Line::from("  cloak status             Show current protection state"),
        Line::from("  cloak unlock             Remove active protection"),
        Line::from("  cloak settings           Open settings"),
        Line::from("  cloak startup            Configure startup integration"),
        Line::from("  cloak reset              Reset all settings to defaults"),
        Line::from("  cloak help               Show this help"),
        Line::from("  cloak about              Version and license info"),
        Line::from(""),
        Line::from(Span::styled("SHORTCUTS:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  CTRL + ALT + L           Lock all"),
        Line::from("  CTRL + ALT + K           Lock keyboard"),
        Line::from("  CTRL + ALT + M           Lock mouse"),
        Line::from("  CTRL + ALT + G           Ghost Mode"),
        Line::from("  CTRL + ALT + SHIFT       Emergency unlock (hold 2 seconds)"),
        Line::from("  CTRL + ALT + U           Uncloak (immediate unlock)"),
        Line::from(""),
        Line::from(Span::styled("LINKS:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  Website:  https://cloakey.io"),
        Line::from("  GitHub:   https://github.com/cloakey/cloakey"),
        Line::from(""),
        Line::from(Span::styled(
            "  Press Esc or Q to return to the menu",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" CloaKey Help ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(ratatui::widgets::Wrap { trim: false });

    frame.render_widget(help, area);
}

/// Render the about screen.
pub fn draw_about_screen(frame: &mut Frame, version: &str) {
    let area = frame.area();

    let about_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  🔑  CloaKey",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  Version {}  ·  MIT License", version),
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  \"Protect the workflow. Don't stop the workflow.\"",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from("  CloaKey is a lightweight desktop workflow protection utility."),
        Line::from("  It suppresses keyboard and mouse input while keeping all"),
        Line::from("  applications, AI agents, downloads, and processes running."),
        Line::from(""),
        Line::from(Span::styled("PRIVACY:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  Input may be blocked. Input is NEVER recorded."),
        Line::from("  No telemetry. No analytics. No accounts."),
        Line::from("  Open source — inspect the code at any time."),
        Line::from(""),
        Line::from(Span::styled("LINKS:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  Website:  https://cloakey.io"),
        Line::from("  GitHub:   https://github.com/cloakey/cloakey"),
        Line::from("  License:  MIT (https://opensource.org/licenses/MIT)"),
        Line::from(""),
        Line::from(Span::styled(
            "  Press Esc or Q to return to the menu",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let about = Paragraph::new(about_text)
        .block(
            Block::default()
                .title(" About CloaKey ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(ratatui::widgets::Wrap { trim: false });

    frame.render_widget(about, area);
}
