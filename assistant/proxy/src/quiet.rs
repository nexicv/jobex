//! Weekday quiet hours (UTC) when the assistant makes no DeepSeek calls.

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Datelike, NaiveTime, Utc, Weekday};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    pub start: NaiveTime,
    pub end: NaiveTime,
}

pub fn parse_windows(specs: &[String]) -> Result<Vec<Window>> {
    let mut out = Vec::new();
    for spec in specs {
        let (a, b) = spec.split_once('-').with_context(|| format!("quiet window {spec:?} must look like HH:MM-HH:MM"))?;
        let start = NaiveTime::parse_from_str(a.trim(), "%H:%M").with_context(|| format!("bad start in {spec:?}"))?;
        let end = NaiveTime::parse_from_str(b.trim(), "%H:%M").with_context(|| format!("bad end in {spec:?}"))?;
        if end <= start {
            bail!("quiet window {spec:?} must end after it starts on the same day");
        }
        out.push(Window { start, end });
    }
    out.sort_by_key(|w| w.start);
    Ok(out)
}

/// When `now` falls in a quiet window, the moment the assistant reopens.
pub fn reopens_at(now: DateTime<Utc>, windows: &[Window]) -> Option<DateTime<Utc>> {
    if matches!(now.weekday(), Weekday::Sat | Weekday::Sun) {
        return None;
    }
    let t = now.time();
    let w = windows.iter().find(|w| t >= w.start && t < w.end)?;
    let mut end = now.date_naive().and_time(w.end).and_utc();
    // Back-to-back windows reopen only after the last one.
    while let Some(next) = windows.iter().find(|n| n.start == end.time()) {
        end = now.date_naive().and_time(next.end).and_utc();
    }
    Some(end)
}

/// Whole minutes until `at`, rounded up, never below 1.
pub fn minutes_until(now: DateTime<Utc>, at: DateTime<Utc>) -> i64 {
    let secs = (at - now).num_seconds().max(0);
    ((secs + 59) / 60).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn w() -> Vec<Window> {
        parse_windows(&["06:00-10:00".into(), "01:00-04:00".into()]).unwrap()
    }

    #[test]
    fn weekday_morning_is_closed_until_ten() {
        // Friday 6 Nov 2026, 08:36 UTC.
        let now = Utc.with_ymd_and_hms(2026, 11, 6, 8, 36, 0).unwrap();
        let at = reopens_at(now, &w()).unwrap();
        assert_eq!(at, Utc.with_ymd_and_hms(2026, 11, 6, 10, 0, 0).unwrap());
        assert_eq!(minutes_until(now, at), 84);
    }

    #[test]
    fn edges_and_gaps() {
        let ws = w();
        assert!(reopens_at(Utc.with_ymd_and_hms(2026, 11, 6, 10, 0, 0).unwrap(), &ws).is_none());
        assert!(reopens_at(Utc.with_ymd_and_hms(2026, 11, 6, 5, 0, 0).unwrap(), &ws).is_none());
        assert!(reopens_at(Utc.with_ymd_and_hms(2026, 11, 6, 6, 0, 0).unwrap(), &ws).is_some());
        assert!(reopens_at(Utc.with_ymd_and_hms(2026, 11, 6, 1, 0, 0).unwrap(), &ws).is_some());
        assert!(reopens_at(Utc.with_ymd_and_hms(2026, 11, 6, 0, 59, 0).unwrap(), &ws).is_none());
    }

    #[test]
    fn weekends_are_open() {
        // Saturday 7 Nov 2026.
        assert!(reopens_at(Utc.with_ymd_and_hms(2026, 11, 7, 8, 0, 0).unwrap(), &w()).is_none());
    }

    #[test]
    fn back_to_back_windows_chain() {
        let ws = parse_windows(&["06:00-08:00".into(), "08:00-10:00".into()]).unwrap();
        let at = reopens_at(Utc.with_ymd_and_hms(2026, 11, 6, 7, 0, 0).unwrap(), &ws).unwrap();
        assert_eq!(at, Utc.with_ymd_and_hms(2026, 11, 6, 10, 0, 0).unwrap());
    }

    #[test]
    fn rejects_bad_windows() {
        assert!(parse_windows(&["10:00-06:00".into()]).is_err());
        assert!(parse_windows(&["nope".into()]).is_err());
    }
}
