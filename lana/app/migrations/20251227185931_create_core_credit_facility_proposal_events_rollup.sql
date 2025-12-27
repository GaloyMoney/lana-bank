-- Auto-generated rollup table for CreditFacilityProposalEvent
CREATE TABLE core_credit_facility_proposal_events_rollup (
  id UUID NOT NULL,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  amount BIGINT,
  approval_process_id UUID,
  custodian_id UUID,
  customer_id UUID,
  customer_type VARCHAR,
  disbursal_credit_account_id UUID,
  obligations_repayment_from_account_id UUID,
  status VARCHAR,
  terms JSONB,

  -- Toggle fields
  is_approval_process_concluded BOOLEAN DEFAULT false
,
  PRIMARY KEY (id, version)
);

-- Auto-generated trigger function for CreditFacilityProposalEvent
CREATE OR REPLACE FUNCTION core_credit_facility_proposal_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_credit_facility_proposal_events_rollup%ROWTYPE;
  new_row core_credit_facility_proposal_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the previous version if this isn't the first event
  IF NEW.sequence > 1 THEN
    SELECT * INTO current_row
    FROM core_credit_facility_proposal_events_rollup
    WHERE id = NEW.id AND version = NEW.sequence - 1;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'customer_approval_concluded', 'approval_process_started', 'approval_process_concluded') THEN
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
    new_row.approval_process_id := (NEW.event ->> 'approval_process_id')::UUID;
    new_row.custodian_id := (NEW.event ->> 'custodian_id')::UUID;
    new_row.customer_id := (NEW.event ->> 'customer_id')::UUID;
    new_row.customer_type := (NEW.event ->> 'customer_type');
    new_row.disbursal_credit_account_id := (NEW.event ->> 'disbursal_credit_account_id')::UUID;
    new_row.is_approval_process_concluded := false;
    new_row.obligations_repayment_from_account_id := (NEW.event ->> 'obligations_repayment_from_account_id')::UUID;
    new_row.status := (NEW.event ->> 'status');
    new_row.terms := (NEW.event -> 'terms');
  ELSE
    -- Default all fields to current values
    new_row.amount := current_row.amount;
    new_row.approval_process_id := current_row.approval_process_id;
    new_row.custodian_id := current_row.custodian_id;
    new_row.customer_id := current_row.customer_id;
    new_row.customer_type := current_row.customer_type;
    new_row.disbursal_credit_account_id := current_row.disbursal_credit_account_id;
    new_row.is_approval_process_concluded := current_row.is_approval_process_concluded;
    new_row.obligations_repayment_from_account_id := current_row.obligations_repayment_from_account_id;
    new_row.status := current_row.status;
    new_row.terms := current_row.terms;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.amount := (NEW.event ->> 'amount')::BIGINT;
      new_row.custodian_id := (NEW.event ->> 'custodian_id')::UUID;
      new_row.customer_id := (NEW.event ->> 'customer_id')::UUID;
      new_row.customer_type := (NEW.event ->> 'customer_type');
      new_row.disbursal_credit_account_id := (NEW.event ->> 'disbursal_credit_account_id')::UUID;
      new_row.obligations_repayment_from_account_id := (NEW.event ->> 'obligations_repayment_from_account_id')::UUID;
      new_row.status := (NEW.event ->> 'status');
      new_row.terms := (NEW.event -> 'terms');
    WHEN 'customer_approval_concluded' THEN
      new_row.status := (NEW.event ->> 'status');
    WHEN 'approval_process_started' THEN
      new_row.approval_process_id := (NEW.event ->> 'approval_process_id')::UUID;
      new_row.status := (NEW.event ->> 'status');
    WHEN 'approval_process_concluded' THEN
      new_row.approval_process_id := (NEW.event ->> 'approval_process_id')::UUID;
      new_row.is_approval_process_concluded := true;
      new_row.status := (NEW.event ->> 'status');
  END CASE;

  INSERT INTO core_credit_facility_proposal_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    amount,
    approval_process_id,
    custodian_id,
    customer_id,
    customer_type,
    disbursal_credit_account_id,
    is_approval_process_concluded,
    obligations_repayment_from_account_id,
    status,
    terms
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.amount,
    new_row.approval_process_id,
    new_row.custodian_id,
    new_row.customer_id,
    new_row.customer_type,
    new_row.disbursal_credit_account_id,
    new_row.is_approval_process_concluded,
    new_row.obligations_repayment_from_account_id,
    new_row.status,
    new_row.terms
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generated trigger for CreditFacilityProposalEvent
CREATE TRIGGER core_credit_facility_proposal_events_rollup_trigger
  AFTER INSERT ON core_credit_facility_proposal_events
  FOR EACH ROW
  EXECUTE FUNCTION core_credit_facility_proposal_events_rollup_trigger();
