use std::str::FromStr;

use crate::MyState;
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    serde::uuid::Uuid,
    Request, State,
};
use spook_chat_db::models::{Session, User};

pub struct LoginGuard {
    pub user: User,
    pub session: Session,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for LoginGuard {
    type Error = sqlx::Error;

    async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        let conn = &request.guard::<&State<MyState>>().await.unwrap().conn;

        if let Some(session_cookie) = request.cookies().get_private("session") {
            let session_id = Uuid::from_str(session_cookie.value()).unwrap();

            let user = match User::filter_by_session_id(&conn, session_id).await {
                Ok(user) => user,
                Err(e) => return Outcome::Failure((Status::InternalServerError, e)),
            };

            let session = match Session::filter_by_id(&conn, session_id).await {
                Ok(session) => session,
                Err(e) => return Outcome::Failure((Status::InternalServerError, e)),
            };

            if let Some(user) = user {
                if let Some(session) = session {
                    return Outcome::Success(LoginGuard { user, session });
                }
            }
        }
        Outcome::Failure((Status::Unauthorized, sqlx::Error::RowNotFound))
    }
}
