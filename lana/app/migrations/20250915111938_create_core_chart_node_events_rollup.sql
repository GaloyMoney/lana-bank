-- Auto-generated rollup table for ChartNodeEvent
CREATE TABLE core_chart_node_events_rollup (
  id UUID NOT NULL,
  version INT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  modified_at TIMESTAMPTZ NOT NULL,
  -- Flattened fields from the event JSON
  chart_id UUID,
  spec JSONB,

  -- Collection rollups
  ledger_account_set_ids UUID[],
  manual_ledger_account_ids UUID[]
,
  PRIMARY KEY (id, version)
);

-- Auto-generated trigger function for ChartNodeEvent
CREATE OR REPLACE FUNCTION core_chart_node_events_rollup_trigger()
RETURNS TRIGGER AS $$
DECLARE
  event_type TEXT;
  current_row core_chart_node_events_rollup%ROWTYPE;
  new_row core_chart_node_events_rollup%ROWTYPE;
BEGIN
  event_type := NEW.event_type;

  -- Load the previous version if this isn't the first event
  IF NEW.sequence > 1 THEN
    SELECT * INTO current_row
    FROM core_chart_node_events_rollup
    WHERE id = NEW.id AND version = NEW.sequence - 1;
  END IF;

  -- Validate event type is known
  IF event_type NOT IN ('initialized', 'manual_transaction_account_assigned') THEN
    RAISE EXCEPTION 'Unknown event type: %', event_type;
  END IF;

  -- Construct the new row based on event type
  new_row.id := NEW.id;
  new_row.version := NEW.sequence;
  new_row.created_at := COALESCE(current_row.created_at, NEW.recorded_at);
  new_row.modified_at := NEW.recorded_at;

  -- Initialize fields with default values if this is a new record
  IF current_row.id IS NULL THEN
    new_row.chart_id := (NEW.event ->> 'chart_id')::UUID;
    new_row.ledger_account_set_ids := CASE
       WHEN NEW.event ? 'ledger_account_set_ids' THEN
         ARRAY(SELECT value::text::UUID FROM jsonb_array_elements_text(NEW.event -> 'ledger_account_set_ids'))
       ELSE ARRAY[]::UUID[]
     END
;
    new_row.manual_ledger_account_ids := CASE
       WHEN NEW.event ? 'manual_ledger_account_ids' THEN
         ARRAY(SELECT value::text::UUID FROM jsonb_array_elements_text(NEW.event -> 'manual_ledger_account_ids'))
       ELSE ARRAY[]::UUID[]
     END
;
    new_row.spec := (NEW.event -> 'spec');
  ELSE
    -- Default all fields to current values
    new_row.chart_id := current_row.chart_id;
    new_row.ledger_account_set_ids := current_row.ledger_account_set_ids;
    new_row.manual_ledger_account_ids := current_row.manual_ledger_account_ids;
    new_row.spec := current_row.spec;
  END IF;

  -- Update only the fields that are modified by the specific event
  CASE event_type
    WHEN 'initialized' THEN
      new_row.chart_id := (NEW.event ->> 'chart_id')::UUID;
      new_row.ledger_account_set_ids := array_append(COALESCE(current_row.ledger_account_set_ids, ARRAY[]::UUID[]), (NEW.event ->> 'ledger_account_set_id')::UUID);
      new_row.spec := (NEW.event -> 'spec');
    WHEN 'manual_transaction_account_assigned' THEN
      new_row.manual_ledger_account_ids := array_append(COALESCE(current_row.manual_ledger_account_ids, ARRAY[]::UUID[]), (NEW.event ->> 'ledger_account_id')::UUID);
  END CASE;

  INSERT INTO core_chart_node_events_rollup (
    id,
    version,
    created_at,
    modified_at,
    chart_id,
    ledger_account_set_ids,
    manual_ledger_account_ids,
    spec
  )
  VALUES (
    new_row.id,
    new_row.version,
    new_row.created_at,
    new_row.modified_at,
    new_row.chart_id,
    new_row.ledger_account_set_ids,
    new_row.manual_ledger_account_ids,
    new_row.spec
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generated trigger for ChartNodeEvent
CREATE TRIGGER core_chart_node_events_rollup_trigger
  AFTER INSERT ON core_chart_node_events
  FOR EACH ROW
  EXECUTE FUNCTION core_chart_node_events_rollup_trigger();
