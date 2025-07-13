-- Auto-generated rollup table for DisbursalEvent
CREATE TABLE core_disbursal_events_rollup (
  id UUID PRIMARY KEY,
  last_sequence INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  account_ids JSONB,
  amount BIGINT,
  approval_process_id UUID,
  approved BOOLEAN,
  disbursal_credit_account_id UUID,
  due_date TIMESTAMPTZ,
  effective VARCHAR,
  facility_id UUID,
  ledger_tx_id UUID,
  liquidation_date TIMESTAMPTZ,
  obligation_id UUID,
  overdue_date TIMESTAMPTZ,
  public_id VARCHAR,

  -- Collection rollups
  audit_entry_ids BIGINT[],

  -- Toggle fields
  is_approval_process_concluded BOOLEAN DEFAULT false,
  is_cancelled BOOLEAN DEFAULT false,
  is_settled BOOLEAN DEFAULT false

);

-- Auto-generated trigger function for DisbursalEvent
CREATE OR REPLACE FUNCTION core_disbursal_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_disbursal_events_rollup%ROWTYPE;
  new_row core_disbursal_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the current rollup state
  SELECT * INTO current_row
  FROM core_disbursal_events_rollup
  WHERE id = NEW.id;

  -- Early return if event is older than current state
  IF current_row.id IS NOT NULL AND NEW.sequence <= current_row.last_sequence THEN
    RETURN NEW;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'approval_process_concluded', 'settled', 'cancelled') THEN
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
    new_row.amount := (NEW.event ->> 'amount')::BIGINT;
    new_row.approval_process_id := (NEW.event ->> 'approval_process_id')::UUID;
    new_row.approved := (NEW.event ->> 'approved')::BOOLEAN;
    new_row.audit_entry_ids := CASE
       WHEN NEW.event ? 'audit_entry_ids' THEN
         ARRAY(SELECT value::text::BIGINT FROM jsonb_array_elements_text(NEW.event -> 'audit_entry_ids'))
       ELSE ARRAY[]::BIGINT[]
     END
;
    new_row.disbursal_credit_account_id := (NEW.event ->> 'disbursal_credit_account_id')::UUID;
    new_row.due_date := (NEW.event ->> 'due_date')::TIMESTAMPTZ;
    new_row.effective := (NEW.event ->> 'effective');
    new_row.facility_id := (NEW.event ->> 'facility_id')::UUID;
    new_row.is_approval_process_concluded := false;
    new_row.is_cancelled := false;
    new_row.is_settled := false;
    new_row.ledger_tx_id := (NEW.event ->> 'ledger_tx_id')::UUID;
    new_row.liquidation_date := (NEW.event ->> 'liquidation_date')::TIMESTAMPTZ;
    new_row.obligation_id := (NEW.event ->> 'obligation_id')::UUID;
    new_row.overdue_date := (NEW.event ->> 'overdue_date')::TIMESTAMPTZ;
    new_row.public_id := (NEW.event ->> 'public_id');
  ELSE
    -- Default all fields to current values
    new_row.account_ids := current_row.account_ids;
    new_row.amount := current_row.amount;
    new_row.approval_process_id := current_row.approval_process_id;
    new_row.approved := current_row.approved;
    new_row.audit_entry_ids := current_row.audit_entry_ids;
    new_row.disbursal_credit_account_id := current_row.disbursal_credit_account_id;
    new_row.due_date := current_row.due_date;
    new_row.effective := current_row.effective;
    new_row.facility_id := current_row.facility_id;
    new_row.is_approval_process_concluded := current_row.is_approval_process_concluded;
    new_row.is_cancelled := current_row.is_cancelled;
    new_row.is_settled := current_row.is_settled;
    new_row.ledger_tx_id := current_row.ledger_tx_id;
    new_row.liquidation_date := current_row.liquidation_date;
    new_row.obligation_id := current_row.obligation_id;
    new_row.overdue_date := current_row.overdue_date;
    new_row.public_id := current_row.public_id;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.account_ids := (NEW.event -> 'account_ids');
      new_row.amount := (NEW.event ->> 'amount')::BIGINT;
      new_row.approval_process_id := (NEW.event ->> 'approval_process_id')::UUID;
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.disbursal_credit_account_id := (NEW.event ->> 'disbursal_credit_account_id')::UUID;
      new_row.due_date := (NEW.event ->> 'due_date')::TIMESTAMPTZ;
      new_row.facility_id := (NEW.event ->> 'facility_id')::UUID;
      new_row.liquidation_date := (NEW.event ->> 'liquidation_date')::TIMESTAMPTZ;
      new_row.overdue_date := (NEW.event ->> 'overdue_date')::TIMESTAMPTZ;
      new_row.public_id := (NEW.event ->> 'public_id');
    WHEN 'approval_process_concluded' THEN
      new_row.approval_process_id := (NEW.event ->> 'approval_process_id')::UUID;
      new_row.approved := (NEW.event ->> 'approved')::BOOLEAN;
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.is_approval_process_concluded := true;
    WHEN 'settled' THEN
      new_row.amount := (NEW.event ->> 'amount')::BIGINT;
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.effective := (NEW.event ->> 'effective');
      new_row.is_settled := true;
      new_row.ledger_tx_id := (NEW.event ->> 'ledger_tx_id')::UUID;
      new_row.obligation_id := (NEW.event ->> 'obligation_id')::UUID;
    WHEN 'cancelled' THEN
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.is_cancelled := true;
      new_row.ledger_tx_id := (NEW.event ->> 'ledger_tx_id')::UUID;
  END CASE;

  INSERT INTO core_disbursal_events_rollup (
    id,
    last_sequence,
    created_at,
    modified_at,
    account_ids,
    amount,
    approval_process_id,
    approved,
    audit_entry_ids,
    disbursal_credit_account_id,
    due_date,
    effective,
    facility_id,
    is_approval_process_concluded,
    is_cancelled,
    is_settled,
    ledger_tx_id,
    liquidation_date,
    obligation_id,
    overdue_date,
    public_id
  )
  VALUES (
    new_row.id,
    new_row.last_sequence,
    new_row.created_at,
    new_row.modified_at,
    new_row.account_ids,
    new_row.amount,
    new_row.approval_process_id,
    new_row.approved,
    new_row.audit_entry_ids,
    new_row.disbursal_credit_account_id,
    new_row.due_date,
    new_row.effective,
    new_row.facility_id,
    new_row.is_approval_process_concluded,
    new_row.is_cancelled,
    new_row.is_settled,
    new_row.ledger_tx_id,
    new_row.liquidation_date,
    new_row.obligation_id,
    new_row.overdue_date,
    new_row.public_id
  )
  ON CONFLICT (id) DO UPDATE SET
    last_sequence = EXCLUDED.last_sequence,
    modified_at = EXCLUDED.modified_at,
    account_ids = EXCLUDED.account_ids,
    amount = EXCLUDED.amount,
    approval_process_id = EXCLUDED.approval_process_id,
    approved = EXCLUDED.approved,
    audit_entry_ids = EXCLUDED.audit_entry_ids,
    disbursal_credit_account_id = EXCLUDED.disbursal_credit_account_id,
    due_date = EXCLUDED.due_date,
    effective = EXCLUDED.effective,
    facility_id = EXCLUDED.facility_id,
    is_approval_process_concluded = EXCLUDED.is_approval_process_concluded,
    is_cancelled = EXCLUDED.is_cancelled,
    is_settled = EXCLUDED.is_settled,
    ledger_tx_id = EXCLUDED.ledger_tx_id,
    liquidation_date = EXCLUDED.liquidation_date,
    obligation_id = EXCLUDED.obligation_id,
    overdue_date = EXCLUDED.overdue_date,
    public_id = EXCLUDED.public_id;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generated trigger for DisbursalEvent
CREATE TRIGGER core_disbursal_events_rollup_trigger
  AFTER INSERT ON core_disbursal_events
  FOR EACH ROW
  EXECUTE FUNCTION core_disbursal_events_rollup_trigger();
