with

accounts as (
    select *
    from
        {{ ref('stg_accounts') }}
)
,

account_sets as (
    select *
    from
        {{ ref('stg_account_sets') }}
)
,

account_set_member_accounts as (
    select *
    from
        {{ ref('stg_account_set_member_accounts') }}
)
,

account_set_member_account_sets as (
    select *
    from
        {{ ref('stg_account_set_member_account_sets') }}
)
,

account_balances as (
    select *
    from
        {{ ref('stg_account_balances') }}
)
,

bitfinex_order_book as (
    select *
    from
        {{ ref('stg_bitfinex_order_book') }}
)
,

bitfinex_ticker_price as (
    select *
    from
        {{ ref('stg_bitfinex_ticker_price') }}
)
,

bitfinex_trades as (
    select *
    from
        {{ ref('stg_bitfinex_trades') }}
)
,

credit_facility_events as (
    select *
    from
        {{ ref('stg_credit_facility_events') }}
)
,

customer_events as (
    select *
    from
        {{ ref('stg_customer_events') }}
)
,

sumsub_applicants as (
    select *
    from
        {{ ref('stg_sumsub_applicants') }}
)


select
    'TODO' as `id_codigo_extracontable`,
    'TODO' as `desc_extra_contable`,
    7060.0 as `Valor`
-- FROM
--     accounts,
--     account_sets,
--     account_set_member_accounts,
--     account_set_member_account_sets,
--     account_balances,
--     bitfinex_order_book,
--     bitfinex_ticker_price,
--     bitfinex_trades,
--     credit_facility_events,
--     customer_events,
--     sumsub_applicants,
