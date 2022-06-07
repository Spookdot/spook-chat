-- Add down migration script here
ALTER TABLE users_servers
DROP COLUMN owner,
DROP COLUMN manage_channels,
DROP COLUMN manage_users,
DROP COLUMN manage_invites,
DROP COLUMN banned;