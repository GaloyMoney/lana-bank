-- Auto-generated rollup table for PaymentAllocationEvent
CREATE TABLE core_payment_allocation_events_rollup (
  id UUID NOT NULL,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  amount BIGINT,
  credit_facility_id UUID,
  effective VARCHAR,
  ledger_tx_id UUID,
  obligation_id UUID,
  obligation_type VARCHAR,
  payment_allocation_idx INTEGER,
  payment_holding_account_id UUID,
  payment_id UUID,
  receivable_account_id UUID
,
  PRIMARY KEY (id, version)
);

-- Auto-generated trigger function for PaymentAllocationEvent
CREATE OR REPLACE FUNCTION core_payment_allocation_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_payment_allocation_events_rollup%ROWTYPE;
  new_row core_payment_allocation_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the previous version if this isn't the first event
  IF NEW.sequence > 1 THEN
    SELECT * INTO current_row
    FROM core_payment_allocation_events_rollup
    WHERE id = NEW.id AND version = NEW.sequence - 1;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized') THEN
    RAISE EXCEPTION 'Unknown event type: %', event_type;
  END IF;

  -- Construct the new row based on event type
  new_row.id := NEW.id;
  new_row.version := NEW.sequence;
  new_row.created_at := COALESCE(current_row.created_at, NEW.recorded_at);
  new_row.modified_at := NEW.recorded_at;

  -- Initialize fields with default values if this is a new record
  IF current_row.id IS NULL THEN
    new_row.amount := (NEW.event ->> 'amount')::BIGINT;
    new_row.credit_facility_id := (NEW.event ->> 'credit_facility_id')::UUID;
    new_row.effective := (NEW.event ->> 'effective');
    new_row.ledger_tx_id := (NEW.event ->> 'ledger_tx_id')::UUID;
    new_row.obligation_id := (NEW.event ->> 'obligation_id')::UUID;
    new_row.obligation_type := (NEW.event ->> 'obligation_type');
    new_row.payment_allocation_idx := (NEW.event ->> 'payment_allocation_idx')::INTEGER;
    new_row.payment_holding_account_id := (NEW.event ->> 'payment_holding_account_id')::UUID;
    new_row.payment_id := (NEW.event ->> 'payment_id')::UUID;
    new_row.receivable_account_id := (NEW.event ->> 'receivable_account_id')::UUID;
  ELSE
    -- Default all fields to current values
    new_row.amount := current_row.amount;
    new_row.credit_facility_id := current_row.credit_facility_id;
    new_row.effective := current_row.effective;
    new_row.ledger_tx_id := current_row.ledger_tx_id;
    new_row.obligation_id := current_row.obligation_id;
    new_row.obligation_type := current_row.obligation_type;
    new_row.payment_allocation_idx := current_row.payment_allocation_idx;
    new_row.payment_holding_account_id := current_row.payment_holding_account_id;
    new_row.payment_id := current_row.payment_id;
    new_row.receivable_account_id := current_row.receivable_account_id;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.amount := (NEW.event ->> 'amount')::BIGINT;
      new_row.credit_facility_id := (NEW.event ->> 'credit_facility_id')::UUID;
      new_row.effective := (NEW.event ->> 'effective');
      new_row.ledger_tx_id := (NEW.event ->> 'ledger_tx_id')::UUID;
      new_row.obligation_id := (NEW.event ->> 'obligation_id')::UUID;
      new_row.obligation_type := (NEW.event ->> 'obligation_type');
      new_row.payment_allocation_idx := (NEW.event ->> 'payment_allocation_idx')::INTEGER;
      new_row.payment_holding_account_id := (NEW.event ->> 'payment_holding_account_id')::UUID;
      new_row.payment_id := (NEW.event ->> 'payment_id')::UUID;
      new_row.receivable_account_id := (NEW.event ->> 'receivable_account_id')::UUID;
  END CASE;

  INSERT INTO core_payment_allocation_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    amount,
    credit_facility_id,
    effective,
    ledger_tx_id,
    obligation_id,
    obligation_type,
    payment_allocation_idx,
    payment_holding_account_id,
    payment_id,
    receivable_account_id
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.amount,
    new_row.credit_facility_id,
    new_row.effective,
    new_row.ledger_tx_id,
    new_row.obligation_id,
    new_row.obligation_type,
    new_row.payment_allocation_idx,
    new_row.payment_holding_account_id,
    new_row.payment_id,
    new_row.receivable_account_id
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generated trigger for PaymentAllocationEvent
CREATE TRIGGER core_payment_allocation_events_rollup_trigger
  AFTER INSERT ON core_payment_allocation_events
  FOR EACH ROW
  EXECUTE FUNCTION core_payment_allocation_events_rollup_trigger();
