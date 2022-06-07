-- Add up migration script here
ALTER TABLE users 
ADD COLUMN email_address VARCHAR(255) NOT NULL UNIQUE;
