-- Auto-generated rollup table for CollateralEvent
CREATE TABLE core_collateral_events_rollup (
  id UUID NOT NULL,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  abs_diff BIGINT,
  account_id UUID,
  action VARCHAR,
  collateral_amount BIGINT,
  credit_facility_id UUID,
  custody_wallet_id UUID,

  -- Collection rollups
  audit_entry_ids BIGINT[],
  ledger_tx_ids UUID[]
,
  PRIMARY KEY (id, version)
);

-- Auto-generated trigger function for CollateralEvent
CREATE OR REPLACE FUNCTION core_collateral_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_collateral_events_rollup%ROWTYPE;
  new_row core_collateral_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the previous version if this isn't the first event
  IF NEW.sequence > 1 THEN
    SELECT * INTO current_row
    FROM core_collateral_events_rollup
    WHERE id = NEW.id AND version = NEW.sequence - 1;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'updated_via_manual_input', 'updated_via_custodian_sync', 'updated') THEN
    RAISE EXCEPTION 'Unknown event type: %', event_type;
  END IF;

  -- Construct the new row based on event type
  new_row.id := NEW.id;
  new_row.version := NEW.sequence;
  new_row.created_at := COALESCE(current_row.created_at, NEW.recorded_at);
  new_row.modified_at := NEW.recorded_at;

  -- Initialize fields with default values if this is a new record
  IF current_row.id IS NULL THEN
    new_row.abs_diff := (NEW.event ->> 'abs_diff')::BIGINT;
    new_row.account_id := (NEW.event ->> 'account_id')::UUID;
    new_row.action := (NEW.event ->> 'action');
    new_row.audit_entry_ids := CASE
       WHEN NEW.event ? 'audit_entry_ids' THEN
         ARRAY(SELECT value::text::BIGINT FROM jsonb_array_elements_text(NEW.event -> 'audit_entry_ids'))
       ELSE ARRAY[]::BIGINT[]
     END
;
    new_row.collateral_amount := (NEW.event ->> 'collateral_amount')::BIGINT;
    new_row.credit_facility_id := (NEW.event ->> 'credit_facility_id')::UUID;
    new_row.custody_wallet_id := (NEW.event ->> 'custody_wallet_id')::UUID;
    new_row.ledger_tx_ids := CASE
       WHEN NEW.event ? 'ledger_tx_ids' THEN
         ARRAY(SELECT value::text::UUID FROM jsonb_array_elements_text(NEW.event -> 'ledger_tx_ids'))
       ELSE ARRAY[]::UUID[]
     END
;
  ELSE
    -- Default all fields to current values
    new_row.abs_diff := current_row.abs_diff;
    new_row.account_id := current_row.account_id;
    new_row.action := current_row.action;
    new_row.audit_entry_ids := current_row.audit_entry_ids;
    new_row.collateral_amount := current_row.collateral_amount;
    new_row.credit_facility_id := current_row.credit_facility_id;
    new_row.custody_wallet_id := current_row.custody_wallet_id;
    new_row.ledger_tx_ids := current_row.ledger_tx_ids;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.account_id := (NEW.event ->> 'account_id')::UUID;
      new_row.credit_facility_id := (NEW.event ->> 'credit_facility_id')::UUID;
      new_row.custody_wallet_id := (NEW.event ->> 'custody_wallet_id')::UUID;
    WHEN 'updated_via_manual_input' THEN
      new_row.abs_diff := (NEW.event ->> 'abs_diff')::BIGINT;
      new_row.action := (NEW.event ->> 'action');
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.collateral_amount := (NEW.event ->> 'collateral_amount')::BIGINT;
    WHEN 'updated_via_custodian_sync' THEN
      new_row.abs_diff := (NEW.event ->> 'abs_diff')::BIGINT;
      new_row.action := (NEW.event ->> 'action');
      new_row.collateral_amount := (NEW.event ->> 'collateral_amount')::BIGINT;
    WHEN 'updated' THEN
      new_row.ledger_tx_ids := array_append(COALESCE(current_row.ledger_tx_ids, ARRAY[]::UUID[]), (NEW.event ->> 'ledger_tx_id')::UUID);
  END CASE;

  INSERT INTO core_collateral_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    abs_diff,
    account_id,
    action,
    audit_entry_ids,
    collateral_amount,
    credit_facility_id,
    custody_wallet_id,
    ledger_tx_ids
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.abs_diff,
    new_row.account_id,
    new_row.action,
    new_row.audit_entry_ids,
    new_row.collateral_amount,
    new_row.credit_facility_id,
    new_row.custody_wallet_id,
    new_row.ledger_tx_ids
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generated trigger for CollateralEvent
CREATE TRIGGER core_collateral_events_rollup_trigger
  AFTER INSERT ON core_collateral_events
  FOR EACH ROW
  EXECUTE FUNCTION core_collateral_events_rollup_trigger();
