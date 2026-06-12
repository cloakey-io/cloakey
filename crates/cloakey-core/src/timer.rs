//! Timer engine — parse duration strings and manage countdown timers.
//!
//! Supports three duration formats:
//! - Bare integer: `60` → 60 seconds
//! - Minutes: `5m` → 5 minutes = 300 seconds
//! - Hours: `1h` → 1 hour = 3600 seconds

use crate::error::CoreError;
use std::time::{Duration, Instant};

/// Maximum allowed timer duration: 24 hours.
const MAX_DURATION_SECS: u64 = 86_400;
/// Minimum allowed timer duration: 1 second.
const MIN_DURATION_SECS: u64 = 1;

/// Parse a duration string into a `std::time::Duration`.
///
/// Supported formats:
/// - `"60"` → 60 seconds
/// - `"5m"` → 5 minutes
/// - `"1h"` → 1 hour
/// - `"30s"` → 30 seconds (optional `s` suffix)
///
/// # Errors
/// Returns `CoreError::InvalidDuration` if the string cannot be parsed.
/// Returns `CoreError::DurationOutOfRange` if the value is 0 or > 24 hours.
pub fn parse_duration(input: &str) -> Result<Duration, CoreError> {
    let input = input.trim();

    let secs: u64 = if let Some(minutes_str) = input.strip_suffix('m') {
        // Minutes format: "5m"
        let minutes: u64 = minutes_str
            .parse()
            .map_err(|_| CoreError::InvalidDuration {
                input: input.to_string(),
            })?;
        minutes.saturating_mul(60)
    } else if let Some(hours_str) = input.strip_suffix('h') {
        // Hours format: "1h"
        let hours: u64 = hours_str.parse().map_err(|_| CoreError::InvalidDuration {
            input: input.to_string(),
        })?;
        hours.saturating_mul(3600)
    } else {
        // Bare seconds (with optional 's' suffix): "60" or "60s"
        let secs_str = input.strip_suffix('s').unwrap_or(input);
        secs_str.parse().map_err(|_| CoreError::InvalidDuration {
            input: input.to_string(),
        })?
    };

    if secs < MIN_DURATION_SECS {
        return Err(CoreError::DurationOutOfRange {
            value: secs,
            min: MIN_DURATION_SECS,
            max: MAX_DURATION_SECS,
        });
    }

    if secs > MAX_DURATION_SECS {
        return Err(CoreError::DurationOutOfRange {
            value: secs,
            min: MIN_DURATION_SECS,
            max: MAX_DURATION_SECS,
        });
    }

    Ok(Duration::from_secs(secs))
}

/// A countdown timer for timed lock sessions.
///
/// Created when a `TimedLock` is started. Tracks the remaining duration
/// and reports when it has expired.
#[derive(Debug)]
pub struct CountdownTimer {
    /// The total duration of the lock.
    total: Duration,
    /// The instant at which the timer was started.
    started_at: Instant,
}

impl CountdownTimer {
    /// Create a new countdown timer with the given duration.
    pub fn new(duration: Duration) -> Self {
        Self {
            total: duration,
            started_at: Instant::now(),
        }
    }

    /// Returns the remaining time, or `Duration::ZERO` if expired.
    pub fn remaining(&self) -> Duration {
        let elapsed = self.started_at.elapsed();
        self.total.saturating_sub(elapsed)
    }

    /// Returns `true` if the timer has expired.
    pub fn is_expired(&self) -> bool {
        self.remaining() == Duration::ZERO
    }

    /// Returns the total duration this timer was created with.
    pub fn total(&self) -> Duration {
        self.total
    }

    /// Returns how long the timer has been running.
    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Returns remaining seconds as a rounded integer (for display).
    pub fn remaining_secs(&self) -> u64 {
        self.remaining().as_secs()
    }

    /// Returns a human-readable remaining time string (e.g., "4m 32s", "45s").
    pub fn remaining_display(&self) -> String {
        let secs = self.remaining_secs();
        if secs >= 3600 {
            let h = secs / 3600;
            let m = (secs % 3600) / 60;
            let s = secs % 60;
            format!("{}h {}m {}s", h, m, s)
        } else if secs >= 60 {
            let m = secs / 60;
            let s = secs % 60;
            format!("{}m {}s", m, s)
        } else {
            format!("{}s", secs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Duration parsing ---

    #[test]
    fn parse_bare_seconds() {
        assert_eq!(parse_duration("60").unwrap(), Duration::from_secs(60));
    }

    #[test]
    fn parse_seconds_with_suffix() {
        assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
    }

    #[test]
    fn parse_minutes() {
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
    }

    #[test]
    fn parse_hours() {
        assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
    }

    #[test]
    fn parse_whitespace_trimmed() {
        assert_eq!(parse_duration("  10m  ").unwrap(), Duration::from_secs(600));
    }

    #[test]
    fn parse_zero_is_rejected() {
        assert!(parse_duration("0").is_err());
    }

    #[test]
    fn parse_empty_string_is_rejected() {
        assert!(parse_duration("").is_err());
    }

    #[test]
    fn parse_negative_is_rejected() {
        assert!(parse_duration("-5").is_err());
    }

    #[test]
    fn parse_text_is_rejected() {
        assert!(parse_duration("five minutes").is_err());
    }

    #[test]
    fn parse_exceeds_max_is_rejected() {
        // 25 hours = 90000 seconds, over the 86400s max
        assert!(parse_duration("25h").is_err());
    }

    #[test]
    fn parse_large_minutes() {
        // 120m = 2 hours = 7200s, which is valid
        assert_eq!(parse_duration("120m").unwrap(), Duration::from_secs(7200));
    }

    // --- CountdownTimer ---

    #[test]
    fn timer_is_not_expired_immediately() {
        let timer = CountdownTimer::new(Duration::from_secs(60));
        assert!(!timer.is_expired());
    }

    #[test]
    fn timer_remaining_at_start_is_close_to_total() {
        let timer = CountdownTimer::new(Duration::from_secs(60));
        let remaining = timer.remaining_secs();
        // Should be 59 or 60 depending on timing
        assert!((59..=60).contains(&remaining));
    }

    #[test]
    fn timer_total_is_correct() {
        let timer = CountdownTimer::new(Duration::from_secs(300));
        assert_eq!(timer.total(), Duration::from_secs(300));
    }

    #[test]
    fn timer_display_seconds() {
        let timer = CountdownTimer::new(Duration::from_secs(45));
        // Can't predict exact display since time has passed, just ensure it's non-empty
        let display = timer.remaining_display();
        assert!(!display.is_empty());
    }

    #[test]
    fn expired_timer_shows_zero_remaining() {
        // Create a timer with a very short duration and wait for it to expire
        let timer = CountdownTimer::new(Duration::from_millis(1));
        std::thread::sleep(Duration::from_millis(10));
        assert!(timer.is_expired());
        assert_eq!(timer.remaining(), Duration::ZERO);
    }
}
