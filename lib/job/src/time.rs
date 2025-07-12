use chrono::{DateTime, Utc};
use std::time::Duration;

#[inline(always)]
pub(crate) fn now() -> DateTime<Utc> {
    #[cfg(feature = "sim-time")]
    let res = { sim_time::now() };

    #[cfg(not(feature = "sim-time"))]
    let res = { Utc::now() };

    res
}

pub(crate) fn sleep(duration: Duration) -> tokio::time::Sleep {
    #[cfg(feature = "sim-time")]
    let res = sim_time::sleep(duration);
    #[cfg(not(feature = "sim-time"))]
    let res = tokio::time::sleep(duration);
    res
}

pub(crate) fn timeout<F>(duration: Duration, future: F) -> tokio::time::Timeout<F::IntoFuture>
where
    F: core::future::IntoFuture,
{
    #[cfg(feature = "sim-time")]
    let res = sim_time::timeout(duration, future);
    #[cfg(not(feature = "sim-time"))]
    let res = tokio::time::timeout(duration, future);
    res
}
