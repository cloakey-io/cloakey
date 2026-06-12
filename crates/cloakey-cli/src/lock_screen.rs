//! Ratatui lock-active screen.
//!
//! Displayed while a lock is active. Shows:
//! - Current lock mode with icon
//! - What is blocked vs allowed
//! - Timer countdown (for timed locks)
//! - Emergency unlock instructions (ALWAYS visible)

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use cloakey_core::{LockMode, LockState};

/// Renders the lock-active screen.
///
/// This is displayed after the user activates a lock, in place of the main menu.
pub fn draw_lock_screen(
    frame: &mut Frame,
    state: &LockState,
    mode: &LockMode,
    timer_display: Option<&str>,
    elapsed_secs: u64,
) {
    let area = frame.area();

    // Outer block
    let block = Block::default()
        .title(Span::styled(
            " 🔑 CloaKey — Protection Active ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Yellow))
        .style(Style::default().bg(Color::Black));

    frame.render_widget(block, area);

    // Inner layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Mode name
            Constraint::Length(4), // Blocked/allowed status
            Constraint::Length(3), // Timer (if active)
            Constraint::Min(0),    // Spacer
            Constraint::Length(4), // Emergency unlock banner
        ])
        .margin(2)
        .split(area);

    // --- Mode Title ---
    let (mode_icon, mode_color) = match state {
        LockState::KeyboardLocked => ("🔒 KEYBOARD LOCKED", Color::Red),
        LockState::MouseLocked => ("🔒 MOUSE LOCKED", Color::Red),
        LockState::FullyLocked => ("🔒 FULLY LOCKED", Color::Red),
        LockState::GhostMode => ("👻 GHOST MODE", Color::Magenta),
        LockState::TimedLock => ("⏱  TIMED LOCK", Color::Yellow),
        LockState::Unlocked => ("✓ UNLOCKED", Color::Green),
    };

    let mode_text = Paragraph::new(mode_icon)
        .alignment(Alignment::Center)
        .style(Style::default().fg(mode_color).add_modifier(Modifier::BOLD));
    frame.render_widget(mode_text, chunks[0]);

    // --- Status Details ---
    let keyboard_status = if mode.keyboard_blocked {
        Span::styled("  ❌ Keyboard: BLOCKED", Style::default().fg(Color::Red))
    } else {
        Span::styled("  ✅ Keyboard: Active", Style::default().fg(Color::Green))
    };
    let mouse_move_status = if mode.mouse_movement_blocked {
        Span::styled(
            "  ❌ Mouse Movement: BLOCKED",
            Style::default().fg(Color::Red),
        )
    } else {
        Span::styled(
            "  ✅ Mouse Movement: Active",
            Style::default().fg(Color::Green),
        )
    };
    let mouse_click_status = if mode.mouse_clicks_blocked {
        Span::styled(
            "  ❌ Mouse Clicks: BLOCKED",
            Style::default().fg(Color::Red),
        )
    } else {
        Span::styled(
            "  ✅ Mouse Clicks: Active",
            Style::default().fg(Color::Green),
        )
    };

    let status_block = Paragraph::new(vec![
        Line::from(keyboard_status),
        Line::from(mouse_move_status),
        Line::from(mouse_click_status),
    ])
    .block(
        Block::default()
            .title(" Protection Status ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(status_block, chunks[1]);

    // --- Timer ---
    if let Some(timer) = timer_display {
        let timer_text = Paragraph::new(format!("⏱  Time remaining: {}", timer))
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
        frame.render_widget(timer_text, chunks[2]);
    } else {
        let elapsed_text = Paragraph::new(format!("Active for: {}s", elapsed_secs))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(elapsed_text, chunks[2]);
    }

    // --- Emergency Unlock Banner (always visible) ---
    let unlock_banner = Paragraph::new(vec![
        Line::from(Span::styled(
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "  EMERGENCY UNLOCK:  CTRL + ALT + SHIFT  (hold 2 seconds)",
            Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(unlock_banner, chunks[4]);
}
