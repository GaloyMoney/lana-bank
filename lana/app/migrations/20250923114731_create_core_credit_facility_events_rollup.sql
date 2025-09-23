-- Auto-generated rollup table for CreditFacilityEvent
CREATE TABLE core_credit_facility_events_rollup (
  id UUID NOT NULL,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  account_ids JSONB,
  activated_at TIMESTAMPTZ,
  amount BIGINT,
  approval_process_id UUID,
  approved BOOLEAN,
  collateral BIGINT,
  collateral_id UUID,
  collateralization_ratio JSONB,
  collateralization_state VARCHAR,
  customer_id UUID,
  disbursal_credit_account_id UUID,
  interest_accrual_cycle_idx INTEGER,
  interest_period JSONB,
  outstanding JSONB,
  price JSONB,
  public_id VARCHAR,
  terms JSONB,

  -- Collection rollups
  interest_accrual_ids UUID[],
  ledger_tx_ids UUID[],
  obligation_ids UUID[],

  -- Toggle fields
  is_activated BOOLEAN DEFAULT false,
  is_approval_process_concluded BOOLEAN DEFAULT false,
  is_completed BOOLEAN DEFAULT false
,
  PRIMARY KEY (id, version)
);

-- Auto-generated trigger function for CreditFacilityEvent
CREATE OR REPLACE FUNCTION core_credit_facility_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_credit_facility_events_rollup%ROWTYPE;
  new_row core_credit_facility_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the previous version if this isn't the first event
  IF NEW.sequence > 1 THEN
    SELECT * INTO current_row
    FROM core_credit_facility_events_rollup
    WHERE id = NEW.id AND version = NEW.sequence - 1;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'approval_process_concluded', 'activated', 'interest_accrual_cycle_started', 'interest_accrual_cycle_concluded', 'collateralization_state_changed', 'collateralization_ratio_changed', 'matured', 'completed') THEN
    RAISE EXCEPTION 'Unknown event type: %', event_type;
  END IF;

  -- Construct the new row based on event type
  new_row.id := NEW.id;
  new_row.version := NEW.sequence;
  new_row.created_at := COALESCE(current_row.created_at, NEW.recorded_at);
  new_row.modified_at := NEW.recorded_at;

  -- Initialize fields with default values if this is a new record
  IF current_row.id IS NULL THEN
    new_row.account_ids := (NEW.event -> 'account_ids');
    new_row.activated_at := (NEW.event ->> 'activated_at')::TIMESTAMPTZ;
    new_row.amount := (NEW.event ->> 'amount')::BIGINT;
    new_row.approval_process_id := (NEW.event ->> 'approval_process_id')::UUID;
    new_row.approved := (NEW.event ->> 'approved')::BOOLEAN;
    new_row.collateral := (NEW.event ->> 'collateral')::BIGINT;
    new_row.collateral_id := (NEW.event ->> 'collateral_id')::UUID;
    new_row.collateralization_ratio := (NEW.event -> 'collateralization_ratio');
    new_row.collateralization_state := (NEW.event ->> 'collateralization_state');
    new_row.customer_id := (NEW.event ->> 'customer_id')::UUID;
    new_row.disbursal_credit_account_id := (NEW.event ->> 'disbursal_credit_account_id')::UUID;
    new_row.interest_accrual_cycle_idx := (NEW.event ->> 'interest_accrual_cycle_idx')::INTEGER;
    new_row.interest_accrual_ids := CASE
       WHEN NEW.event ? 'interest_accrual_ids' THEN
         ARRAY(SELECT value::text::UUID FROM jsonb_array_elements_text(NEW.event -> 'interest_accrual_ids'))
       ELSE ARRAY[]::UUID[]
     END
;
    new_row.interest_period := (NEW.event -> 'interest_period');
    new_row.is_activated := false;
    new_row.is_approval_process_concluded := false;
    new_row.is_completed := false;
    new_row.ledger_tx_ids := CASE
       WHEN NEW.event ? 'ledger_tx_ids' THEN
         ARRAY(SELECT value::text::UUID FROM jsonb_array_elements_text(NEW.event -> 'ledger_tx_ids'))
       ELSE ARRAY[]::UUID[]
     END
;
    new_row.obligation_ids := CASE
       WHEN NEW.event ? 'obligation_ids' THEN
         ARRAY(SELECT value::text::UUID FROM jsonb_array_elements_text(NEW.event -> 'obligation_ids'))
       ELSE ARRAY[]::UUID[]
     END
;
    new_row.outstanding := (NEW.event -> 'outstanding');
    new_row.price := (NEW.event -> 'price');
    new_row.public_id := (NEW.event ->> 'public_id');
    new_row.terms := (NEW.event -> 'terms');
  ELSE
    -- Default all fields to current values
    new_row.account_ids := current_row.account_ids;
    new_row.activated_at := current_row.activated_at;
    new_row.amount := current_row.amount;
    new_row.approval_process_id := current_row.approval_process_id;
    new_row.approved := current_row.approved;
    new_row.collateral := current_row.collateral;
    new_row.collateral_id := current_row.collateral_id;
    new_row.collateralization_ratio := current_row.collateralization_ratio;
    new_row.collateralization_state := current_row.collateralization_state;
    new_row.customer_id := current_row.customer_id;
    new_row.disbursal_credit_account_id := current_row.disbursal_credit_account_id;
    new_row.interest_accrual_cycle_idx := current_row.interest_accrual_cycle_idx;
    new_row.interest_accrual_ids := current_row.interest_accrual_ids;
    new_row.interest_period := current_row.interest_period;
    new_row.is_activated := current_row.is_activated;
    new_row.is_approval_process_concluded := current_row.is_approval_process_concluded;
    new_row.is_completed := current_row.is_completed;
    new_row.ledger_tx_ids := current_row.ledger_tx_ids;
    new_row.obligation_ids := current_row.obligation_ids;
    new_row.outstanding := current_row.outstanding;
    new_row.price := current_row.price;
    new_row.public_id := current_row.public_id;
    new_row.terms := current_row.terms;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.account_ids := (NEW.event -> 'account_ids');
      new_row.amount := (NEW.event ->> 'amount')::BIGINT;
      new_row.approval_process_id := (NEW.event ->> 'approval_process_id')::UUID;
      new_row.collateral_id := (NEW.event ->> 'collateral_id')::UUID;
      new_row.customer_id := (NEW.event ->> 'customer_id')::UUID;
      new_row.disbursal_credit_account_id := (NEW.event ->> 'disbursal_credit_account_id')::UUID;
      new_row.ledger_tx_ids := array_append(COALESCE(current_row.ledger_tx_ids, ARRAY[]::UUID[]), (NEW.event ->> 'ledger_tx_id')::UUID);
      new_row.public_id := (NEW.event ->> 'public_id');
      new_row.terms := (NEW.event -> 'terms');
    WHEN 'approval_process_concluded' THEN
      new_row.approval_process_id := (NEW.event ->> 'approval_process_id')::UUID;
      new_row.approved := (NEW.event ->> 'approved')::BOOLEAN;
      new_row.is_approval_process_concluded := true;
    WHEN 'activated' THEN
      new_row.activated_at := (NEW.event ->> 'activated_at')::TIMESTAMPTZ;
      new_row.is_activated := true;
      new_row.ledger_tx_ids := array_append(COALESCE(current_row.ledger_tx_ids, ARRAY[]::UUID[]), (NEW.event ->> 'ledger_tx_id')::UUID);
    WHEN 'interest_accrual_cycle_started' THEN
      new_row.interest_accrual_cycle_idx := (NEW.event ->> 'interest_accrual_cycle_idx')::INTEGER;
      new_row.interest_accrual_ids := array_append(COALESCE(current_row.interest_accrual_ids, ARRAY[]::UUID[]), (NEW.event ->> 'interest_accrual_id')::UUID);
      new_row.interest_period := (NEW.event -> 'interest_period');
    WHEN 'interest_accrual_cycle_concluded' THEN
      new_row.interest_accrual_cycle_idx := (NEW.event ->> 'interest_accrual_cycle_idx')::INTEGER;
      new_row.ledger_tx_ids := array_append(COALESCE(current_row.ledger_tx_ids, ARRAY[]::UUID[]), (NEW.event ->> 'ledger_tx_id')::UUID);
      new_row.obligation_ids := array_append(COALESCE(current_row.obligation_ids, ARRAY[]::UUID[]), (NEW.event ->> 'obligation_id')::UUID);
    WHEN 'collateralization_state_changed' THEN
      new_row.collateral := (NEW.event ->> 'collateral')::BIGINT;
      new_row.collateralization_state := (NEW.event ->> 'collateralization_state');
      new_row.outstanding := (NEW.event -> 'outstanding');
      new_row.price := (NEW.event -> 'price');
    WHEN 'collateralization_ratio_changed' THEN
      new_row.collateralization_ratio := (NEW.event -> 'collateralization_ratio');
    WHEN 'matured' THEN
    WHEN 'completed' THEN
      new_row.is_completed := true;
  END CASE;

  INSERT INTO core_credit_facility_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    account_ids,
    activated_at,
    amount,
    approval_process_id,
    approved,
    collateral,
    collateral_id,
    collateralization_ratio,
    collateralization_state,
    customer_id,
    disbursal_credit_account_id,
    interest_accrual_cycle_idx,
    interest_accrual_ids,
    interest_period,
    is_activated,
    is_approval_process_concluded,
    is_completed,
    ledger_tx_ids,
    obligation_ids,
    outstanding,
    price,
    public_id,
    terms
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.account_ids,
    new_row.activated_at,
    new_row.amount,
    new_row.approval_process_id,
    new_row.approved,
    new_row.collateral,
    new_row.collateral_id,
    new_row.collateralization_ratio,
    new_row.collateralization_state,
    new_row.customer_id,
    new_row.disbursal_credit_account_id,
    new_row.interest_accrual_cycle_idx,
    new_row.interest_accrual_ids,
    new_row.interest_period,
    new_row.is_activated,
    new_row.is_approval_process_concluded,
    new_row.is_completed,
    new_row.ledger_tx_ids,
    new_row.obligation_ids,
    new_row.outstanding,
    new_row.price,
    new_row.public_id,
    new_row.terms
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generated trigger for CreditFacilityEvent
CREATE TRIGGER core_credit_facility_events_rollup_trigger
  AFTER INSERT ON core_credit_facility_events
  FOR EACH ROW
  EXECUTE FUNCTION core_credit_facility_events_rollup_trigger();
