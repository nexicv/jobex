//! Per-IP request limits, team-passphrase failures and the rolling-hour spending cap.

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const TEN_MIN: Duration = Duration::from_secs(600);
const DAY: Duration = Duration::from_secs(86_400);
const HOUR: Duration = Duration::from_secs(3_600);

#[derive(Default)]
struct IpState {
    requests: VecDeque<Instant>,
    team_fails: VecDeque<Instant>,
    team_blocked_until: Option<Instant>,
}

pub struct Limiter {
    per_10min: u32,
    per_day: u32,
    team_fail_limit: u32,
    usd_per_hour: f64,
    ips: Mutex<HashMap<String, IpState>>,
    spend: Mutex<VecDeque<(Instant, f64)>>,
}

pub enum Verdict {
    Ok,
    /// Seconds until the visitor may try again.
    TooMany(u64),
}

impl Limiter {
    pub fn new(per_10min: u32, per_day: u32, team_fail_limit: u32, usd_per_hour: f64) -> Self {
        Self { per_10min, per_day, team_fail_limit, usd_per_hour, ips: Mutex::default(), spend: Mutex::default() }
    }

    pub fn hit(&self, ip: &str, now: Instant) -> Verdict {
        let mut ips = self.ips.lock().unwrap();
        let st = ips.entry(ip.to_string()).or_default();
        while st.requests.front().is_some_and(|t| now.duration_since(*t) >= DAY) {
            st.requests.pop_front();
        }
        let recent = st.requests.iter().filter(|t| now.duration_since(**t) < TEN_MIN).count() as u32;
        if recent >= self.per_10min {
            let oldest = st.requests.iter().find(|t| now.duration_since(**t) < TEN_MIN).copied().unwrap_or(now);
            return Verdict::TooMany(TEN_MIN.saturating_sub(now.duration_since(oldest)).as_secs().max(1));
        }
        if st.requests.len() as u32 >= self.per_day {
            let oldest = *st.requests.front().unwrap();
            return Verdict::TooMany(DAY.saturating_sub(now.duration_since(oldest)).as_secs().max(1));
        }
        st.requests.push_back(now);
        Verdict::Ok
    }

    pub fn team_blocked(&self, ip: &str, now: Instant) -> bool {
        let ips = self.ips.lock().unwrap();
        ips.get(ip).and_then(|s| s.team_blocked_until).is_some_and(|until| now < until)
    }

    pub fn team_failed(&self, ip: &str, now: Instant) {
        let mut ips = self.ips.lock().unwrap();
        let st = ips.entry(ip.to_string()).or_default();
        while st.team_fails.front().is_some_and(|t| now.duration_since(*t) >= HOUR) {
            st.team_fails.pop_front();
        }
        st.team_fails.push_back(now);
        if st.team_fails.len() as u32 >= self.team_fail_limit {
            st.team_blocked_until = Some(now + HOUR);
            st.team_fails.clear();
        }
    }

    /// Seconds until spending in the last hour drops under the cap, or None when under it.
    pub fn spend_wait(&self, now: Instant) -> Option<u64> {
        let mut spend = self.spend.lock().unwrap();
        while spend.front().is_some_and(|(t, _)| now.duration_since(*t) >= HOUR) {
            spend.pop_front();
        }
        let mut total: f64 = spend.iter().map(|(_, c)| c).sum();
        if total < self.usd_per_hour {
            return None;
        }
        for (t, c) in spend.iter() {
            total -= c;
            if total < self.usd_per_hour {
                return Some(HOUR.saturating_sub(now.duration_since(*t)).as_secs().max(1));
            }
        }
        Some(HOUR.as_secs())
    }

    pub fn record_spend(&self, usd: f64, now: Instant) {
        if usd > 0.0 {
            self.spend.lock().unwrap().push_back((now, usd));
        }
    }

    pub fn spent_last_hour(&self, now: Instant) -> f64 {
        self.spend.lock().unwrap().iter().filter(|(t, _)| now.duration_since(*t) < HOUR).map(|(_, c)| c).sum()
    }

    /// Drop idle IP entries so memory stays flat.
    pub fn sweep(&self, now: Instant) {
        self.ips.lock().unwrap().retain(|_, s| {
            s.requests.back().is_some_and(|t| now.duration_since(*t) < DAY) || s.team_blocked_until.is_some_and(|u| now < u) || !s.team_fails.is_empty()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn per_ip_window() {
        let l = Limiter::new(3, 100, 5, 1.0);
        let t0 = Instant::now();
        for _ in 0..3 {
            assert!(matches!(l.hit("a", t0), Verdict::Ok));
        }
        assert!(matches!(l.hit("a", t0), Verdict::TooMany(s) if s == 600));
        assert!(matches!(l.hit("b", t0), Verdict::Ok));
        assert!(matches!(l.hit("a", t0 + TEN_MIN), Verdict::Ok));
    }

    #[test]
    fn spend_cap_rolls() {
        let l = Limiter::new(100, 100, 5, 1.0);
        let t0 = Instant::now();
        l.record_spend(0.6, t0);
        assert!(l.spend_wait(t0).is_none());
        l.record_spend(0.5, t0 + Duration::from_secs(600));
        let wait = l.spend_wait(t0 + Duration::from_secs(1200)).unwrap();
        assert_eq!(wait, 2400);
        assert!(l.spend_wait(t0 + HOUR).is_none());
    }

    #[test]
    fn team_block_after_failures() {
        let l = Limiter::new(100, 100, 3, 1.0);
        let t0 = Instant::now();
        for _ in 0..2 {
            l.team_failed("x", t0);
        }
        assert!(!l.team_blocked("x", t0));
        l.team_failed("x", t0);
        assert!(l.team_blocked("x", t0));
        assert!(!l.team_blocked("x", t0 + HOUR));
    }
}
