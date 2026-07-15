use super::{DownstreamRuleQuery, RulePolicy};
use crate::{
    layout::ProvenanceLayout,
    state_store::{ScopeSnapshot, StateStore},
};
use provenance_core::{ResolutionStatus, RuleSeverity, ScopeId};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Default)]
pub struct StaleResolutionPolicy {
    pub min_age_days: u32,
    pub rule_severities: Vec<RuleSeverity>,
    pub min_downstream_rules: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct StaleResolution {
    pub resolution_id: String,
    pub reason: String,
}

pub type StaleOptions = StaleResolutionPolicy;
pub type StaleItem = StaleResolution;

#[derive(Clone, Copy)]
struct UnixMillis(u64);

#[derive(Clone, Copy)]
struct CalendarDay(i64);

pub fn find_stale(
    layout: &ProvenanceLayout,
    scope: &ScopeId,
) -> anyhow::Result<Vec<StaleResolution>> {
    find_stale_with_policy(layout, scope, &StaleResolutionPolicy::default())
}

pub fn find_stale_with_policy(
    layout: &ProvenanceLayout,
    scope: &ScopeId,
    policy: &StaleResolutionPolicy,
) -> anyhow::Result<Vec<StaleResolution>> {
    let snapshot = StateStore::new(layout.clone()).scope_snapshot(scope)?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let now = UnixMillis(u64::try_from(now.as_millis())?);
    Ok(find_stale_in_snapshot(&snapshot, policy, now))
}

pub fn find_stale_with_options(
    layout: &ProvenanceLayout,
    scope: &ScopeId,
    options: &StaleOptions,
) -> anyhow::Result<Vec<StaleItem>> {
    find_stale_with_policy(layout, scope, options)
}

fn find_stale_in_snapshot(
    snapshot: &ScopeSnapshot,
    policy: &StaleResolutionPolicy,
    now: UnixMillis,
) -> Vec<StaleResolution> {
    let today = CalendarDay(i64::try_from(now.0 / 86_400_000).unwrap_or(i64::MAX));
    let query = DownstreamRuleQuery::new(
        &snapshot.scope,
        &snapshot.edges,
        &snapshot.requirements,
        &snapshot.resolutions,
        &snapshot.rules,
    );
    let rule_policy = RulePolicy {
        severities: policy.rule_severities.clone(),
        minimum: policy.min_downstream_rules,
    };
    snapshot
        .resolutions
        .iter()
        .filter(|resolution| resolution.status == ResolutionStatus::Approved)
        .filter(|resolution| old_enough(resolution.approved_at, policy.min_age_days, now))
        .filter(|resolution| rule_policy.matches(&query.for_resolution(&resolution.id)))
        .filter_map(|resolution| {
            let reason = if resolution.superseded_by.is_some() {
                Some("approved resolution was superseded")
            } else if resolution
                .review_on
                .as_deref()
                .and_then(CalendarDay::parse)
                .is_some_and(|review_day| review_day.0 < today.0)
            {
                Some("approved resolution is past review date")
            } else {
                None
            }?;
            Some(StaleResolution {
                resolution_id: resolution.id.as_str().to_string(),
                reason: reason.to_string(),
            })
        })
        .collect()
}

pub(super) fn find_stale_in_current_snapshot(
    snapshot: &ScopeSnapshot,
) -> anyhow::Result<Vec<StaleResolution>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let now = UnixMillis(u64::try_from(now.as_millis())?);
    Ok(find_stale_in_snapshot(
        snapshot,
        &StaleResolutionPolicy::default(),
        now,
    ))
}

fn old_enough(approved_at: Option<i64>, min_age_days: u32, now: UnixMillis) -> bool {
    if min_age_days == 0 {
        return true;
    }
    let Some(approved) = approved_at.and_then(|value| u64::try_from(value).ok()) else {
        return false;
    };
    now.0.saturating_sub(approved) / 86_400_000 >= u64::from(min_age_days)
}

impl CalendarDay {
    fn parse(value: &str) -> Option<Self> {
        let bytes = value.as_bytes();
        if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
            return None;
        }
        let mut parts = value.split('-');
        let year: i64 = parts.next()?.parse().ok()?;
        let month: i64 = parts.next()?.parse().ok()?;
        let day: i64 = parts.next()?.parse().ok()?;
        if parts.next().is_some() || !(1..=12).contains(&month) {
            return None;
        }
        let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        let max_day = match month {
            2 if leap => 29,
            2 => 28,
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        };
        if !(1..=max_day).contains(&day) {
            return None;
        }
        let adjusted_year = year - i64::from(month <= 2);
        let era = adjusted_year.div_euclid(400);
        let year_of_era = adjusted_year - era * 400;
        let shifted_month = month + if month > 2 { -3 } else { 9 };
        let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        Some(Self(era * 146_097 + day_of_era - 719_468))
    }
}

#[cfg(test)]
mod tests {
    use super::CalendarDay;

    #[test]
    fn parses_only_valid_calendar_dates() {
        assert_eq!(CalendarDay::parse("1970-01-01").map(|day| day.0), Some(0));
        assert!(CalendarDay::parse("2025-02-29").is_none());
        assert!(CalendarDay::parse("not-a-date").is_none());
    }
}
