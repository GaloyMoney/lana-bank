-- Auto-generated rollup table for PaymentEvent
CREATE TABLE core_payment_events_rollup (
  id UUID PRIMARY KEY,
  last_sequence INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  amount JSONB,
  credit_facility_id UUID,
  disbursal JSONB,
  interest JSONB,

  -- Collection rollups
  audit_entry_ids BIGINT[],

  -- Toggle fields
  is_payment_allocated BOOLEAN DEFAULT false

);

-- Auto-generated trigger function for PaymentEvent
CREATE OR REPLACE FUNCTION core_payment_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_payment_events_rollup%ROWTYPE;
  new_row core_payment_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the current rollup state
  SELECT * INTO current_row
  FROM core_payment_events_rollup
  WHERE id = NEW.id;

  -- Early return if event is older than current state
  IF current_row.id IS NOT NULL AND NEW.sequence <= current_row.last_sequence THEN
    RETURN NEW;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'payment_allocated') THEN
    RAISE EXCEPTION 'Unknown event type: %', event_type;
  END IF;

  -- Construct the new row based on event type
  new_row.id := NEW.id;
  new_row.last_sequence := NEW.sequence;
  new_row.created_at := COALESCE(current_row.created_at, NEW.recorded_at);
  new_row.modified_at := NEW.recorded_at;

  -- Initialize fields with default values if this is a new record
  IF current_row.id IS NULL THEN
    new_row.amount := (NEW.event -> 'amount');
    new_row.credit_facility_id := (NEW.event ->> 'credit_facility_id')::UUID;
    new_row.disbursal := (NEW.event -> 'disbursal');
    new_row.interest := (NEW.event -> 'interest');
    new_row.audit_entry_ids := CASE
       WHEN NEW.event ? 'audit_entry_ids' THEN
         ARRAY(SELECT value::text::BIGINT FROM jsonb_array_elements_text(NEW.event -> 'audit_entry_ids'))
       ELSE ARRAY[]::BIGINT[]
     END
;
    new_row.is_payment_allocated := false;
  ELSE
    -- Default all fields to current values
    new_row.amount := current_row.amount;
    new_row.credit_facility_id := current_row.credit_facility_id;
    new_row.disbursal := current_row.disbursal;
    new_row.interest := current_row.interest;
    new_row.audit_entry_ids := current_row.audit_entry_ids;
    new_row.is_payment_allocated := current_row.is_payment_allocated;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.amount := (NEW.event -> 'amount');
      new_row.credit_facility_id := (NEW.event ->> 'credit_facility_id')::UUID;
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
    WHEN 'payment_allocated' THEN
      new_row.disbursal := (NEW.event -> 'disbursal');
      new_row.interest := (NEW.event -> 'interest');
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.is_payment_allocated := true;
  END CASE;

  INSERT INTO core_payment_events_rollup (
    id,
    last_sequence,
    created_at,
    modified_at,
    amount,
    credit_facility_id,
    disbursal,
    interest,
    audit_entry_ids,
    is_payment_allocated
  )
  VALUES (
    new_row.id,
    new_row.last_sequence,
    new_row.created_at,
    new_row.modified_at,
    new_row.amount,
    new_row.credit_facility_id,
    new_row.disbursal,
    new_row.interest,
    new_row.audit_entry_ids,
    new_row.is_payment_allocated
  )
  ON CONFLICT (id) DO UPDATE SET
    last_sequence = EXCLUDED.last_sequence,
    modified_at = EXCLUDED.modified_at,
    amount = EXCLUDED.amount,
    credit_facility_id = EXCLUDED.credit_facility_id,
    disbursal = EXCLUDED.disbursal,
    interest = EXCLUDED.interest,
    audit_entry_ids = EXCLUDED.audit_entry_ids,
    is_payment_allocated = EXCLUDED.is_payment_allocated;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generated trigger for PaymentEvent
CREATE TRIGGER core_payment_events_rollup_trigger
  AFTER INSERT ON core_payment_events
  FOR EACH ROW
  EXECUTE FUNCTION core_payment_events_rollup_trigger();
