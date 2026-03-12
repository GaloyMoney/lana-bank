CREATE TABLE core_accounting_templates (
  id UUID PRIMARY KEY,
  name VARCHAR NOT NULL UNIQUE,
  code VARCHAR NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_accounting_template_events (
  id UUID NOT NULL REFERENCES core_accounting_templates(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

