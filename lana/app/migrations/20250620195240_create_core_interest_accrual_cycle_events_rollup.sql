-- Auto-generated rollup table for InterestAccrualCycleEvent
CREATE TABLE core_interest_accrual_cycle_events_rollup (
  id UUID PRIMARY KEY,
  last_sequence INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  account_ids JSONB,
  facility_id UUID,
  facility_matures_at TIMESTAMPTZ,
  idx INTEGER,
  period JSONB,
  terms JSONB,
  accrued_at TIMESTAMPTZ,
  amount JSONB,
  tx_ref VARCHAR,
  effective VARCHAR,
  obligation_id UUID,
  total JSONB,

  -- Collection rollups
  audit_entry_ids BIGINT[],
  ledger_tx_ids UUID[],

  -- Toggle fields
  is_interest_accruals_posted BOOLEAN DEFAULT false

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

  -- Load the current rollup state
  SELECT * INTO current_row
  FROM core_interest_accrual_cycle_events_rollup
  WHERE id = NEW.id;

  -- Early return if event is older than current state
  IF current_row.id IS NOT NULL AND NEW.sequence <= current_row.last_sequence THEN
    RETURN NEW;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'interest_accrued', 'interest_accruals_posted') THEN
    RAISE EXCEPTION 'Unknown event type: %', event_type;
  END IF;

  -- Construct the new row based on event type
  new_row.id := NEW.id;
  new_row.last_sequence := NEW.sequence;
  new_row.created_at := COALESCE(current_row.created_at, NEW.recorded_at);
  new_row.modified_at := NEW.recorded_at;

  -- Initialize fields with default values if this is a new record
  IF current_row.id IS NULL THEN
    new_row.account_ids := (NEW.event -> 'account_ids');
    new_row.facility_id := (NEW.event ->> 'facility_id')::UUID;
    new_row.facility_matures_at := (NEW.event ->> 'facility_matures_at')::TIMESTAMPTZ;
    new_row.idx := (NEW.event ->> 'idx')::INTEGER;
    new_row.period := (NEW.event -> 'period');
    new_row.terms := (NEW.event -> 'terms');
    new_row.accrued_at := (NEW.event ->> 'accrued_at')::TIMESTAMPTZ;
    new_row.amount := (NEW.event -> 'amount');
    new_row.tx_ref := (NEW.event ->> 'tx_ref');
    new_row.effective := (NEW.event ->> 'effective');
    new_row.obligation_id := (NEW.event ->> 'obligation_id')::UUID;
    new_row.total := (NEW.event -> 'total');
    new_row.audit_entry_ids := CASE
       WHEN NEW.event ? 'audit_entry_ids' THEN
         ARRAY(SELECT value::text::BIGINT FROM jsonb_array_elements_text(NEW.event -> 'audit_entry_ids'))
       ELSE ARRAY[]::BIGINT[]
     END
;
    new_row.ledger_tx_ids := CASE
       WHEN NEW.event ? 'ledger_tx_ids' THEN
         ARRAY(SELECT value::text::UUID FROM jsonb_array_elements_text(NEW.event -> 'ledger_tx_ids'))
       ELSE ARRAY[]::UUID[]
     END
;
    new_row.is_interest_accruals_posted := false;
  ELSE
    -- Default all fields to current values
    new_row.account_ids := current_row.account_ids;
    new_row.facility_id := current_row.facility_id;
    new_row.facility_matures_at := current_row.facility_matures_at;
    new_row.idx := current_row.idx;
    new_row.period := current_row.period;
    new_row.terms := current_row.terms;
    new_row.accrued_at := current_row.accrued_at;
    new_row.amount := current_row.amount;
    new_row.tx_ref := current_row.tx_ref;
    new_row.effective := current_row.effective;
    new_row.obligation_id := current_row.obligation_id;
    new_row.total := current_row.total;
    new_row.audit_entry_ids := current_row.audit_entry_ids;
    new_row.ledger_tx_ids := current_row.ledger_tx_ids;
    new_row.is_interest_accruals_posted := current_row.is_interest_accruals_posted;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.account_ids := (NEW.event -> 'account_ids');
      new_row.facility_id := (NEW.event ->> 'facility_id')::UUID;
      new_row.facility_matures_at := (NEW.event ->> 'facility_matures_at')::TIMESTAMPTZ;
      new_row.idx := (NEW.event ->> 'idx')::INTEGER;
      new_row.period := (NEW.event -> 'period');
      new_row.terms := (NEW.event -> 'terms');
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
    WHEN 'interest_accrued' THEN
      new_row.accrued_at := (NEW.event ->> 'accrued_at')::TIMESTAMPTZ;
      new_row.amount := (NEW.event -> 'amount');
      new_row.tx_ref := (NEW.event ->> 'tx_ref');
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.ledger_tx_ids := array_append(COALESCE(current_row.ledger_tx_ids, ARRAY[]::UUID[]), (NEW.event ->> 'ledger_tx_id')::UUID);
    WHEN 'interest_accruals_posted' THEN
      new_row.tx_ref := (NEW.event ->> 'tx_ref');
      new_row.effective := (NEW.event ->> 'effective');
      new_row.obligation_id := (NEW.event ->> 'obligation_id')::UUID;
      new_row.total := (NEW.event -> 'total');
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.ledger_tx_ids := array_append(COALESCE(current_row.ledger_tx_ids, ARRAY[]::UUID[]), (NEW.event ->> 'ledger_tx_id')::UUID);
      new_row.is_interest_accruals_posted := true;
  END CASE;

  INSERT INTO core_interest_accrual_cycle_events_rollup (
    id,
    last_sequence,
    created_at,
    modified_at,
    account_ids,
    facility_id,
    facility_matures_at,
    idx,
    period,
    terms,
    accrued_at,
    amount,
    tx_ref,
    effective,
    obligation_id,
    total,
    audit_entry_ids,
    ledger_tx_ids,
    is_interest_accruals_posted
  )
  VALUES (
    new_row.id,
    new_row.last_sequence,
    new_row.created_at,
    new_row.modified_at,
    new_row.account_ids,
    new_row.facility_id,
    new_row.facility_matures_at,
    new_row.idx,
    new_row.period,
    new_row.terms,
    new_row.accrued_at,
    new_row.amount,
    new_row.tx_ref,
    new_row.effective,
    new_row.obligation_id,
    new_row.total,
    new_row.audit_entry_ids,
    new_row.ledger_tx_ids,
    new_row.is_interest_accruals_posted
  )
  ON CONFLICT (id) DO UPDATE SET
    last_sequence = EXCLUDED.last_sequence,
    modified_at = EXCLUDED.modified_at,
    account_ids = EXCLUDED.account_ids,
    facility_id = EXCLUDED.facility_id,
    facility_matures_at = EXCLUDED.facility_matures_at,
    idx = EXCLUDED.idx,
    period = EXCLUDED.period,
    terms = EXCLUDED.terms,
    accrued_at = EXCLUDED.accrued_at,
    amount = EXCLUDED.amount,
    tx_ref = EXCLUDED.tx_ref,
    effective = EXCLUDED.effective,
    obligation_id = EXCLUDED.obligation_id,
    total = EXCLUDED.total,
    audit_entry_ids = EXCLUDED.audit_entry_ids,
    ledger_tx_ids = EXCLUDED.ledger_tx_ids,
    is_interest_accruals_posted = EXCLUDED.is_interest_accruals_posted;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generated trigger for InterestAccrualCycleEvent
CREATE TRIGGER core_interest_accrual_cycle_events_rollup_trigger
  AFTER INSERT ON core_interest_accrual_cycle_events
  FOR EACH ROW
  EXECUTE FUNCTION core_interest_accrual_cycle_events_rollup_trigger();
