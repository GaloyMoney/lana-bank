-- Compare legacy SQL model with new seed implementation
-- This test should return 0 rows if both are equivalent

(
    select estado, `explicación`, status, explanation, consumer_calendar_ge_days, consumer_calendar_le_days
    from {{ ref('static_nrp_41_estados_del_prestamo_legacy') }}
    except distinct
    select estado, `explicación`, status, explanation, consumer_calendar_ge_days, consumer_calendar_le_days
    from {{ ref('static_nrp_41_estados_del_prestamo') }}
)
union all
(
    select estado, `explicación`, status, explanation, consumer_calendar_ge_days, consumer_calendar_le_days
    from {{ ref('static_nrp_41_estados_del_prestamo') }}
    except distinct
    select estado, `explicación`, status, explanation, consumer_calendar_ge_days, consumer_calendar_le_days
    from {{ ref('static_nrp_41_estados_del_prestamo_legacy') }}
)
