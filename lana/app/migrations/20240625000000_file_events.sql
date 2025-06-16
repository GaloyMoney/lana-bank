CREATE TABLE file_events (
  id UUID NOT NULL,
  file_kind VARCHAR NOT NULL,
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE(id, sequence, file_kind)
);
