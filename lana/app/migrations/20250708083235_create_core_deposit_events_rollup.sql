-- Auto-generated rollup table for DepositEvent
CREATE TABLE core_deposit_events_rollup (
  id UUID PRIMARY KEY,
  last_sequence INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  amount BIGINT,
  deposit_account_id UUID,
  ledger_tx_id UUID,
  reference VARCHAR,

  -- Collection rollups
  audit_entry_ids BIGINT[]

);

-- Auto-generated trigger function for DepositEvent
CREATE OR REPLACE FUNCTION core_deposit_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_deposit_events_rollup%ROWTYPE;
  new_row core_deposit_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the current rollup state
  SELECT * INTO current_row
  FROM core_deposit_events_rollup
  WHERE id = NEW.id;

  -- Early return if event is older than current state
  IF current_row.id IS NOT NULL AND NEW.sequence <= current_row.last_sequence THEN
    RETURN NEW;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized') THEN
    RAISE EXCEPTION 'Unknown event type: %', event_type;
  END IF;

  -- Construct the new row based on event type
  new_row.id := NEW.id;
  new_row.last_sequence := NEW.sequence;
  new_row.created_at := COALESCE(current_row.created_at, NEW.recorded_at);
  new_row.modified_at := NEW.recorded_at;

  -- Initialize fields with default values if this is a new record
  IF current_row.id IS NULL THEN
    new_row.amount := (NEW.event ->> 'amount')::BIGINT;
    new_row.audit_entry_ids := CASE
       WHEN NEW.event ? 'audit_entry_ids' THEN
         ARRAY(SELECT value::text::BIGINT FROM jsonb_array_elements_text(NEW.event -> 'audit_entry_ids'))
       ELSE ARRAY[]::BIGINT[]
     END
;
    new_row.deposit_account_id := (NEW.event ->> 'deposit_account_id')::UUID;
    new_row.ledger_tx_id := (NEW.event ->> 'ledger_tx_id')::UUID;
    new_row.reference := (NEW.event ->> 'reference');
  ELSE
    -- Default all fields to current values
    new_row.amount := current_row.amount;
    new_row.audit_entry_ids := current_row.audit_entry_ids;
    new_row.deposit_account_id := current_row.deposit_account_id;
    new_row.ledger_tx_id := current_row.ledger_tx_id;
    new_row.reference := current_row.reference;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.amount := (NEW.event ->> 'amount')::BIGINT;
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.deposit_account_id := (NEW.event ->> 'deposit_account_id')::UUID;
      new_row.ledger_tx_id := (NEW.event ->> 'ledger_tx_id')::UUID;
      new_row.reference := (NEW.event ->> 'reference');
  END CASE;

  INSERT INTO core_deposit_events_rollup (
    id,
    last_sequence,
    created_at,
    modified_at,
    amount,
    audit_entry_ids,
    deposit_account_id,
    ledger_tx_id,
    reference
  )
  VALUES (
    new_row.id,
    new_row.last_sequence,
    new_row.created_at,
    new_row.modified_at,
    new_row.amount,
    new_row.audit_entry_ids,
    new_row.deposit_account_id,
    new_row.ledger_tx_id,
    new_row.reference
  )
  ON CONFLICT (id) DO UPDATE SET
    last_sequence = EXCLUDED.last_sequence,
    modified_at = EXCLUDED.modified_at,
    amount = EXCLUDED.amount,
    audit_entry_ids = EXCLUDED.audit_entry_ids,
    deposit_account_id = EXCLUDED.deposit_account_id,
    ledger_tx_id = EXCLUDED.ledger_tx_id,
    reference = EXCLUDED.reference;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generated trigger for DepositEvent
CREATE TRIGGER core_deposit_events_rollup_trigger
  AFTER INSERT ON core_deposit_events
  FOR EACH ROW
  EXECUTE FUNCTION core_deposit_events_rollup_trigger();
