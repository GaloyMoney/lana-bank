use chrono::{DateTime, Utc};
// use std::time::Duration;

#[inline(always)]
pub(crate) fn now() -> DateTime<Utc> {
    #[cfg(feature = "sim-time")]
    let res = { sim_time::now() };

    #[cfg(not(feature = "sim-time"))]
    let res = { Utc::now() };

    res
}

// pub(crate) async fn sleep(duration: Duration) {
//     #[cfg(feature = "sim-time")]
//     sim_time::sleep(duration).await;
//     #[cfg(not(feature = "sim-time"))]
//     tokio::time::sleep(duration).await;
// }
