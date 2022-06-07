-- Add up migration script here
CREATE TABLE IF NOT EXISTS users (
  user_id UUID PRIMARY KEY,
  username VARCHAR(30) NOT NULL,
  password VARCHAR(255) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS servers (
  server_id UUID PRIMARY KEY,
  name VARCHAR(30) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS channels (
  channel_id UUID PRIMARY KEY,
  name VARCHAR(30) NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  server_id UUID NOT NULL,
  FOREIGN KEY (server_id) REFERENCES servers (server_id)
);

CREATE TABLE IF NOT EXISTS messages (
  message_id UUID PRIMARY KEY,
  content TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  user_id UUID NOT NULL,
  channel_id UUID NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users (user_id),
  FOREIGN KEY (channel_id) REFERENCES channels (channel_id)
);

CREATE TABLE IF NOT EXISTS users_servers (
  user_id UUID NOT NULL,
  server_id UUID NOT NULL,
  PRIMARY KEY (user_id, server_id),
  FOREIGN KEY (user_id) REFERENCES users (user_id),
  FOREIGN KEY (server_id) REFERENCES servers (server_id)
);

CREATE TABLE IF NOT EXISTS sessions (
  session_id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users (user_id)
);