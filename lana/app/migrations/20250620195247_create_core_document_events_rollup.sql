-- Auto-generated rollup table for DocumentEvent
CREATE TABLE core_document_events_rollup (
  id UUID PRIMARY KEY,
  last_sequence INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  content_type VARCHAR,
  document_type VARCHAR,
  original_filename VARCHAR,
  path_in_storage VARCHAR,
  reference_id UUID,
  sanitized_filename VARCHAR,
  storage_identifier VARCHAR,
  error VARCHAR,

  -- Collection rollups
  audit_entry_ids BIGINT[],

  -- Toggle fields
  is_file_uploaded BOOLEAN DEFAULT false,
  is_deleted BOOLEAN DEFAULT false,
  is_archived BOOLEAN DEFAULT false

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

  -- Load the current rollup state
  SELECT * INTO current_row
  FROM core_document_events_rollup
  WHERE id = NEW.id;

  -- Early return if event is older than current state
  IF current_row.id IS NOT NULL AND NEW.sequence <= current_row.last_sequence THEN
    RETURN NEW;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'file_uploaded', 'upload_failed', 'download_link_generated', 'deleted', 'archived') THEN
    RAISE EXCEPTION 'Unknown event type: %', event_type;
  END IF;

  -- Construct the new row based on event type
  new_row.id := NEW.id;
  new_row.last_sequence := NEW.sequence;
  new_row.created_at := COALESCE(current_row.created_at, NEW.recorded_at);
  new_row.modified_at := NEW.recorded_at;

  -- Initialize fields with default values if this is a new record
  IF current_row.id IS NULL THEN
    new_row.content_type := (NEW.event ->> 'content_type');
    new_row.document_type := (NEW.event ->> 'document_type');
    new_row.original_filename := (NEW.event ->> 'original_filename');
    new_row.path_in_storage := (NEW.event ->> 'path_in_storage');
    new_row.reference_id := (NEW.event ->> 'reference_id')::UUID;
    new_row.sanitized_filename := (NEW.event ->> 'sanitized_filename');
    new_row.storage_identifier := (NEW.event ->> 'storage_identifier');
    new_row.error := (NEW.event ->> 'error');
    new_row.audit_entry_ids := CASE
       WHEN NEW.event ? 'audit_entry_ids' THEN
         ARRAY(SELECT value::text::BIGINT FROM jsonb_array_elements_text(NEW.event -> 'audit_entry_ids'))
       ELSE ARRAY[]::BIGINT[]
     END
;
    new_row.is_file_uploaded := false;
    new_row.is_deleted := false;
    new_row.is_archived := false;
  ELSE
    -- Default all fields to current values
    new_row.content_type := current_row.content_type;
    new_row.document_type := current_row.document_type;
    new_row.original_filename := current_row.original_filename;
    new_row.path_in_storage := current_row.path_in_storage;
    new_row.reference_id := current_row.reference_id;
    new_row.sanitized_filename := current_row.sanitized_filename;
    new_row.storage_identifier := current_row.storage_identifier;
    new_row.error := current_row.error;
    new_row.audit_entry_ids := current_row.audit_entry_ids;
    new_row.is_file_uploaded := current_row.is_file_uploaded;
    new_row.is_deleted := current_row.is_deleted;
    new_row.is_archived := current_row.is_archived;
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
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
    WHEN 'file_uploaded' THEN
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.is_file_uploaded := true;
    WHEN 'upload_failed' THEN
      new_row.error := (NEW.event ->> 'error');
    WHEN 'download_link_generated' THEN
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
    WHEN 'deleted' THEN
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.is_deleted := true;
    WHEN 'archived' THEN
      new_row.audit_entry_ids := array_append(COALESCE(current_row.audit_entry_ids, ARRAY[]::BIGINT[]), (NEW.event -> 'audit_info' ->> 'audit_entry_id')::BIGINT);
      new_row.is_archived := true;
  END CASE;

  INSERT INTO core_document_events_rollup (
    id,
    last_sequence,
    created_at,
    modified_at,
    content_type,
    document_type,
    original_filename,
    path_in_storage,
    reference_id,
    sanitized_filename,
    storage_identifier,
    error,
    audit_entry_ids,
    is_file_uploaded,
    is_deleted,
    is_archived
  )
  VALUES (
    new_row.id,
    new_row.last_sequence,
    new_row.created_at,
    new_row.modified_at,
    new_row.content_type,
    new_row.document_type,
    new_row.original_filename,
    new_row.path_in_storage,
    new_row.reference_id,
    new_row.sanitized_filename,
    new_row.storage_identifier,
    new_row.error,
    new_row.audit_entry_ids,
    new_row.is_file_uploaded,
    new_row.is_deleted,
    new_row.is_archived
  )
  ON CONFLICT (id) DO UPDATE SET
    last_sequence = EXCLUDED.last_sequence,
    modified_at = EXCLUDED.modified_at,
    content_type = EXCLUDED.content_type,
    document_type = EXCLUDED.document_type,
    original_filename = EXCLUDED.original_filename,
    path_in_storage = EXCLUDED.path_in_storage,
    reference_id = EXCLUDED.reference_id,
    sanitized_filename = EXCLUDED.sanitized_filename,
    storage_identifier = EXCLUDED.storage_identifier,
    error = EXCLUDED.error,
    audit_entry_ids = EXCLUDED.audit_entry_ids,
    is_file_uploaded = EXCLUDED.is_file_uploaded,
    is_deleted = EXCLUDED.is_deleted,
    is_archived = EXCLUDED.is_archived;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generated trigger for DocumentEvent
CREATE TRIGGER core_document_events_rollup_trigger
  AFTER INSERT ON core_document_events
  FOR EACH ROW
  EXECUTE FUNCTION core_document_events_rollup_trigger();
