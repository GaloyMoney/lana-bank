-- Auto-generated rollup table for CustomerEvent
CREATE TABLE core_customer_events_rollup (
  id UUID NOT NULL,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  activity VARCHAR,
  applicant_id VARCHAR,
  customer_type VARCHAR,
  email VARCHAR,
  kyc_verification VARCHAR,
  level VARCHAR,
  public_id VARCHAR,
  telegram_id VARCHAR,

  -- Toggle fields
  is_kyc_approved BOOLEAN DEFAULT false
,
  PRIMARY KEY (id, version)
);

-- Auto-generated trigger function for CustomerEvent
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
  IF event_type NOT IN ('initialized', 'kyc_started', 'kyc_approved', 'kyc_declined', 'kyc_verification_updated', 'telegram_id_updated', 'email_updated', 'activity_updated') THEN
    RAISE EXCEPTION 'Unknown event type: %', event_type;
  END IF;

  -- Construct the new row based on event type
  new_row.id := NEW.id;
  new_row.version := NEW.sequence;
  new_row.created_at := COALESCE(current_row.created_at, NEW.recorded_at);
  new_row.modified_at := NEW.recorded_at;

  -- Initialize fields with default values if this is a new record
  IF current_row.id IS NULL THEN
    new_row.activity := (NEW.event ->> 'activity');
    new_row.applicant_id := (NEW.event ->> 'applicant_id');
    new_row.customer_type := (NEW.event ->> 'customer_type');
    new_row.email := (NEW.event ->> 'email');
    new_row.is_kyc_approved := false;
    new_row.kyc_verification := (NEW.event ->> 'kyc_verification');
    new_row.level := (NEW.event ->> 'level');
    new_row.public_id := (NEW.event ->> 'public_id');
    new_row.telegram_id := (NEW.event ->> 'telegram_id');
  ELSE
    -- Default all fields to current values
    new_row.activity := current_row.activity;
    new_row.applicant_id := current_row.applicant_id;
    new_row.customer_type := current_row.customer_type;
    new_row.email := current_row.email;
    new_row.is_kyc_approved := current_row.is_kyc_approved;
    new_row.kyc_verification := current_row.kyc_verification;
    new_row.level := current_row.level;
    new_row.public_id := current_row.public_id;
    new_row.telegram_id := current_row.telegram_id;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.activity := (NEW.event ->> 'activity');
      new_row.customer_type := (NEW.event ->> 'customer_type');
      new_row.email := (NEW.event ->> 'email');
      new_row.public_id := (NEW.event ->> 'public_id');
      new_row.telegram_id := (NEW.event ->> 'telegram_id');
    WHEN 'kyc_started' THEN
      new_row.applicant_id := (NEW.event ->> 'applicant_id');
    WHEN 'kyc_approved' THEN
      new_row.applicant_id := (NEW.event ->> 'applicant_id');
      new_row.is_kyc_approved := true;
      new_row.level := (NEW.event ->> 'level');
    WHEN 'kyc_declined' THEN
      new_row.applicant_id := (NEW.event ->> 'applicant_id');
    WHEN 'kyc_verification_updated' THEN
      new_row.kyc_verification := (NEW.event ->> 'kyc_verification');
    WHEN 'telegram_id_updated' THEN
      new_row.telegram_id := (NEW.event ->> 'telegram_id');
    WHEN 'email_updated' THEN
      new_row.email := (NEW.event ->> 'email');
    WHEN 'activity_updated' THEN
      new_row.activity := (NEW.event ->> 'activity');
  END CASE;

  INSERT INTO core_customer_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    activity,
    applicant_id,
    customer_type,
    email,
    is_kyc_approved,
    kyc_verification,
    level,
    public_id,
    telegram_id
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.activity,
    new_row.applicant_id,
    new_row.customer_type,
    new_row.email,
    new_row.is_kyc_approved,
    new_row.kyc_verification,
    new_row.level,
    new_row.public_id,
    new_row.telegram_id
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generated trigger for CustomerEvent
CREATE TRIGGER core_customer_events_rollup_trigger
  AFTER INSERT ON core_customer_events
  FOR EACH ROW
  EXECUTE FUNCTION core_customer_events_rollup_trigger();
