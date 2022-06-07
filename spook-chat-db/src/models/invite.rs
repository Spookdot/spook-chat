use super::Server;
use chrono::{DateTime, Utc};
use sqlx::{postgres::PgQueryResult, PgPool};
use uuid::Uuid;

pub struct Invite {
    pub invite_id: Uuid,
    pub server_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl Invite {
    pub fn new(server: &Server, expires: Option<DateTime<Utc>>) -> Self {
        Self {
            invite_id: Uuid::new_v4(),
            server_id: server.server_id,
            created_at: Utc::now(),
            expires_at: expires,
        }
    }

    pub async fn save(&self, pool: &PgPool) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            "INSERT INTO invites (invite_id, server_id, created_at, expires_at)
            VALUES ($1, $2, $3, $4)",
            self.invite_id,
            self.server_id,
            self.created_at,
            self.expires_at
        )
        .execute(pool)
        .await
    }

    /// Check if this invite has expired yet
    pub fn is_valid(&self) -> bool {
        if let Some(expires) = self.expires_at {
            expires > Utc::now()
        } else {
            true
        }
    }

    pub async fn filter_by_id(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Invite>> {
        sqlx::query_as!(Invite, "SELECT * FROM invites WHERE invite_id = $1", id)
            .fetch_optional(pool)
            .await
    }
}
