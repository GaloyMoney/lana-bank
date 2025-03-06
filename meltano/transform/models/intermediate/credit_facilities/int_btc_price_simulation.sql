{{ config(materialized='table') }}

with btc_volatility_term_structure as (
    -- todo: calculate the following based on an up-to-date btc/usd price time series
    select
        0.0341 as vol_1_day,
        0.0558 as vol_3_day,
        0.1008 as vol_1_week,
        0.1406 as vol_2_week,
        0.212 as vol_3_week,
        0.2155 as vol_1_month,
        0.513 as vol_3_month,   -- less than 30 samples, careful with use
        0.7649 as vol_6_month,  -- less than 30 samples, careful with use
        1.6023 as vol_1_year    -- less than 30 samples, careful with use
),

sigma_to_event_frequency as (
    select
        1 as sigma_level,
        100.00 as normal_frequency_per_year,
        90.0 as laplace_frequency_per_year
    union all
    select
        2 as sigma_level,
        20.000 as normal_frequency_per_year,
        20.0 as laplace_frequency_per_year
    union all
    select
        3 as sigma_level,
        1.0000 as normal_frequency_per_year,
        5.00 as laplace_frequency_per_year
    union all
    select
        4 as sigma_level,
        0.0200 as normal_frequency_per_year,
        1.00 as laplace_frequency_per_year
    union all
    select
        5 as sigma_level,
        0.0002 as normal_frequency_per_year,
        0.30 as laplace_frequency_per_year
    union all
    select
        6 as sigma_level,
        7e-007 as normal_frequency_per_year,
        0.08 as laplace_frequency_per_year
    union all
    select
        7 as sigma_level,
        9e-010 as normal_frequency_per_year,
        0.02 as laplace_frequency_per_year
),

btc_price_loss_simulation as (
    select
        freq.*,
        (vol.vol_1_day * -freq.sigma_level) as period_1_day_loss_percent,
        (vol.vol_3_day * -freq.sigma_level) as period_3_day_loss_percent,
        (vol.vol_1_week * -freq.sigma_level) as period_1_week_loss_percent,
        (vol.vol_2_week * -freq.sigma_level) as period_2_week_loss_percent,
        (vol.vol_3_week * -freq.sigma_level) as period_3_week_loss_percent,
        (vol.vol_1_month * -freq.sigma_level) as period_1_month_loss_percent,
        (vol.vol_3_month * -freq.sigma_level) as period_3_month_loss_percent,
        (vol.vol_6_month * -freq.sigma_level) as period_6_month_loss_percent,
        (vol.vol_1_year * -freq.sigma_level) as period_1_year_loss_percent
    from sigma_to_event_frequency as freq, btc_volatility_term_structure as vol
)

select *
from btc_price_loss_simulation
