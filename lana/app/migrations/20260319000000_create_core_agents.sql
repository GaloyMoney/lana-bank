CREATE TABLE core_agents (
  id UUID PRIMARY KEY,
  name VARCHAR NOT NULL,
  keycloak_client_id VARCHAR NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE core_agent_events (
  id UUID NOT NULL REFERENCES core_agents(id),
  sequence INT NOT NULL,
  event_type VARCHAR NOT NULL,
  event JSONB NOT NULL,
  context JSONB DEFAULT NULL,
  recorded_at TIMESTAMPTZ NOT NULL,
  UNIQUE(id, sequence)
);
