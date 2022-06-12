-- Add down migration script here
ALTER TABLE users_servers
ADD COLUMN owner BOOL NOT NULL DEFAULT FALSE;

ALTER TABLE servers
DROP COLUMN owner_id,
DROP FOREIGN KEY (owner_id);
