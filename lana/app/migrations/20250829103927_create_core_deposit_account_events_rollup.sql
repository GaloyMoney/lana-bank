-- Auto-generated rollup table for DepositAccountEvent
CREATE TABLE core_deposit_account_events_rollup (
  id UUID NOT NULL,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  account_holder_id UUID,
  frozen_deposit_account_id UUID,
  ledger_account_id UUID,
  public_id VARCHAR,
  status VARCHAR
,
  PRIMARY KEY (id, version)
);

-- Auto-generated trigger function for DepositAccountEvent
CREATE OR REPLACE FUNCTION core_deposit_account_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_deposit_account_events_rollup%ROWTYPE;
  new_row core_deposit_account_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the previous version if this isn't the first event
  IF NEW.sequence > 1 THEN
    SELECT * INTO current_row
    FROM core_deposit_account_events_rollup
    WHERE id = NEW.id AND version = NEW.sequence - 1;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'account_status_updated') THEN
    RAISE EXCEPTION 'Unknown event type: %', event_type;
  END IF;

  -- Construct the new row based on event type
  new_row.id := NEW.id;
  new_row.version := NEW.sequence;
  new_row.created_at := COALESCE(current_row.created_at, NEW.recorded_at);
  new_row.modified_at := NEW.recorded_at;

  -- Initialize fields with default values if this is a new record
  IF current_row.id IS NULL THEN
    new_row.account_holder_id := (NEW.event ->> 'account_holder_id')::UUID;
    new_row.frozen_deposit_account_id := (NEW.event ->> 'frozen_deposit_account_id')::UUID;
    new_row.ledger_account_id := (NEW.event ->> 'ledger_account_id')::UUID;
    new_row.public_id := (NEW.event ->> 'public_id');
    new_row.status := (NEW.event ->> 'status');
  ELSE
    -- Default all fields to current values
    new_row.account_holder_id := current_row.account_holder_id;
    new_row.frozen_deposit_account_id := current_row.frozen_deposit_account_id;
    new_row.ledger_account_id := current_row.ledger_account_id;
    new_row.public_id := current_row.public_id;
    new_row.status := current_row.status;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.account_holder_id := (NEW.event ->> 'account_holder_id')::UUID;
      new_row.frozen_deposit_account_id := (NEW.event ->> 'frozen_deposit_account_id')::UUID;
      new_row.ledger_account_id := (NEW.event ->> 'ledger_account_id')::UUID;
      new_row.public_id := (NEW.event ->> 'public_id');
      new_row.status := (NEW.event ->> 'status');
    WHEN 'account_status_updated' THEN
      new_row.status := (NEW.event ->> 'status');
  END CASE;

  INSERT INTO core_deposit_account_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    account_holder_id,
    frozen_deposit_account_id,
    ledger_account_id,
    public_id,
    status
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.account_holder_id,
    new_row.frozen_deposit_account_id,
    new_row.ledger_account_id,
    new_row.public_id,
    new_row.status
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generated trigger for DepositAccountEvent
CREATE TRIGGER core_deposit_account_events_rollup_trigger
  AFTER INSERT ON core_deposit_account_events
  FOR EACH ROW
  EXECUTE FUNCTION core_deposit_account_events_rollup_trigger();
