-- Auto-generated rollup table for LiquidationEvent
CREATE TABLE core_liquidation_events_rollup (
  id UUID NOT NULL,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  amount BIGINT,
  collateral_account_id UUID,
  collateral_id UUID,
  collateral_in_liquidation_account_id UUID,
  credit_facility_id UUID,
  current_price BIGINT,
  expected_to_receive BIGINT,
  facility_payment_holding_account_id UUID,
  facility_proceeds_from_liquidation_account_id UUID,
  facility_uncovered_outstanding_account_id UUID,
  initially_estimated_to_liquidate BIGINT,
  initially_expected_to_receive BIGINT,
  ledger_tx_id UUID,
  liquidated_collateral_account_id UUID,
  liquidation_proceeds_omnibus_account_id UUID,
  outstanding BIGINT,
  payment_id UUID,
  to_liquidate_at_current_price BIGINT,
  trigger_price BIGINT,

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
  IF event_type NOT IN ('initialized', 'updated', 'collateral_sent_out', 'proceeds_from_liquidation_received', 'completed') THEN
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
    new_row.collateral_account_id := (NEW.event ->> 'collateral_account_id')::UUID;
    new_row.collateral_id := (NEW.event ->> 'collateral_id')::UUID;
    new_row.collateral_in_liquidation_account_id := (NEW.event ->> 'collateral_in_liquidation_account_id')::UUID;
    new_row.credit_facility_id := (NEW.event ->> 'credit_facility_id')::UUID;
    new_row.current_price := (NEW.event ->> 'current_price')::BIGINT;
    new_row.expected_to_receive := (NEW.event ->> 'expected_to_receive')::BIGINT;
    new_row.facility_payment_holding_account_id := (NEW.event ->> 'facility_payment_holding_account_id')::UUID;
    new_row.facility_proceeds_from_liquidation_account_id := (NEW.event ->> 'facility_proceeds_from_liquidation_account_id')::UUID;
    new_row.facility_uncovered_outstanding_account_id := (NEW.event ->> 'facility_uncovered_outstanding_account_id')::UUID;
    new_row.initially_estimated_to_liquidate := (NEW.event ->> 'initially_estimated_to_liquidate')::BIGINT;
    new_row.initially_expected_to_receive := (NEW.event ->> 'initially_expected_to_receive')::BIGINT;
    new_row.is_completed := false;
    new_row.ledger_tx_id := (NEW.event ->> 'ledger_tx_id')::UUID;
    new_row.liquidated_collateral_account_id := (NEW.event ->> 'liquidated_collateral_account_id')::UUID;
    new_row.liquidation_proceeds_omnibus_account_id := (NEW.event ->> 'liquidation_proceeds_omnibus_account_id')::UUID;
    new_row.outstanding := (NEW.event ->> 'outstanding')::BIGINT;
    new_row.payment_id := (NEW.event ->> 'payment_id')::UUID;
    new_row.to_liquidate_at_current_price := (NEW.event ->> 'to_liquidate_at_current_price')::BIGINT;
    new_row.trigger_price := (NEW.event ->> 'trigger_price')::BIGINT;
  ELSE
    -- Default all fields to current values
    new_row.amount := current_row.amount;
    new_row.collateral_account_id := current_row.collateral_account_id;
    new_row.collateral_id := current_row.collateral_id;
    new_row.collateral_in_liquidation_account_id := current_row.collateral_in_liquidation_account_id;
    new_row.credit_facility_id := current_row.credit_facility_id;
    new_row.current_price := current_row.current_price;
    new_row.expected_to_receive := current_row.expected_to_receive;
    new_row.facility_payment_holding_account_id := current_row.facility_payment_holding_account_id;
    new_row.facility_proceeds_from_liquidation_account_id := current_row.facility_proceeds_from_liquidation_account_id;
    new_row.facility_uncovered_outstanding_account_id := current_row.facility_uncovered_outstanding_account_id;
    new_row.initially_estimated_to_liquidate := current_row.initially_estimated_to_liquidate;
    new_row.initially_expected_to_receive := current_row.initially_expected_to_receive;
    new_row.is_completed := current_row.is_completed;
    new_row.ledger_tx_id := current_row.ledger_tx_id;
    new_row.liquidated_collateral_account_id := current_row.liquidated_collateral_account_id;
    new_row.liquidation_proceeds_omnibus_account_id := current_row.liquidation_proceeds_omnibus_account_id;
    new_row.outstanding := current_row.outstanding;
    new_row.payment_id := current_row.payment_id;
    new_row.to_liquidate_at_current_price := current_row.to_liquidate_at_current_price;
    new_row.trigger_price := current_row.trigger_price;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.collateral_account_id := (NEW.event ->> 'collateral_account_id')::UUID;
      new_row.collateral_id := (NEW.event ->> 'collateral_id')::UUID;
      new_row.collateral_in_liquidation_account_id := (NEW.event ->> 'collateral_in_liquidation_account_id')::UUID;
      new_row.credit_facility_id := (NEW.event ->> 'credit_facility_id')::UUID;
      new_row.facility_payment_holding_account_id := (NEW.event ->> 'facility_payment_holding_account_id')::UUID;
      new_row.facility_proceeds_from_liquidation_account_id := (NEW.event ->> 'facility_proceeds_from_liquidation_account_id')::UUID;
      new_row.facility_uncovered_outstanding_account_id := (NEW.event ->> 'facility_uncovered_outstanding_account_id')::UUID;
      new_row.initially_estimated_to_liquidate := (NEW.event ->> 'initially_estimated_to_liquidate')::BIGINT;
      new_row.initially_expected_to_receive := (NEW.event ->> 'initially_expected_to_receive')::BIGINT;
      new_row.liquidated_collateral_account_id := (NEW.event ->> 'liquidated_collateral_account_id')::UUID;
      new_row.liquidation_proceeds_omnibus_account_id := (NEW.event ->> 'liquidation_proceeds_omnibus_account_id')::UUID;
      new_row.trigger_price := (NEW.event ->> 'trigger_price')::BIGINT;
    WHEN 'updated' THEN
      new_row.current_price := (NEW.event ->> 'current_price')::BIGINT;
      new_row.expected_to_receive := (NEW.event ->> 'expected_to_receive')::BIGINT;
      new_row.outstanding := (NEW.event ->> 'outstanding')::BIGINT;
      new_row.to_liquidate_at_current_price := (NEW.event ->> 'to_liquidate_at_current_price')::BIGINT;
    WHEN 'collateral_sent_out' THEN
      new_row.amount := (NEW.event ->> 'amount')::BIGINT;
      new_row.ledger_tx_id := (NEW.event ->> 'ledger_tx_id')::UUID;
    WHEN 'proceeds_from_liquidation_received' THEN
      new_row.amount := (NEW.event ->> 'amount')::BIGINT;
      new_row.ledger_tx_id := (NEW.event ->> 'ledger_tx_id')::UUID;
      new_row.payment_id := (NEW.event ->> 'payment_id')::UUID;
    WHEN 'completed' THEN
      new_row.is_completed := true;
  END CASE;

  INSERT INTO core_liquidation_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    amount,
    collateral_account_id,
    collateral_id,
    collateral_in_liquidation_account_id,
    credit_facility_id,
    current_price,
    expected_to_receive,
    facility_payment_holding_account_id,
    facility_proceeds_from_liquidation_account_id,
    facility_uncovered_outstanding_account_id,
    initially_estimated_to_liquidate,
    initially_expected_to_receive,
    is_completed,
    ledger_tx_id,
    liquidated_collateral_account_id,
    liquidation_proceeds_omnibus_account_id,
    outstanding,
    payment_id,
    to_liquidate_at_current_price,
    trigger_price
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.amount,
    new_row.collateral_account_id,
    new_row.collateral_id,
    new_row.collateral_in_liquidation_account_id,
    new_row.credit_facility_id,
    new_row.current_price,
    new_row.expected_to_receive,
    new_row.facility_payment_holding_account_id,
    new_row.facility_proceeds_from_liquidation_account_id,
    new_row.facility_uncovered_outstanding_account_id,
    new_row.initially_estimated_to_liquidate,
    new_row.initially_expected_to_receive,
    new_row.is_completed,
    new_row.ledger_tx_id,
    new_row.liquidated_collateral_account_id,
    new_row.liquidation_proceeds_omnibus_account_id,
    new_row.outstanding,
    new_row.payment_id,
    new_row.to_liquidate_at_current_price,
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
