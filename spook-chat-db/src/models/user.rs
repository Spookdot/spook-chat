use super::{channel::Channel, server::Server, Session};
use crate::ARGON2;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    PasswordHash, PasswordHasher, PasswordVerifier,
};
use chrono::{DateTime, Utc};
use sqlx::{postgres::PgQueryResult, FromRow, PgPool};
use uuid::Uuid;

#[derive(FromRow)]
pub struct User {
    pub user_id: Uuid,
    pub email_address: String,
    pub username: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(email_address: &str, username: &str, password: &str) -> Self {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = ARGON2
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        Self {
            user_id: Uuid::new_v4(),
            email_address: email_address.to_string(),
            username: username.to_string(),
            password: password_hash,
            created_at: Utc::now(),
        }
    }

    pub async fn save(&self, pool: &PgPool) -> sqlx::Result<()> {
        sqlx::query!(
            "INSERT INTO users (user_id, email_address, username, password, created_at)
            VALUES ($1, $2, $3, $4, $5)",
            self.user_id,
            self.email_address,
            self.username,
            self.password,
            self.created_at
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn servers(&self, pool: &PgPool) -> sqlx::Result<Vec<Server>> {
        sqlx::query_as!(
            Server,
            "SELECT A.* FROM servers A WHERE A.server_id IN (
                SELECT B.server_id FROM users_servers B WHERE B.user_id = $1
            )",
            self.user_id
        )
        .fetch_all(pool)
        .await
    }

    pub async fn channels(&self, pool: &PgPool) -> sqlx::Result<Vec<Channel>> {
        sqlx::query_as!(
            Channel,
            "SELECT A.* FROM channels A WHERE A.server_id IN (
                SELECT B.server_id FROM users_servers B WHERE B.user_id = $1
            )",
            self.user_id
        )
        .fetch_all(pool)
        .await
    }

    pub fn verify_password(&self, password: &str) -> bool {
        let hash = PasswordHash::new(self.password.as_str()).unwrap();
        ARGON2.verify_password(password.as_bytes(), &hash).is_ok()
    }

    pub async fn new_session(&self, pool: &PgPool) -> sqlx::Result<String> {
        let session = Session::new(self);
        sqlx::query!(
            "INSERT INTO sessions (session_id, user_id, created_at)
            VALUES ($1, $2, $3)",
            session.session_id,
            session.user_id,
            session.created_at
        )
        .execute(pool)
        .await?;

        Ok(session.session_id.to_string())
    }

    pub async fn add_to_server(
        &self,
        pool: &PgPool,
        server_id: Uuid,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            "INSERT INTO users_servers (user_id, server_id)
            VALUES ($1, $2)",
            self.user_id,
            server_id
        )
        .execute(pool)
        .await
    }

    pub async fn has_access_to_channel(
        &self,
        pool: &PgPool,
        channel_id: Uuid,
    ) -> sqlx::Result<bool> {
        let channel = sqlx::query_as!(
            Channel,
            "SELECT A.* FROM channels A WHERE A.server_id IN (
                SELECT B.server_id FROM users_servers B WHERE B.user_id = $1
            ) AND A.channel_id = $2",
            self.user_id,
            channel_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(channel.is_some())
    }

    pub async fn has_access_to_server(&self, pool: &PgPool, server_id: Uuid) -> sqlx::Result<bool> {
        let server = sqlx::query_as!(
            Server,
            "SELECT A.* FROM servers A WHERE A.server_id IN (
                SELECT B.server_id FROM users_servers B WHERE B.user_id = $1
            ) AND A.server_id = $2",
            self.user_id,
            server_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(server.is_some())
    }

    pub async fn filter_by_id(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<User>> {
        let user = sqlx::query_as!(User, "SELECT * FROM users WHERE user_id = $1", id)
            .fetch_optional(pool)
            .await;

        user
    }

    pub async fn filter_by_email(pool: &PgPool, email_address: &str) -> sqlx::Result<Option<Self>> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email_address = $1",
            email_address
        )
        .fetch_optional(pool)
        .await;

        user
    }

    pub async fn filter_by_session_id(
        pool: &PgPool,
        session_id: Uuid,
    ) -> sqlx::Result<Option<Self>> {
        let users = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE user_id IN (
                SELECT user_id FROM sessions WHERE session_id = $1
            )",
            session_id
        )
        .fetch_optional(pool)
        .await;

        users
    }

    pub async fn filter_by_username(pool: &PgPool, username: &str) -> sqlx::Result<Vec<Self>> {
        let user = sqlx::query_as!(User, "SELECT * FROM users WHERE username = $1", username)
            .fetch_all(pool)
            .await;

        user
    }
}
