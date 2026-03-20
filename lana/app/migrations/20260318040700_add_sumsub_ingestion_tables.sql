CREATE TABLE data_sumsub_applicants (
  applicant_id TEXT NOT NULL,
  external_user_id TEXT NOT NULL,
  raw_json JSONB NOT NULL,
  fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  PRIMARY KEY (applicant_id, fetched_at)
);

CREATE INDEX data_sumsub_applicants_external_user_id_idx
  ON data_sumsub_applicants (external_user_id, fetched_at DESC);

CREATE INDEX data_sumsub_applicants_applicant_id_idx
  ON data_sumsub_applicants (applicant_id, fetched_at DESC);

CREATE TABLE data_sumsub_documents (
  applicant_id TEXT NOT NULL,
  external_user_id TEXT NOT NULL,
  raw_json JSONB NOT NULL,
  fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  PRIMARY KEY (applicant_id, fetched_at)
);

CREATE INDEX data_sumsub_documents_external_user_id_idx
  ON data_sumsub_documents (external_user_id, fetched_at DESC);

CREATE INDEX data_sumsub_documents_applicant_id_idx
  ON data_sumsub_documents (applicant_id, fetched_at DESC);
