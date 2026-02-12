-- Auto-generated rollup table for InterestAccrualCycleEvent
CREATE TABLE core_interest_accrual_cycle_events_rollup (
  id UUID NOT NULL,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  event_type TEXT NOT NULL,
  -- Flattened fields from the event JSON
  account_ids JSONB,
  accrued_at TIMESTAMPTZ,
  amount BIGINT,
  effective VARCHAR,
  facility_id UUID,
  facility_maturity_date TIMESTAMPTZ,
  idx INTEGER,
  obligation_id UUID,
  period JSONB,
  terms JSONB,
  total BIGINT,
  tx_ref VARCHAR,

  -- Collection rollups
  ledger_tx_ids UUID[],

  -- Toggle fields
  is_interest_accruals_posted BOOLEAN DEFAULT false
,
  PRIMARY KEY (id, version)
);


-- Auto-generated trigger function for InterestAccrualCycleEvent
CREATE OR REPLACE FUNCTION core_interest_accrual_cycle_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_interest_accrual_cycle_events_rollup%ROWTYPE;
  new_row core_interest_accrual_cycle_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the previous version if this isn't the first event
  IF NEW.sequence > 1 THEN
    SELECT * INTO current_row
    FROM core_interest_accrual_cycle_events_rollup
    WHERE id = NEW.id AND version = NEW.sequence - 1;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'interest_accrued', 'interest_accruals_posted') THEN
    RAISE EXCEPTION 'Unknown event type: %', event_type;
  END IF;

  -- Construct the new row based on event type
  new_row.id := NEW.id;
  new_row.version := NEW.sequence;
  new_row.created_at := COALESCE(current_row.created_at, NEW.recorded_at);
  new_row.modified_at := NEW.recorded_at;
  new_row.event_type := NEW.recorded_at;

  -- Initialize fields with default values if this is a new record
  IF current_row.id IS NULL THEN
    new_row.account_ids := (NEW.event -> 'account_ids');
    new_row.accrued_at := (NEW.event ->> 'accrued_at')::TIMESTAMPTZ;
    new_row.amount := (NEW.event ->> 'amount')::BIGINT;
    new_row.effective := (NEW.event ->> 'effective');
    new_row.facility_id := (NEW.event ->> 'facility_id')::UUID;
    new_row.facility_maturity_date := (NEW.event ->> 'facility_maturity_date')::TIMESTAMPTZ;
    new_row.idx := (NEW.event ->> 'idx')::INTEGER;
    new_row.is_interest_accruals_posted := false;
    new_row.ledger_tx_ids := CASE
       WHEN NEW.event ? 'ledger_tx_ids' THEN
         ARRAY(SELECT value::text::UUID FROM jsonb_array_elements_text(NEW.event -> 'ledger_tx_ids'))
       ELSE ARRAY[]::UUID[]
     END
;
    new_row.obligation_id := (NEW.event ->> 'obligation_id')::UUID;
    new_row.period := (NEW.event -> 'period');
    new_row.terms := (NEW.event -> 'terms');
    new_row.total := (NEW.event ->> 'total')::BIGINT;
    new_row.tx_ref := (NEW.event ->> 'tx_ref');
  ELSE
    -- Default all fields to current values
    new_row.account_ids := current_row.account_ids;
    new_row.accrued_at := current_row.accrued_at;
    new_row.amount := current_row.amount;
    new_row.effective := current_row.effective;
    new_row.facility_id := current_row.facility_id;
    new_row.facility_maturity_date := current_row.facility_maturity_date;
    new_row.idx := current_row.idx;
    new_row.is_interest_accruals_posted := current_row.is_interest_accruals_posted;
    new_row.ledger_tx_ids := current_row.ledger_tx_ids;
    new_row.obligation_id := current_row.obligation_id;
    new_row.period := current_row.period;
    new_row.terms := current_row.terms;
    new_row.total := current_row.total;
    new_row.tx_ref := current_row.tx_ref;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.account_ids := (NEW.event -> 'account_ids');
      new_row.facility_id := (NEW.event ->> 'facility_id')::UUID;
      new_row.facility_maturity_date := (NEW.event ->> 'facility_maturity_date')::TIMESTAMPTZ;
      new_row.idx := (NEW.event ->> 'idx')::INTEGER;
      new_row.period := (NEW.event -> 'period');
      new_row.terms := (NEW.event -> 'terms');
    WHEN 'interest_accrued' THEN
      new_row.accrued_at := (NEW.event ->> 'accrued_at')::TIMESTAMPTZ;
      new_row.amount := (NEW.event ->> 'amount')::BIGINT;
      new_row.ledger_tx_ids := array_append(COALESCE(current_row.ledger_tx_ids, ARRAY[]::UUID[]), (NEW.event ->> 'ledger_tx_id')::UUID);
      new_row.tx_ref := (NEW.event ->> 'tx_ref');
    WHEN 'interest_accruals_posted' THEN
      new_row.effective := (NEW.event ->> 'effective');
      new_row.is_interest_accruals_posted := true;
      new_row.ledger_tx_ids := array_append(COALESCE(current_row.ledger_tx_ids, ARRAY[]::UUID[]), (NEW.event ->> 'ledger_tx_id')::UUID);
      new_row.obligation_id := (NEW.event ->> 'obligation_id')::UUID;
      new_row.total := (NEW.event ->> 'total')::BIGINT;
      new_row.tx_ref := (NEW.event ->> 'tx_ref');
  END CASE;

  INSERT INTO core_interest_accrual_cycle_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    event_type,
    account_ids,
    accrued_at,
    amount,
    effective,
    facility_id,
    facility_maturity_date,
    idx,
    is_interest_accruals_posted,
    ledger_tx_ids,
    obligation_id,
    period,
    terms,
    total,
    tx_ref
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.event_type,
    new_row.account_ids,
    new_row.accrued_at,
    new_row.amount,
    new_row.effective,
    new_row.facility_id,
    new_row.facility_maturity_date,
    new_row.idx,
    new_row.is_interest_accruals_posted,
    new_row.ledger_tx_ids,
    new_row.obligation_id,
    new_row.period,
    new_row.terms,
    new_row.total,
    new_row.tx_ref
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;


-- Auto-generated trigger for InterestAccrualCycleEvent
CREATE TRIGGER core_interest_accrual_cycle_events_rollup_trigger
  AFTER INSERT ON core_interest_accrual_cycle_events
  FOR EACH ROW
  EXECUTE FUNCTION core_interest_accrual_cycle_events_rollup_trigger();
