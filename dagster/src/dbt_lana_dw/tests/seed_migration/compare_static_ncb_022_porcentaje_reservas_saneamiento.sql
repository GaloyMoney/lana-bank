-- Compare legacy SQL model with new seed implementation
-- This test should return 0 rows if both are equivalent

(
    select category, reserve_percentage, consumer_calendar_ge_days, consumer_calendar_le_days
    from {{ ref('static_ncb_022_porcentaje_reservas_saneamiento_legacy') }}
    except distinct
    select category, reserve_percentage, consumer_calendar_ge_days, consumer_calendar_le_days
    from {{ ref('static_ncb_022_porcentaje_reservas_saneamiento') }}
)
union all
(
    select category, reserve_percentage, consumer_calendar_ge_days, consumer_calendar_le_days
    from {{ ref('static_ncb_022_porcentaje_reservas_saneamiento') }}
    except distinct
    select category, reserve_percentage, consumer_calendar_ge_days, consumer_calendar_le_days
    from {{ ref('static_ncb_022_porcentaje_reservas_saneamiento_legacy') }}
)
