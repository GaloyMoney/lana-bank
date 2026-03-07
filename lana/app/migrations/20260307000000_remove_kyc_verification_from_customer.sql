-- Remove kyc_verification column from the rollup table
ALTER TABLE core_customer_events_rollup DROP COLUMN IF EXISTS kyc_verification;

-- Remove kyc_verification column from the main customers table
ALTER TABLE core_customers DROP COLUMN IF EXISTS kyc_verification;

-- Update trigger function to no longer reference kyc_verification and remove kyc_rejected event
CREATE OR REPLACE FUNCTION core_customer_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_customer_events_rollup%ROWTYPE;
  new_row core_customer_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the previous version if this isn't the first event
  IF NEW.sequence > 1 THEN
    SELECT * INTO current_row
    FROM core_customer_events_rollup
    WHERE id = NEW.id AND version = NEW.sequence - 1;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'activity_updated', 'kyc_rejected', 'closed', 'frozen', 'unfrozen') THEN
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
    new_row.activity := (NEW.event ->> 'activity');
    new_row.applicant_id := (NEW.event ->> 'applicant_id');
    new_row.customer_type := (NEW.event ->> 'customer_type');
    new_row.is_kyc_approved := false;
    new_row.level := (NEW.event ->> 'level');
    new_row.party_id := (NEW.event ->> 'party_id')::UUID;
    new_row.public_id := (NEW.event ->> 'public_id');
    new_row.status := 'active';
  ELSE
    -- Default all fields to current values
    new_row.activity := current_row.activity;
    new_row.applicant_id := current_row.applicant_id;
    new_row.customer_type := current_row.customer_type;
    new_row.is_kyc_approved := current_row.is_kyc_approved;
    new_row.level := current_row.level;
    new_row.party_id := current_row.party_id;
    new_row.public_id := current_row.public_id;
    new_row.status := current_row.status;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.activity := (NEW.event ->> 'activity');
      new_row.applicant_id := (NEW.event ->> 'applicant_id');
      new_row.customer_type := (NEW.event ->> 'customer_type');
      new_row.level := (NEW.event ->> 'level');
      new_row.party_id := (NEW.event ->> 'party_id')::UUID;
      new_row.public_id := (NEW.event ->> 'public_id');
    WHEN 'activity_updated' THEN
      new_row.activity := (NEW.event ->> 'activity');
    WHEN 'kyc_rejected' THEN
      -- legacy event type, no-op
    WHEN 'closed' THEN
      new_row.status := (NEW.event ->> 'status');
    WHEN 'frozen' THEN
      new_row.status := (NEW.event ->> 'status');
    WHEN 'unfrozen' THEN
      new_row.status := (NEW.event ->> 'status');
  END CASE;

  INSERT INTO core_customer_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    event_type,
    activity,
    applicant_id,
    customer_type,
    is_kyc_approved,
    level,
    party_id,
    public_id,
    status
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.event_type,
    new_row.activity,
    new_row.applicant_id,
    new_row.customer_type,
    new_row.is_kyc_approved,
    new_row.level,
    new_row.party_id,
    new_row.public_id,
    new_row.status
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;
