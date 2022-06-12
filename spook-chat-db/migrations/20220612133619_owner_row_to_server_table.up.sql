-- Add up migration script here
ALTER TABLE users_servers 
DROP COLUMN owner;

ALTER TABLE servers
ADD COLUMN owner_id UUID NOT NULL,
ADD FOREIGN KEY (owner_id) REFERENCES users (user_id);
