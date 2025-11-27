CREATE TABLE domain_configurations (
  key TEXT PRIMARY KEY,
  value JSONB NOT NULL,
  updated_by TEXT NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL,
  reason TEXT NULL,
  correlation_id TEXT NULL
);

CREATE TABLE domain_configuration_events (
  id TEXT NOT NULL REFERENCES domain_configurations(key),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);
