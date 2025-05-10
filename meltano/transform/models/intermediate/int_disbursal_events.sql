with initialized as (
    select
        id as disbursal_id,
        recorded_at as initialized_recorded_at,
        json_value(event, '$.facility_id') as facility_id,
        cast(json_value(event, '$.amount') as numeric) as initialized_amount,
        cast(json_value(event, '$.audit_info.audit_entry_id') as integer) as audit_entry_id,

    from {{ ref('stg_disbursal_events') }}
    where event_type = 'initialized'
)

, concluded as (
    select
        id as disbursal_id,
        recorded_at as concluded_recorded_at,
        cast(json_value(event, '$.approved') as boolean) as approved,
        cast(json_value(event, '$.audit_info.audit_entry_id') as integer) as audit_entry_id,

    from {{ ref('stg_disbursal_events') }}
    where event_type = "approval_process_concluded"
)

, settled as (
    select
        id as disbursal_id,
        recorded_at as event_recorded_at,
        cast(json_value(event, '$.recorded_at') as timestamp) as settled_recorded_at,
        cast(json_value(event, '$.amount') as numeric) as settled_amount,
        cast(json_value(event, '$.audit_info.audit_entry_id') as integer) as audit_entry_id,

    from {{ ref('stg_disbursal_events') }}
    where event_type = 'settled'
)

, final as (
    select *
    from initialized
    left join concluded using (disbursal_id, audit_entry_id)
    left join settled using (disbursal_id, audit_entry_id)
)


select * from final
