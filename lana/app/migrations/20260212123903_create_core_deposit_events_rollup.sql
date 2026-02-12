-- Auto-generated rollup table for DepositEvent
CREATE TABLE core_deposit_events_rollup (
  id UUID NOT NULL,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  event_type TEXT NOT NULL,
  -- Flattened fields from the event JSON
  amount BIGINT,
  deposit_account_id UUID,
  public_id VARCHAR,
  reference VARCHAR,
  status VARCHAR,

  -- Collection rollups
  ledger_tx_ids UUID[]
,
  PRIMARY KEY (id, version)
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

  -- Load the previous version if this isn't the first event
  IF NEW.sequence > 1 THEN
    SELECT * INTO current_row
    FROM core_deposit_events_rollup
    WHERE id = NEW.id AND version = NEW.sequence - 1;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'reverted') THEN
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
    new_row.amount := (NEW.event ->> 'amount')::BIGINT;
    new_row.deposit_account_id := (NEW.event ->> 'deposit_account_id')::UUID;
    new_row.ledger_tx_ids := CASE
       WHEN NEW.event ? 'ledger_tx_ids' THEN
         ARRAY(SELECT value::text::UUID FROM jsonb_array_elements_text(NEW.event -> 'ledger_tx_ids'))
       ELSE ARRAY[]::UUID[]
     END
;
    new_row.public_id := (NEW.event ->> 'public_id');
    new_row.reference := (NEW.event ->> 'reference');
    new_row.status := (NEW.event ->> 'status');
  ELSE
    -- Default all fields to current values
    new_row.amount := current_row.amount;
    new_row.deposit_account_id := current_row.deposit_account_id;
    new_row.ledger_tx_ids := current_row.ledger_tx_ids;
    new_row.public_id := current_row.public_id;
    new_row.reference := current_row.reference;
    new_row.status := current_row.status;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.amount := (NEW.event ->> 'amount')::BIGINT;
      new_row.deposit_account_id := (NEW.event ->> 'deposit_account_id')::UUID;
      new_row.ledger_tx_ids := array_append(COALESCE(current_row.ledger_tx_ids, ARRAY[]::UUID[]), (NEW.event ->> 'ledger_tx_id')::UUID);
      new_row.public_id := (NEW.event ->> 'public_id');
      new_row.reference := (NEW.event ->> 'reference');
      new_row.status := (NEW.event ->> 'status');
    WHEN 'reverted' THEN
      new_row.ledger_tx_ids := array_append(COALESCE(current_row.ledger_tx_ids, ARRAY[]::UUID[]), (NEW.event ->> 'ledger_tx_id')::UUID);
      new_row.status := (NEW.event ->> 'status');
  END CASE;

  INSERT INTO core_deposit_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    event_type,
    amount,
    deposit_account_id,
    ledger_tx_ids,
    public_id,
    reference,
    status
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.event_type,
    new_row.amount,
    new_row.deposit_account_id,
    new_row.ledger_tx_ids,
    new_row.public_id,
    new_row.reference,
    new_row.status
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;


-- Auto-generated trigger for DepositEvent
CREATE TRIGGER core_deposit_events_rollup_trigger
  AFTER INSERT ON core_deposit_events
  FOR EACH ROW
  EXECUTE FUNCTION core_deposit_events_rollup_trigger();
