use chrono::{DateTime, Duration, NaiveTime, Timelike, Utc};
use keycloak_client::KeycloakConnectionConfig;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomerSyncConfig {
    #[serde(default = "default_customer_status_sync_active")]
    pub customer_status_sync_active: bool,
    #[serde(default = "default_create_deposit_account_on_customer_create")]
    pub create_deposit_account_on_customer_create: bool,
    #[serde(default = "default_keycloak")]
    pub keycloak: KeycloakConnectionConfig,
    #[serde(default = "default_activity_update_job_run_time")]
    pub activity_update_utc_time: ActivityUpdateJobRunTime,
    #[serde(default = "default_activity_update_enabled")]
    pub activity_update_enabled: bool,
}

impl Default for CustomerSyncConfig {
    fn default() -> Self {
        Self {
            customer_status_sync_active: default_customer_status_sync_active(),
            create_deposit_account_on_customer_create:
                default_create_deposit_account_on_customer_create(),
            keycloak: default_keycloak(),
            activity_update_utc_time: default_activity_update_job_run_time(),
            activity_update_enabled: default_activity_update_enabled(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActivityUpdateJobRunTime {
    hours_past_midnight: u32,
    minutes_past_hour: u32,
}

impl ActivityUpdateJobRunTime {
    pub fn next_after(&self, after: DateTime<Utc>) -> DateTime<Utc> {
        let tomorrow = after + Duration::days(1);

        let midnight = tomorrow
            .date_naive()
            .and_hms_opt(self.hours_past_midnight, self.minutes_past_hour, 0)
            .expect("Cannot update time");

        midnight
            .and_local_timezone(Utc)
            .single()
            .expect("Cannot update time")
    }
}

impl<'de> Deserialize<'de> for ActivityUpdateJobRunTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let time = NaiveTime::parse_from_str(&s, "%H:%M")
            .map_err(|e| serde::de::Error::custom(format!("Invalid time format '{}': {}", s, e)))?;

        Ok(ActivityUpdateJobRunTime {
            hours_past_midnight: time.hour(),
            minutes_past_hour: time.minute(),
        })
    }
}

impl Serialize for ActivityUpdateJobRunTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let time_str = format!(
            "{:02}:{:02}",
            self.hours_past_midnight, self.minutes_past_hour
        );
        serializer.serialize_str(&time_str)
    }
}

fn default_activity_update_job_run_time() -> ActivityUpdateJobRunTime {
    ActivityUpdateJobRunTime {
        hours_past_midnight: 0,
        minutes_past_hour: 0,
    }
}

fn default_activity_update_enabled() -> bool {
    true
}

fn default_keycloak() -> KeycloakConnectionConfig {
    KeycloakConnectionConfig {
        url: "http://localhost:8081".to_string(),
        client_id: "customer-service-account".to_string(),
        client_secret: "secret".to_string(),
        realm: "customer".to_string(),
    }
}

fn default_customer_status_sync_active() -> bool {
    true
}

fn default_create_deposit_account_on_customer_create() -> bool {
    false
}
