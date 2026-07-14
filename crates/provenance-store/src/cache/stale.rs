use crate::{layout::ProvenanceLayout, state_store::StateStore};
use provenance_core::{Edge, EdgeType, NodeType, ResolutionStatus, Rule, RuleSeverity, ScopeId};
use std::collections::BTreeSet;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Default)]
pub struct StaleOptions {
    pub min_age_days: u32,
    pub rule_severities: Vec<RuleSeverity>,
    pub min_downstream_rules: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct StaleItem {
    pub resolution_id: String,
    pub reason: String,
}

pub fn find_stale(layout: &ProvenanceLayout, scope: &ScopeId) -> anyhow::Result<Vec<StaleItem>> {
    find_stale_with_options(layout, scope, &StaleOptions::default())
}

pub fn find_stale_with_options(
    layout: &ProvenanceLayout,
    scope: &ScopeId,
    options: &StaleOptions,
) -> anyhow::Result<Vec<StaleItem>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    find_stale_at(layout, scope, options, now)
}

fn find_stale_at(
    layout: &ProvenanceLayout,
    scope: &ScopeId,
    options: &StaleOptions,
    now: u64,
) -> anyhow::Result<Vec<StaleItem>> {
    let store = StateStore::new(layout.clone());
    let edges: Vec<_> = store
        .list_edges()?
        .into_iter()
        .filter(|edge| edge.scope_id == *scope)
        .collect();
    let rules = store.list_rules(scope)?;
    let today = i64::try_from(now / 86_400)?;
    Ok(store
        .list_resolutions(scope)?
        .into_iter()
        .filter(|resolution| resolution.status == ResolutionStatus::Approved)
        .filter(|resolution| old_enough(resolution.approved_at, options.min_age_days, now))
        .filter(|resolution| {
            let downstream = downstream_rules(resolution.id.as_str(), &edges, &rules);
            rules_match(&downstream, options)
        })
        .filter_map(|resolution| {
            let reason = if resolution.superseded_by.is_some() {
                Some("approved resolution was superseded")
            } else if resolution
                .review_on
                .as_deref()
                .and_then(parse_iso_date)
                .is_some_and(|review_day| review_day < today)
            {
                Some("approved resolution is past review date")
            } else {
                None
            }?;
            Some(StaleItem {
                resolution_id: resolution.id.as_str().to_string(),
                reason: reason.to_string(),
            })
        })
        .collect())
}

fn old_enough(approved_at: Option<i64>, min_age_days: u32, now: u64) -> bool {
    if min_age_days == 0 {
        return true;
    }
    let Some(mut approved) = approved_at.and_then(|value| u64::try_from(value).ok()) else {
        return false;
    };
    if approved > 10_000_000_000 {
        approved /= 1_000;
    }
    now.saturating_sub(approved) / 86_400 >= u64::from(min_age_days)
}

fn downstream_rules<'a>(resolution_id: &str, edges: &[Edge], rules: &'a [Rule]) -> Vec<&'a Rule> {
    let requirement_ids: BTreeSet<_> = edges
        .iter()
        .filter_map(|edge| {
            let resolves = edge.edge_type == EdgeType::Resolves
                && edge.from_type == NodeType::Resolution
                && edge.from_id.as_str() == resolution_id;
            let needs = edge.edge_type == EdgeType::Needs
                && edge.to_type == NodeType::Resolution
                && edge.to_id.as_str() == resolution_id;
            if resolves {
                Some(edge.to_id.as_str())
            } else if needs {
                Some(edge.from_id.as_str())
            } else {
                None
            }
        })
        .collect();
    rules
        .iter()
        .filter(|rule| {
            edges.iter().any(|edge| {
                edge.edge_type == EdgeType::Produces
                    && edge.to_type == NodeType::Rule
                    && edge.to_id == rule.id
                    && ((edge.from_type == NodeType::Resolution
                        && edge.from_id.as_str() == resolution_id)
                        || (edge.from_type == NodeType::Requirement
                            && requirement_ids.contains(edge.from_id.as_str())))
            })
        })
        .collect()
}

fn rules_match(rules: &[&Rule], options: &StaleOptions) -> bool {
    let selected = rules
        .iter()
        .filter(|rule| {
            options.rule_severities.is_empty() || options.rule_severities.contains(&rule.severity)
        })
        .count();
    selected >= options.min_downstream_rules as usize
        && (options.rule_severities.is_empty() || selected > 0)
}

fn parse_iso_date(value: &str) -> Option<i64> {
    let mut parts = value.split('-');
    let year: i64 = parts.next()?.parse().ok()?;
    let month: i64 = parts.next()?.parse().ok()?;
    let day: i64 = parts.next()?.parse().ok()?;
    if parts.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let adjusted_year = year - i64::from(month <= 2);
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let shifted_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Some(era * 146_097 + day_of_era - 719_468)
}

#[cfg(test)]
mod tests {
    use super::parse_iso_date;

    #[test]
    fn parses_unix_epoch_date() {
        assert_eq!(parse_iso_date("1970-01-01"), Some(0));
        assert!(parse_iso_date("not-a-date").is_none());
    }
}
