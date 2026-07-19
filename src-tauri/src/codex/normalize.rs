use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RawQuotaWindow {
    pub used_percent: Option<f64>,
    pub window_duration_mins: Option<i64>,
    pub resets_at: Option<i64>,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RawRateLimits {
    pub primary: Option<RawQuotaWindow>,
    pub secondary: Option<RawQuotaWindow>,
    pub rate_limit_reached_type: Option<String>,
    #[serde(flatten)]
    pub supplemental: std::collections::HashMap<String, Value>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaWindow {
    pub used_percent: Option<f64>,
    pub remaining_percent: Option<f64>,
    pub window_duration_mins: Option<i64>,
    pub label: String,
    pub resets_at: Option<i64>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexUsageState {
    pub status: String,
    pub primary: Option<QuotaWindow>,
    pub secondary: Option<QuotaWindow>,
    pub plan_type: Option<String>,
    pub rate_limit_reached_type: Option<String>,
    pub reset_credit_count: Option<i64>,
    pub last_successful_refresh_at: Option<i64>,
    pub error_message: Option<String>,
}
impl Default for CodexUsageState {
    fn default() -> Self {
        Self {
            status: "starting".into(),
            primary: None,
            secondary: None,
            plan_type: None,
            rate_limit_reached_type: None,
            reset_credit_count: None,
            last_successful_refresh_at: None,
            error_message: None,
        }
    }
}

pub fn remaining(used: f64) -> f64 {
    (100.0 - used).clamp(0.0, 100.0)
}
pub fn duration_label(minutes: Option<i64>) -> String {
    match minutes {
        Some(m) if m > 0 && m % 1440 == 0 => format!("{}d", m / 1440),
        Some(m) if m > 0 && m % 60 == 0 => format!("{}h", m / 60),
        Some(m) if m > 60 => format!("{}h{}m", m / 60, m % 60),
        Some(m) if m > 0 => format!("{}m", m),
        _ => "-".into(),
    }
}
fn quota(raw: RawQuotaWindow) -> QuotaWindow {
    QuotaWindow {
        remaining_percent: raw.used_percent.map(remaining),
        used_percent: raw.used_percent,
        label: duration_label(raw.window_duration_mins),
        window_duration_mins: raw.window_duration_mins,
        resets_at: raw.resets_at,
    }
}
pub fn merge_window(current: &mut Option<RawQuotaWindow>, update: Option<RawQuotaWindow>) {
    if let Some(update) = update {
        let target = current.get_or_insert_with(Default::default);
        if update.used_percent.is_some() {
            target.used_percent = update.used_percent
        };
        if update.window_duration_mins.is_some() {
            target.window_duration_mins = update.window_duration_mins
        };
        if update.resets_at.is_some() {
            target.resets_at = update.resets_at
        };
    }
}
pub fn merge_limits(current: &mut RawRateLimits, update: RawRateLimits) {
    merge_window(&mut current.primary, update.primary);
    merge_window(&mut current.secondary, update.secondary);
    if update.rate_limit_reached_type.is_some() {
        current.rate_limit_reached_type = update.rate_limit_reached_type;
    }
    current.supplemental.extend(update.supplemental);
}
pub fn apply_limits(state: &mut CodexUsageState, raw: &RawRateLimits, credits: Option<i64>) {
    state.primary = raw.primary.clone().map(quota);
    state.secondary = raw.secondary.clone().map(quota);
    state.rate_limit_reached_type = raw.rate_limit_reached_type.clone();
    state.reset_credit_count = credits;
    state.last_successful_refresh_at = Some(chrono::Utc::now().timestamp());
    state.status = "connected".into();
    state.error_message = None;
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn remaining_is_clamped() {
        assert_eq!(remaining(0.), 100.);
        assert_eq!(remaining(25.), 75.);
        assert_eq!(remaining(100.), 0.);
        assert_eq!(remaining(120.), 0.);
        assert_eq!(remaining(-10.), 100.);
    }
    #[test]
    fn labels() {
        assert_eq!(duration_label(Some(15)), "15m");
        assert_eq!(duration_label(Some(60)), "1h");
        assert_eq!(duration_label(Some(90)), "1h30m");
        assert_eq!(duration_label(Some(300)), "5h");
        assert_eq!(duration_label(Some(1440)), "1d");
        assert_eq!(duration_label(Some(10080)), "7d");
    }
    #[test]
    fn sparse_merge_preserves_fields() {
        let mut old = RawRateLimits {
            primary: Some(RawQuotaWindow {
                used_percent: Some(20.),
                window_duration_mins: Some(300),
                resets_at: Some(1000),
            }),
            secondary: Some(RawQuotaWindow {
                used_percent: Some(40.),
                window_duration_mins: Some(10080),
                resets_at: Some(2000),
            }),
            ..Default::default()
        };
        merge_limits(
            &mut old,
            RawRateLimits {
                primary: Some(RawQuotaWindow {
                    used_percent: Some(25.),
                    ..Default::default()
                }),
                ..Default::default()
            },
        );
        assert_eq!(old.primary.unwrap().resets_at, Some(1000));
        assert_eq!(old.secondary.unwrap().window_duration_mins, Some(10080));
    }
}
