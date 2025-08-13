-- Auto-generated rollup table for DepositAccountEvent
CREATE TABLE core_deposit_account_events_rollup (
  id UUID NOT NULL,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  account_holder_id UUID,
  description VARCHAR,
  ledger_account_id UUID,
  name VARCHAR,
  public_id VARCHAR,
  reference VARCHAR,
  status VARCHAR,

  -- Collection rollups
  audit_entry_ids BIGINT[]
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
    new_row.audit_entry_ids := CASE
       WHEN NEW.event ? 'audit_entry_ids' THEN
         ARRAY(SELECT value::text::BIGINT FROM jsonb_array_elements_text(NEW.event -> 'audit_entry_ids'))
       ELSE ARRAY[]::BIGINT[]
     END
;
    new_row.description := (NEW.event ->> 'description');
    new_row.ledger_account_id := (NEW.event ->> 'ledger_account_id')::UUID;
    new_row.name := (NEW.event ->> 'name');
    new_row.public_id := (NEW.event ->> 'public_id');
    new_row.reference := (NEW.event ->> 'reference');
    new_row.status := (NEW.event ->> 'status');
  ELSE
    -- Default all fields to current values
    new_row.account_holder_id := current_row.account_holder_id;
    new_row.audit_entry_ids := current_row.audit_entry_ids;
    new_row.description := current_row.description;
    new_row.ledger_account_id := current_row.ledger_account_id;
    new_row.name := current_row.name;
    new_row.public_id := current_row.public_id;
    new_row.reference := current_row.reference;
    new_row.status := current_row.status;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.account_holder_id := (NEW.event ->> 'account_holder_id')::UUID;
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.description := (NEW.event ->> 'description');
      new_row.ledger_account_id := (NEW.event ->> 'ledger_account_id')::UUID;
      new_row.name := (NEW.event ->> 'name');
      new_row.public_id := (NEW.event ->> 'public_id');
      new_row.reference := (NEW.event ->> 'reference');
      new_row.status := (NEW.event ->> 'status');
    WHEN 'account_status_updated' THEN
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.status := (NEW.event ->> 'status');
  END CASE;

  INSERT INTO core_deposit_account_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    account_holder_id,
    audit_entry_ids,
    description,
    ledger_account_id,
    name,
    public_id,
    reference,
    status
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.account_holder_id,
    new_row.audit_entry_ids,
    new_row.description,
    new_row.ledger_account_id,
    new_row.name,
    new_row.public_id,
    new_row.reference,
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
