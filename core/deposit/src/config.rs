use chrono::{DateTime, Duration, Utc};
use domain_config::{DomainConfigError, define_exposed_config};

const DEFAULT_INACTIVE_THRESHOLD_DAYS: u64 = 365;
const DEFAULT_ESCHEATABLE_THRESHOLD_DAYS: u64 = 3650;

define_exposed_config! {
    /// Number of days without account activity before a deposit account is classified as inactive.
    pub struct DepositActivityInactiveThresholdDays(u64);
    spec {
        key: "deposit-activity-inactive-threshold-days";
        default: || Some(DEFAULT_INACTIVE_THRESHOLD_DAYS);
        validate: |value: &u64| validate_positive_threshold(
            "deposit-activity-inactive-threshold-days",
            *value,
        );
    }
}

define_exposed_config! {
    /// Number of days without account activity before a deposit account is classified as escheatable.
    pub struct DepositActivityEscheatableThresholdDays(u64);
    spec {
        key: "deposit-activity-escheatable-threshold-days";
        default: || Some(DEFAULT_ESCHEATABLE_THRESHOLD_DAYS);
        validate: |value: &u64| validate_positive_threshold(
            "deposit-activity-escheatable-threshold-days",
            *value,
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DepositActivityThresholds {
    inactive_threshold_days: u64,
    escheatable_threshold_days: u64,
}

impl DepositActivityThresholds {
    pub fn try_new(
        inactive_threshold_days: u64,
        escheatable_threshold_days: u64,
    ) -> Result<Self, DomainConfigError> {
        if escheatable_threshold_days <= inactive_threshold_days {
            return Err(DomainConfigError::InvalidState(
                "deposit-activity-escheatable-threshold-days must be greater than deposit-activity-inactive-threshold-days".to_string(),
            ));
        }

        Ok(Self {
            inactive_threshold_days,
            escheatable_threshold_days,
        })
    }

    pub fn inactive_threshold_date(
        &self,
        now: DateTime<Utc>,
    ) -> Result<DateTime<Utc>, DomainConfigError> {
        Ok(now
            - Duration::days(days_to_i64(
                "deposit-activity-inactive-threshold-days",
                self.inactive_threshold_days,
            )?))
    }

    pub fn escheatable_threshold_date(
        &self,
        now: DateTime<Utc>,
    ) -> Result<DateTime<Utc>, DomainConfigError> {
        Ok(now
            - Duration::days(days_to_i64(
                "deposit-activity-escheatable-threshold-days",
                self.escheatable_threshold_days,
            )?))
    }
}

fn validate_positive_threshold(key: &str, value: u64) -> Result<(), DomainConfigError> {
    if value == 0 {
        return Err(DomainConfigError::InvalidState(format!(
            "{key} must be greater than 0",
        )));
    }

    Ok(())
}

fn days_to_i64(key: &str, days: u64) -> Result<i64, DomainConfigError> {
    i64::try_from(days).map_err(|_| {
        DomainConfigError::InvalidState(format!("{key} is too large to convert to a duration",))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thresholds_require_escheatable_days_to_exceed_inactive_days() {
        let err = DepositActivityThresholds::try_new(365, 365).unwrap_err();
        assert!(
            err.to_string().contains(
                "deposit-activity-escheatable-threshold-days must be greater than deposit-activity-inactive-threshold-days"
            )
        );
    }
}
