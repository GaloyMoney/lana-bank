-- Auto-generated rollup table for WalletEvent
CREATE TABLE core_wallet_events_rollup (
  id UUID NOT NULL,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  event_type TEXT NOT NULL,
  -- Flattened fields from the event JSON
  address VARCHAR,
  changed_at TIMESTAMPTZ,
  custodian_id UUID,
  custodian_response JSONB,
  external_wallet_id VARCHAR,
  network VARCHAR,
  new_balance BIGINT
,
  PRIMARY KEY (id, version)
);


-- Auto-generated trigger function for WalletEvent
CREATE OR REPLACE FUNCTION core_wallet_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_wallet_events_rollup%ROWTYPE;
  new_row core_wallet_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the previous version if this isn't the first event
  IF NEW.sequence > 1 THEN
    SELECT * INTO current_row
    FROM core_wallet_events_rollup
    WHERE id = NEW.id AND version = NEW.sequence - 1;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'balance_changed') THEN
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
    new_row.address := (NEW.event ->> 'address');
    new_row.changed_at := (NEW.event ->> 'changed_at')::TIMESTAMPTZ;
    new_row.custodian_id := (NEW.event ->> 'custodian_id')::UUID;
    new_row.custodian_response := (NEW.event -> 'custodian_response');
    new_row.external_wallet_id := (NEW.event ->> 'external_wallet_id');
    new_row.network := (NEW.event ->> 'network');
    new_row.new_balance := (NEW.event ->> 'new_balance')::BIGINT;
  ELSE
    -- Default all fields to current values
    new_row.address := current_row.address;
    new_row.changed_at := current_row.changed_at;
    new_row.custodian_id := current_row.custodian_id;
    new_row.custodian_response := current_row.custodian_response;
    new_row.external_wallet_id := current_row.external_wallet_id;
    new_row.network := current_row.network;
    new_row.new_balance := current_row.new_balance;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.address := (NEW.event ->> 'address');
      new_row.custodian_id := (NEW.event ->> 'custodian_id')::UUID;
      new_row.custodian_response := (NEW.event -> 'custodian_response');
      new_row.external_wallet_id := (NEW.event ->> 'external_wallet_id');
      new_row.network := (NEW.event ->> 'network');
    WHEN 'balance_changed' THEN
      new_row.changed_at := (NEW.event ->> 'changed_at')::TIMESTAMPTZ;
      new_row.new_balance := (NEW.event ->> 'new_balance')::BIGINT;
  END CASE;

  INSERT INTO core_wallet_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    event_type,
    address,
    changed_at,
    custodian_id,
    custodian_response,
    external_wallet_id,
    network,
    new_balance
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.event_type,
    new_row.address,
    new_row.changed_at,
    new_row.custodian_id,
    new_row.custodian_response,
    new_row.external_wallet_id,
    new_row.network,
    new_row.new_balance
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;


-- Auto-generated trigger for WalletEvent
CREATE TRIGGER core_wallet_events_rollup_trigger
  AFTER INSERT ON core_wallet_events
  FOR EACH ROW
  EXECUTE FUNCTION core_wallet_events_rollup_trigger();
