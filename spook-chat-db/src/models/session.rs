use super::User;
use chrono::{DateTime, Utc};
use sqlx::postgres::PgPool;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow)]
pub struct Session {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl Session {
    pub fn new(user: &User) -> Self {
        Self {
            session_id: Uuid::new_v4(),
            user_id: user.user_id,
            created_at: Utc::now(),
        }
    }

    pub async fn filter_by_id(pool: &PgPool, session_id: Uuid) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Session,
            "SELECT * FROM sessions WHERE session_id = $1",
            session_id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn delete(&self, pool: &PgPool) -> sqlx::Result<()> {
        sqlx::query!(
            "DELETE FROM sessions WHERE session_id = $1",
            self.session_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}
