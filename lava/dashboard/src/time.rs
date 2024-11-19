use chrono::{DateTime, Utc};

#[inline(always)]
pub(crate) fn now() -> DateTime<Utc> {
    #[cfg(feature = "sim-time")]
    let res = { sim_time::now() };

    #[cfg(not(feature = "sim-time"))]
    let res = { Utc::now() };

    res
}
