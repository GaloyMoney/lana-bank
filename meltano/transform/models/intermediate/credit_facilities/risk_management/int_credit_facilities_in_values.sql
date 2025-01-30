WITH value_approved_cf AS (
  SELECT
    SAFE_DIVIDE(SUM(facility), 100.0) as amount_in_usd
  FROM {{ref("int_credit_facilities")}}
  WHERE approval_process_concluded_approved
), disbursed AS (
  SELECT
    SAFE_DIVIDE(SUM(amount), 100.0) as amount_in_usd
  FROM {{ref("int_cf_disbursals")}}
  WHERE disbursal_concluded_event_recorded_at_date_key != 19000101
), breakeven AS (
  SELECT
      cfe.event_id
    , 5.53 AS bench_mark              	      -- TODO get from proper source
    , cfe.terms_annual_rate
    , COALESCE(amount, 0) AS disbursal_amount_in_cents
    , facility AS credit_facility_limit_in_cents
  FROM {{ref("int_cf_denormalized")}} cfe
  WHERE approval_process_concluded_approved
  AND facility > 0
), breakeven_by_cf AS (
  SELECT
      event_id
    , bench_mark
    , terms_annual_rate
    , SUM(disbursal_amount_in_cents) AS disbursal_amount_in_cents
    , credit_facility_limit_in_cents
  FROM breakeven
  GROUP BY
      event_id
    , bench_mark
    , terms_annual_rate
    , credit_facility_limit_in_cents
), breakeven_ratio AS (
  SELECT
      event_id
    , bench_mark
    , terms_annual_rate
    , disbursal_amount_in_cents
    , credit_facility_limit_in_cents
    , bench_mark / 100.0 AS bench_mark_interest_rate
    , SAFE_DIVIDE(credit_facility_limit_in_cents, SUM(credit_facility_limit_in_cents) OVER ()) AS facility_limit_ratio
    , SAFE_DIVIDE(disbursal_amount_in_cents, credit_facility_limit_in_cents) AS disbursal_ratio
    , SAFE_DIVIDE(bench_mark, terms_annual_rate) AS breakeven_disbursal_ratio
  FROM breakeven_by_cf
), breakeven_prop AS (
  SELECT
      event_id
    , bench_mark
    , terms_annual_rate
    , disbursal_amount_in_cents
    , credit_facility_limit_in_cents
    , bench_mark_interest_rate
    , facility_limit_ratio
    , disbursal_ratio
    , breakeven_disbursal_ratio
    , SAFE_MULTIPLY(breakeven_disbursal_ratio, facility_limit_ratio) AS prop_breakeven_disbursal_ratio
    , SAFE_MULTIPLY(disbursal_ratio, facility_limit_ratio) AS prop_disbursal_ratio
  FROM breakeven_ratio
), breakeven_sum AS (
  SELECT
      bench_mark
    , SUM(prop_breakeven_disbursal_ratio) AS breakeven_disbursal_ratio
    , SUM(prop_disbursal_ratio) AS disbursal_ratio
  FROM breakeven_prop
  GROUP BY bench_mark
)


SELECT 1 AS order_by, CAST(amount_in_usd AS STRING) AS value, 'Total Value of Approved Credit Facilities' AS name FROM value_approved_cf
  UNION ALL
SELECT 2 AS order_by, CAST(amount_in_usd AS STRING), 'Total Value Disbursed from Approved Credit Facilities' FROM disbursed
  UNION ALL
SELECT 3 AS order_by, CAST(SAFE_SUBTRACT(v.amount_in_usd, d.amount_in_usd) AS STRING), 'Total Value NOT-YET Disbursed from Approved Credit Facilities' FROM value_approved_cf v, disbursed d
  UNION ALL
SELECT 4 AS order_by, CAST(SAFE_DIVIDE(d.amount_in_usd, v.amount_in_usd) * 100 AS STRING), 'Disbursed-to-Approved ratio (%)' FROM value_approved_cf v, disbursed d
  UNION ALL
SELECT 5 AS order_by, CAST(disbursal_ratio * 100 AS STRING), 'Disbursal ratio (%) - proportional' FROM breakeven_sum
  UNION ALL
SELECT 6 AS order_by, CAST(breakeven_disbursal_ratio * 100 AS STRING), 'Breakeven ratio (%) - proportional @' || bench_mark || '% benchmark' FROM breakeven_sum

ORDER BY order_by
