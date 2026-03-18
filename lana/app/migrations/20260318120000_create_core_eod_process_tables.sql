CREATE TABLE core_eod_processes (
  id UUID PRIMARY KEY,
  date DATE NOT NULL,
  status VARCHAR NOT NULL DEFAULT 'Initialized',
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_eod_process_events (
  id UUID NOT NULL REFERENCES core_eod_processes(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);

CREATE INDEX idx_core_eod_processes_date ON core_eod_processes(date);
CREATE INDEX idx_core_eod_processes_status ON core_eod_processes(status);
