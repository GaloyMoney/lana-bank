-- Auto-generated rollup table for ProspectEvent
CREATE TABLE core_prospect_events_rollup (
  id UUID NOT NULL,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  event_type TEXT NOT NULL,
  -- Flattened fields from the event JSON
  applicant_id VARCHAR,
  customer_type VARCHAR,
  email VARCHAR,
  level VARCHAR,
  public_id VARCHAR,
  telegram_handle VARCHAR,

  -- Toggle fields
  is_kyc_approved BOOLEAN DEFAULT false
,
  PRIMARY KEY (id, version)
);


-- Auto-generated trigger function for ProspectEvent
CREATE OR REPLACE FUNCTION core_prospect_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_prospect_events_rollup%ROWTYPE;
  new_row core_prospect_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the previous version if this isn't the first event
  IF NEW.sequence > 1 THEN
    SELECT * INTO current_row
    FROM core_prospect_events_rollup
    WHERE id = NEW.id AND version = NEW.sequence - 1;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'kyc_started', 'kyc_approved', 'kyc_declined', 'manually_converted', 'closed', 'telegram_handle_updated') THEN
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
    new_row.applicant_id := (NEW.event ->> 'applicant_id');
    new_row.customer_type := (NEW.event ->> 'customer_type');
    new_row.email := (NEW.event ->> 'email');
    new_row.is_kyc_approved := false;
    new_row.level := (NEW.event ->> 'level');
    new_row.public_id := (NEW.event ->> 'public_id');
    new_row.telegram_handle := (NEW.event ->> 'telegram_handle');
  ELSE
    -- Default all fields to current values
    new_row.applicant_id := current_row.applicant_id;
    new_row.customer_type := current_row.customer_type;
    new_row.email := current_row.email;
    new_row.is_kyc_approved := current_row.is_kyc_approved;
    new_row.level := current_row.level;
    new_row.public_id := current_row.public_id;
    new_row.telegram_handle := current_row.telegram_handle;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.customer_type := (NEW.event ->> 'customer_type');
      new_row.email := (NEW.event ->> 'email');
      new_row.public_id := (NEW.event ->> 'public_id');
      new_row.telegram_handle := (NEW.event ->> 'telegram_handle');
    WHEN 'kyc_started' THEN
      new_row.applicant_id := (NEW.event ->> 'applicant_id');
    WHEN 'kyc_approved' THEN
      new_row.applicant_id := (NEW.event ->> 'applicant_id');
      new_row.is_kyc_approved := true;
      new_row.level := (NEW.event ->> 'level');
    WHEN 'kyc_declined' THEN
      new_row.applicant_id := (NEW.event ->> 'applicant_id');
    WHEN 'manually_converted' THEN
    WHEN 'closed' THEN
    WHEN 'telegram_handle_updated' THEN
      new_row.telegram_handle := (NEW.event ->> 'telegram_handle');
  END CASE;

  INSERT INTO core_prospect_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    event_type,
    applicant_id,
    customer_type,
    email,
    is_kyc_approved,
    level,
    public_id,
    telegram_handle
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.event_type,
    new_row.applicant_id,
    new_row.customer_type,
    new_row.email,
    new_row.is_kyc_approved,
    new_row.level,
    new_row.public_id,
    new_row.telegram_handle
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;


-- Auto-generated trigger for ProspectEvent
CREATE TRIGGER core_prospect_events_rollup_trigger
  AFTER INSERT ON core_prospect_events
  FOR EACH ROW
  EXECUTE FUNCTION core_prospect_events_rollup_trigger();
