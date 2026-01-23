use domain_config::define_exposed_config;

define_exposed_config! {
    pub struct Timezone(chrono_tz::Tz);
    spec {
        key: "timezone";
        default: || Some(chrono_tz::UTC);
    }
}

define_exposed_config! {
    pub struct ClosingTime(chrono::NaiveTime);
    spec {
        key: "closing-time";
        default: || Some(chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    }
}
