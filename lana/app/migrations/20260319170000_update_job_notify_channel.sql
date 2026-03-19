-- Update job notification triggers for job crate 0.6.9
-- The channel changed from 'job_execution' (plain text) to 'job_events' (JSON payload)
-- and the trigger structure changed from per-operation functions to a single function.

-- Drop old per-operation triggers
DROP TRIGGER IF EXISTS job_executions_notify_insert_trigger ON job_executions;
DROP TRIGGER IF EXISTS job_executions_notify_update_trigger ON job_executions;
DROP TRIGGER IF EXISTS job_executions_notify_delete_trigger ON job_executions;

-- Drop old per-operation functions
DROP FUNCTION IF EXISTS notify_job_execution_insert();
DROP FUNCTION IF EXISTS notify_job_execution_update();
DROP FUNCTION IF EXISTS notify_job_execution_delete();

-- Create the unified notify function used by job 0.6.9
CREATE OR REPLACE FUNCTION notify_job_event() RETURNS TRIGGER AS $$
BEGIN
  IF TG_OP = 'INSERT' THEN
    PERFORM pg_notify('job_events',
      json_build_object('type', 'execution_ready', 'job_type', NEW.job_type)::text);
    RETURN NULL;
  END IF;

  IF TG_OP = 'UPDATE' THEN
    IF NEW.execute_at IS DISTINCT FROM OLD.execute_at THEN
      PERFORM pg_notify('job_events',
        json_build_object('type', 'execution_ready', 'job_type', NEW.job_type)::text);
    END IF;
    RETURN NULL;
  END IF;

  IF TG_OP = 'DELETE' THEN
    PERFORM pg_notify('job_events',
      json_build_object('type', 'job_terminal', 'job_id', OLD.id::text)::text);
    IF OLD.queue_id IS NOT NULL THEN
      PERFORM pg_notify('job_events',
        json_build_object('type', 'execution_ready', 'job_type', OLD.job_type)::text);
    END IF;
    RETURN NULL;
  END IF;

  RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Create the unified trigger
CREATE TRIGGER job_executions_notify_event_trigger
AFTER INSERT OR UPDATE OR DELETE ON job_executions
FOR EACH ROW
EXECUTE FUNCTION notify_job_event();
