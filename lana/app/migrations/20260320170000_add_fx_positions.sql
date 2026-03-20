CREATE TABLE core_fx_positions (
  id UUID PRIMARY KEY,
  currency VARCHAR NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_fx_position_events (
  id UUID NOT NULL REFERENCES core_fx_positions(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);
