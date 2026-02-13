-- Auto-generated rollup table for DocumentEvent
CREATE TABLE core_document_events_rollup (
  id UUID NOT NULL,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  event_type TEXT NOT NULL,
  -- Flattened fields from the event JSON
  content_type VARCHAR,
  document_type VARCHAR,
  error VARCHAR,
  original_filename VARCHAR,
  path_in_storage VARCHAR,
  reference_id UUID,
  sanitized_filename VARCHAR,
  storage_identifier VARCHAR,

  -- Toggle fields
  is_archived BOOLEAN DEFAULT false,
  is_deleted BOOLEAN DEFAULT false,
  is_file_uploaded BOOLEAN DEFAULT false
,
  PRIMARY KEY (id, version)
);


-- Auto-generated trigger function for DocumentEvent
CREATE OR REPLACE FUNCTION core_document_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_document_events_rollup%ROWTYPE;
  new_row core_document_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the previous version if this isn't the first event
  IF NEW.sequence > 1 THEN
    SELECT * INTO current_row
    FROM core_document_events_rollup
    WHERE id = NEW.id AND version = NEW.sequence - 1;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'file_uploaded', 'upload_failed', 'download_link_generated', 'deleted', 'archived') THEN
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
    new_row.content_type := (NEW.event ->> 'content_type');
    new_row.document_type := (NEW.event ->> 'document_type');
    new_row.error := (NEW.event ->> 'error');
    new_row.is_archived := false;
    new_row.is_deleted := false;
    new_row.is_file_uploaded := false;
    new_row.original_filename := (NEW.event ->> 'original_filename');
    new_row.path_in_storage := (NEW.event ->> 'path_in_storage');
    new_row.reference_id := (NEW.event ->> 'reference_id')::UUID;
    new_row.sanitized_filename := (NEW.event ->> 'sanitized_filename');
    new_row.storage_identifier := (NEW.event ->> 'storage_identifier');
  ELSE
    -- Default all fields to current values
    new_row.content_type := current_row.content_type;
    new_row.document_type := current_row.document_type;
    new_row.error := current_row.error;
    new_row.is_archived := current_row.is_archived;
    new_row.is_deleted := current_row.is_deleted;
    new_row.is_file_uploaded := current_row.is_file_uploaded;
    new_row.original_filename := current_row.original_filename;
    new_row.path_in_storage := current_row.path_in_storage;
    new_row.reference_id := current_row.reference_id;
    new_row.sanitized_filename := current_row.sanitized_filename;
    new_row.storage_identifier := current_row.storage_identifier;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.content_type := (NEW.event ->> 'content_type');
      new_row.document_type := (NEW.event ->> 'document_type');
      new_row.original_filename := (NEW.event ->> 'original_filename');
      new_row.path_in_storage := (NEW.event ->> 'path_in_storage');
      new_row.reference_id := (NEW.event ->> 'reference_id')::UUID;
      new_row.sanitized_filename := (NEW.event ->> 'sanitized_filename');
      new_row.storage_identifier := (NEW.event ->> 'storage_identifier');
    WHEN 'file_uploaded' THEN
      new_row.is_file_uploaded := true;
    WHEN 'upload_failed' THEN
      new_row.error := (NEW.event ->> 'error');
    WHEN 'download_link_generated' THEN
    WHEN 'deleted' THEN
      new_row.is_deleted := true;
    WHEN 'archived' THEN
      new_row.is_archived := true;
  END CASE;

  INSERT INTO core_document_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    event_type,
    content_type,
    document_type,
    error,
    is_archived,
    is_deleted,
    is_file_uploaded,
    original_filename,
    path_in_storage,
    reference_id,
    sanitized_filename,
    storage_identifier
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.event_type,
    new_row.content_type,
    new_row.document_type,
    new_row.error,
    new_row.is_archived,
    new_row.is_deleted,
    new_row.is_file_uploaded,
    new_row.original_filename,
    new_row.path_in_storage,
    new_row.reference_id,
    new_row.sanitized_filename,
    new_row.storage_identifier
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;


-- Auto-generated trigger for DocumentEvent
CREATE TRIGGER core_document_events_rollup_trigger
  AFTER INSERT ON core_document_events
  FOR EACH ROW
  EXECUTE FUNCTION core_document_events_rollup_trigger();
