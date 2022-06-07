-- Add up migration script here
CREATE TABLE IF NOT EXISTS invites (
  invite_id UUID PRIMARY KEY,
  server_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  expires_at TIMESTAMPTZ,
  FOREIGN KEY (server_id) REFERENCES servers (server_id)
);