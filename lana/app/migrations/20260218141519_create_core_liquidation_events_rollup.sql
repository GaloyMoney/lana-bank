-- Auto-generated rollup table for LiquidationEvent
CREATE TABLE core_liquidation_events_rollup (
  id UUID NOT NULL,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  event_type TEXT NOT NULL,
  -- Flattened fields from the event JSON
  account_ids JSONB,
  amount BIGINT,
  collateral_id UUID,
  initially_estimated_to_liquidate BIGINT,
  initially_expected_to_receive BIGINT,
  trigger_price BIGINT,

  -- Collection rollups
  ledger_tx_ids UUID[],

  -- Toggle fields
  is_completed BOOLEAN DEFAULT false
,
  PRIMARY KEY (id, version)
);


-- Auto-generated trigger function for LiquidationEvent
CREATE OR REPLACE FUNCTION core_liquidation_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_liquidation_events_rollup%ROWTYPE;
  new_row core_liquidation_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the previous version if this isn't the first event
  IF NEW.sequence > 1 THEN
    SELECT * INTO current_row
    FROM core_liquidation_events_rollup
    WHERE id = NEW.id AND version = NEW.sequence - 1;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'collateral_sent_out', 'proceeds_received_and_liquidation_completed', 'proceeds_from_liquidation_received') THEN
    RAISE EXCEPTION 'Unknown event type: %', event_type;
  END IF;

  -- Construct the new row based on event type
  new_row.id := NEW.id;
  new_row.version := NEW.sequence;
  new_row.created_at := COALESCE(current_row.created_at, NEW.recorded_at);
  new_row.modified_at := NEW.recorded_at;
  new_row.event_type := NEW.event_type;

  -- Initialize fields with default values if this is a new record
  IF current_row.id IS NULL THEN
    new_row.account_ids := (NEW.event -> 'account_ids');
    new_row.amount := (NEW.event ->> 'amount')::BIGINT;
    new_row.collateral_id := (NEW.event ->> 'collateral_id')::UUID;
    new_row.initially_estimated_to_liquidate := (NEW.event ->> 'initially_estimated_to_liquidate')::BIGINT;
    new_row.initially_expected_to_receive := (NEW.event ->> 'initially_expected_to_receive')::BIGINT;
    new_row.is_completed := false;
    new_row.ledger_tx_ids := CASE
       WHEN NEW.event ? 'ledger_tx_ids' THEN
         ARRAY(SELECT value::text::UUID FROM jsonb_array_elements_text(NEW.event -> 'ledger_tx_ids'))
       ELSE ARRAY[]::UUID[]
     END
;
    new_row.trigger_price := (NEW.event ->> 'trigger_price')::BIGINT;
  ELSE
    -- Default all fields to current values
    new_row.account_ids := current_row.account_ids;
    new_row.amount := current_row.amount;
    new_row.collateral_id := current_row.collateral_id;
    new_row.initially_estimated_to_liquidate := current_row.initially_estimated_to_liquidate;
    new_row.initially_expected_to_receive := current_row.initially_expected_to_receive;
    new_row.is_completed := current_row.is_completed;
    new_row.ledger_tx_ids := current_row.ledger_tx_ids;
    new_row.trigger_price := current_row.trigger_price;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.account_ids := (NEW.event -> 'account_ids');
      new_row.collateral_id := (NEW.event ->> 'collateral_id')::UUID;
      new_row.initially_estimated_to_liquidate := (NEW.event ->> 'initially_estimated_to_liquidate')::BIGINT;
      new_row.initially_expected_to_receive := (NEW.event ->> 'initially_expected_to_receive')::BIGINT;
      new_row.trigger_price := (NEW.event ->> 'trigger_price')::BIGINT;
    WHEN 'collateral_sent_out' THEN
      new_row.amount := (NEW.event ->> 'amount')::BIGINT;
      new_row.ledger_tx_ids := array_append(COALESCE(current_row.ledger_tx_ids, ARRAY[]::UUID[]), (NEW.event ->> 'ledger_tx_id')::UUID);
    WHEN 'proceeds_received_and_liquidation_completed' THEN
      new_row.amount := (NEW.event ->> 'amount')::BIGINT;
    WHEN 'proceeds_from_liquidation_received' THEN
      new_row.ledger_tx_ids := array_append(COALESCE(current_row.ledger_tx_ids, ARRAY[]::UUID[]), (NEW.event ->> 'ledger_tx_id')::UUID);
  END CASE;

  INSERT INTO core_liquidation_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    event_type,
    account_ids,
    amount,
    collateral_id,
    initially_estimated_to_liquidate,
    initially_expected_to_receive,
    is_completed,
    ledger_tx_ids,
    trigger_price
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.event_type,
    new_row.account_ids,
    new_row.amount,
    new_row.collateral_id,
    new_row.initially_estimated_to_liquidate,
    new_row.initially_expected_to_receive,
    new_row.is_completed,
    new_row.ledger_tx_ids,
    new_row.trigger_price
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;


-- Auto-generated trigger for LiquidationEvent
CREATE TRIGGER core_liquidation_events_rollup_trigger
  AFTER INSERT ON core_liquidation_events
  FOR EACH ROW
  EXECUTE FUNCTION core_liquidation_events_rollup_trigger();
