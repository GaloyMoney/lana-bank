-- Auto-generated rollup table for ObligationEvent
CREATE TABLE core_obligation_events_rollup (
  id UUID NOT NULL,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  amount BIGINT,
  audit_info JSONB,
  credit_facility_id UUID,
  defaulted_account_id UUID,
  defaulted_amount BIGINT,
  defaulted_date JSONB,
  due_accounts JSONB,
  due_amount BIGINT,
  due_date VARCHAR,
  effective VARCHAR,
  in_liquidation_account_id UUID,
  initial_amount BIGINT,
  liquidation_date JSONB,
  liquidation_process_id UUID,
  not_yet_due_accounts JSONB,
  obligation_installment_amount BIGINT,
  obligation_type VARCHAR,
  overdue_accounts JSONB,
  overdue_amount BIGINT,
  overdue_date JSONB,
  payment_id UUID,
  reference VARCHAR,

  -- Collection rollups
  ledger_tx_ids UUID[],
  obligation_installment_ids UUID[],

  -- Toggle fields
  is_completed BOOLEAN DEFAULT false,
  is_defaulted_recorded BOOLEAN DEFAULT false,
  is_due_recorded BOOLEAN DEFAULT false,
  is_overdue_recorded BOOLEAN DEFAULT false
,
  PRIMARY KEY (id, version)
);

-- Auto-generated trigger function for ObligationEvent
CREATE OR REPLACE FUNCTION core_obligation_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_obligation_events_rollup%ROWTYPE;
  new_row core_obligation_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the previous version if this isn't the first event
  IF NEW.sequence > 1 THEN
    SELECT * INTO current_row
    FROM core_obligation_events_rollup
    WHERE id = NEW.id AND version = NEW.sequence - 1;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'due_recorded', 'overdue_recorded', 'defaulted_recorded', 'installment_applied', 'liquidation_process_started', 'liquidation_process_concluded', 'completed') THEN
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
    new_row.audit_info := (NEW.event -> 'audit_info');
    new_row.credit_facility_id := (NEW.event ->> 'credit_facility_id')::UUID;
    new_row.defaulted_account_id := (NEW.event ->> 'defaulted_account_id')::UUID;
    new_row.defaulted_amount := (NEW.event ->> 'defaulted_amount')::BIGINT;
    new_row.defaulted_date := (NEW.event -> 'defaulted_date');
    new_row.due_accounts := (NEW.event -> 'due_accounts');
    new_row.due_amount := (NEW.event ->> 'due_amount')::BIGINT;
    new_row.due_date := (NEW.event ->> 'due_date');
    new_row.effective := (NEW.event ->> 'effective');
    new_row.in_liquidation_account_id := (NEW.event ->> 'in_liquidation_account_id')::UUID;
    new_row.initial_amount := (NEW.event ->> 'initial_amount')::BIGINT;
    new_row.is_completed := false;
    new_row.is_defaulted_recorded := false;
    new_row.is_due_recorded := false;
    new_row.is_overdue_recorded := false;
    new_row.ledger_tx_ids := CASE
       WHEN NEW.event ? 'ledger_tx_ids' THEN
         ARRAY(SELECT value::text::UUID FROM jsonb_array_elements_text(NEW.event -> 'ledger_tx_ids'))
       ELSE ARRAY[]::UUID[]
     END
;
    new_row.liquidation_date := (NEW.event -> 'liquidation_date');
    new_row.liquidation_process_id := (NEW.event ->> 'liquidation_process_id')::UUID;
    new_row.not_yet_due_accounts := (NEW.event -> 'not_yet_due_accounts');
    new_row.obligation_installment_amount := (NEW.event ->> 'obligation_installment_amount')::BIGINT;
    new_row.obligation_installment_ids := CASE
       WHEN NEW.event ? 'obligation_installment_ids' THEN
         ARRAY(SELECT value::text::UUID FROM jsonb_array_elements_text(NEW.event -> 'obligation_installment_ids'))
       ELSE ARRAY[]::UUID[]
     END
;
    new_row.obligation_type := (NEW.event ->> 'obligation_type');
    new_row.overdue_accounts := (NEW.event -> 'overdue_accounts');
    new_row.overdue_amount := (NEW.event ->> 'overdue_amount')::BIGINT;
    new_row.overdue_date := (NEW.event -> 'overdue_date');
    new_row.payment_id := (NEW.event ->> 'payment_id')::UUID;
    new_row.reference := (NEW.event ->> 'reference');
  ELSE
    -- Default all fields to current values
    new_row.amount := current_row.amount;
    new_row.audit_info := current_row.audit_info;
    new_row.credit_facility_id := current_row.credit_facility_id;
    new_row.defaulted_account_id := current_row.defaulted_account_id;
    new_row.defaulted_amount := current_row.defaulted_amount;
    new_row.defaulted_date := current_row.defaulted_date;
    new_row.due_accounts := current_row.due_accounts;
    new_row.due_amount := current_row.due_amount;
    new_row.due_date := current_row.due_date;
    new_row.effective := current_row.effective;
    new_row.in_liquidation_account_id := current_row.in_liquidation_account_id;
    new_row.initial_amount := current_row.initial_amount;
    new_row.is_completed := current_row.is_completed;
    new_row.is_defaulted_recorded := current_row.is_defaulted_recorded;
    new_row.is_due_recorded := current_row.is_due_recorded;
    new_row.is_overdue_recorded := current_row.is_overdue_recorded;
    new_row.ledger_tx_ids := current_row.ledger_tx_ids;
    new_row.liquidation_date := current_row.liquidation_date;
    new_row.liquidation_process_id := current_row.liquidation_process_id;
    new_row.not_yet_due_accounts := current_row.not_yet_due_accounts;
    new_row.obligation_installment_amount := current_row.obligation_installment_amount;
    new_row.obligation_installment_ids := current_row.obligation_installment_ids;
    new_row.obligation_type := current_row.obligation_type;
    new_row.overdue_accounts := current_row.overdue_accounts;
    new_row.overdue_amount := current_row.overdue_amount;
    new_row.overdue_date := current_row.overdue_date;
    new_row.payment_id := current_row.payment_id;
    new_row.reference := current_row.reference;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.amount := (NEW.event ->> 'amount')::BIGINT;
      new_row.audit_info := (NEW.event -> 'audit_info');
      new_row.credit_facility_id := (NEW.event ->> 'credit_facility_id')::UUID;
      new_row.defaulted_account_id := (NEW.event ->> 'defaulted_account_id')::UUID;
      new_row.defaulted_date := (NEW.event -> 'defaulted_date');
      new_row.due_accounts := (NEW.event -> 'due_accounts');
      new_row.due_date := (NEW.event ->> 'due_date');
      new_row.effective := (NEW.event ->> 'effective');
      new_row.in_liquidation_account_id := (NEW.event ->> 'in_liquidation_account_id')::UUID;
      new_row.ledger_tx_ids := array_append(COALESCE(current_row.ledger_tx_ids, ARRAY[]::UUID[]), (NEW.event ->> 'ledger_tx_id')::UUID);
      new_row.liquidation_date := (NEW.event -> 'liquidation_date');
      new_row.not_yet_due_accounts := (NEW.event -> 'not_yet_due_accounts');
      new_row.obligation_type := (NEW.event ->> 'obligation_type');
      new_row.overdue_accounts := (NEW.event -> 'overdue_accounts');
      new_row.overdue_date := (NEW.event -> 'overdue_date');
      new_row.reference := (NEW.event ->> 'reference');
    WHEN 'due_recorded' THEN
      new_row.audit_info := (NEW.event -> 'audit_info');
      new_row.due_amount := (NEW.event ->> 'due_amount')::BIGINT;
      new_row.is_due_recorded := true;
      new_row.ledger_tx_ids := array_append(COALESCE(current_row.ledger_tx_ids, ARRAY[]::UUID[]), (NEW.event ->> 'ledger_tx_id')::UUID);
    WHEN 'overdue_recorded' THEN
      new_row.audit_info := (NEW.event -> 'audit_info');
      new_row.is_overdue_recorded := true;
      new_row.ledger_tx_ids := array_append(COALESCE(current_row.ledger_tx_ids, ARRAY[]::UUID[]), (NEW.event ->> 'ledger_tx_id')::UUID);
      new_row.overdue_amount := (NEW.event ->> 'overdue_amount')::BIGINT;
    WHEN 'defaulted_recorded' THEN
      new_row.audit_info := (NEW.event -> 'audit_info');
      new_row.defaulted_amount := (NEW.event ->> 'defaulted_amount')::BIGINT;
      new_row.is_defaulted_recorded := true;
      new_row.ledger_tx_ids := array_append(COALESCE(current_row.ledger_tx_ids, ARRAY[]::UUID[]), (NEW.event ->> 'ledger_tx_id')::UUID);
    WHEN 'installment_applied' THEN
      new_row.ledger_tx_ids := array_append(COALESCE(current_row.ledger_tx_ids, ARRAY[]::UUID[]), (NEW.event ->> 'ledger_tx_id')::UUID);
      new_row.obligation_installment_amount := (NEW.event ->> 'obligation_installment_amount')::BIGINT;
      new_row.obligation_installment_ids := array_append(COALESCE(current_row.obligation_installment_ids, ARRAY[]::UUID[]), (NEW.event ->> 'obligation_installment_id')::UUID);
      new_row.payment_id := (NEW.event ->> 'payment_id')::UUID;
    WHEN 'liquidation_process_started' THEN
      new_row.audit_info := (NEW.event -> 'audit_info');
      new_row.effective := (NEW.event ->> 'effective');
      new_row.initial_amount := (NEW.event ->> 'initial_amount')::BIGINT;
      new_row.liquidation_process_id := (NEW.event ->> 'liquidation_process_id')::UUID;
    WHEN 'liquidation_process_concluded' THEN
      new_row.audit_info := (NEW.event -> 'audit_info');
      new_row.liquidation_process_id := (NEW.event ->> 'liquidation_process_id')::UUID;
    WHEN 'completed' THEN
      new_row.audit_info := (NEW.event -> 'audit_info');
      new_row.effective := (NEW.event ->> 'effective');
      new_row.is_completed := true;
  END CASE;

  INSERT INTO core_obligation_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    amount,
    audit_info,
    credit_facility_id,
    defaulted_account_id,
    defaulted_amount,
    defaulted_date,
    due_accounts,
    due_amount,
    due_date,
    effective,
    in_liquidation_account_id,
    initial_amount,
    is_completed,
    is_defaulted_recorded,
    is_due_recorded,
    is_overdue_recorded,
    ledger_tx_ids,
    liquidation_date,
    liquidation_process_id,
    not_yet_due_accounts,
    obligation_installment_amount,
    obligation_installment_ids,
    obligation_type,
    overdue_accounts,
    overdue_amount,
    overdue_date,
    payment_id,
    reference
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.amount,
    new_row.audit_info,
    new_row.credit_facility_id,
    new_row.defaulted_account_id,
    new_row.defaulted_amount,
    new_row.defaulted_date,
    new_row.due_accounts,
    new_row.due_amount,
    new_row.due_date,
    new_row.effective,
    new_row.in_liquidation_account_id,
    new_row.initial_amount,
    new_row.is_completed,
    new_row.is_defaulted_recorded,
    new_row.is_due_recorded,
    new_row.is_overdue_recorded,
    new_row.ledger_tx_ids,
    new_row.liquidation_date,
    new_row.liquidation_process_id,
    new_row.not_yet_due_accounts,
    new_row.obligation_installment_amount,
    new_row.obligation_installment_ids,
    new_row.obligation_type,
    new_row.overdue_accounts,
    new_row.overdue_amount,
    new_row.overdue_date,
    new_row.payment_id,
    new_row.reference
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generated trigger for ObligationEvent
CREATE TRIGGER core_obligation_events_rollup_trigger
  AFTER INSERT ON core_obligation_events
  FOR EACH ROW
  EXECUTE FUNCTION core_obligation_events_rollup_trigger();
