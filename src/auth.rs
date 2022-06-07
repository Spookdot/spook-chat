use crate::{guards::LoginGuard, quick_response, MyState};
use rocket::{
    http::{Cookie, CookieJar, Status},
    response::Responder,
    serde::{json::Json, uuid::Uuid, Deserialize},
    FromForm, State,
};
use spook_chat_db::models::User;
use std::str::FromStr;

#[derive(FromForm, Deserialize)]
#[serde(crate = "rocket::serde")]
struct RegisterData<'a> {
    email: &'a str,
    username: &'a str,
    password: &'a str,
}

#[derive(FromForm, Deserialize)]
#[serde(crate = "rocket::serde")]
struct LoginData<'a> {
    email: &'a str,
    password: &'a str,
}

enum AuthErrors {
    UuidError(rocket::serde::uuid::Error),
    SqlxError(sqlx::Error),
    WrongEmail,
    WrongPassword,
}

impl<'r, 'o: 'r> Responder<'r, 'o> for AuthErrors {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        match self {
            Self::SqlxError(e) => Ok(quick_response(Status::InternalServerError, e.to_string())),
            Self::UuidError(e) => Ok(quick_response(Status::BadRequest, e.to_string())),
            Self::WrongEmail | Self::WrongPassword => Ok(quick_response(
                Status::BadRequest,
                "Wrong E-Mail or Password",
            )),
        }
    }
}

impl From<rocket::serde::uuid::Error> for AuthErrors {
    fn from(e: rocket::serde::uuid::Error) -> Self {
        Self::UuidError(e)
    }
}

impl From<sqlx::Error> for AuthErrors {
    fn from(e: sqlx::Error) -> Self {
        Self::SqlxError(e)
    }
}

#[get("/test")]
async fn test() -> Result<(Status, String), AuthErrors> {
    let ud = Uuid::from_str("f9e4c8e5-bad6-49b3-85ef-4a6d2c2edde4")?;
    Ok((Status::Ok, ud.to_string()))
}

#[get("/authenticated")]
async fn authenticated(login: LoginGuard) -> (Status, String) {
    (
        Status::Ok,
        format!("You are logged in as {}", login.user.username),
    )
}

#[get("/logout")]
async fn logout(state: &State<MyState>, user: LoginGuard) -> Result<(Status, String), AuthErrors> {
    user.session.delete(&state.conn).await?;
    return Ok((
        Status::Ok,
        "You've been successfully logged out".to_string(),
    ));
}

#[post("/register", data = "<register>")]
async fn register(
    state: &State<MyState>,
    register: Json<RegisterData<'_>>,
) -> Result<(Status, String), AuthErrors> {
    let user = User::new(register.email, register.username, register.password);
    user.save(&state.conn).await?;
    Ok((Status::Ok, format!("Created User: {}", user.username)))
}

#[post("/login", data = "<login>")]
async fn login(
    state: &State<MyState>,
    cookies: &CookieJar<'_>,
    login: Json<LoginData<'_>>,
) -> Result<(Status, String), AuthErrors> {
    if let Some(cookie) = cookies.get_private("session") {
        let session_id = Uuid::from_str(cookie.value())?;
        if let Some(user) = User::filter_by_session_id(&state.conn, session_id).await? {
            return Ok((
                Status::Ok,
                format!("You are already logged in as {}", user.username),
            ));
        }
    }

    if let Some(user) = User::filter_by_email(&state.conn, login.email).await? {
        if user.verify_password(login.password) {
            let session_id = user.new_session(&state.conn).await.unwrap();
            cookies.add_private(Cookie::build("session", session_id).finish());
            Ok((Status::Ok, format!("Logged in as {}", user.username)))
        } else {
            Err(AuthErrors::WrongPassword)
        }
    } else {
        Err(AuthErrors::WrongEmail)
    }
}

pub fn routes() -> Vec<rocket::Route> {
    routes![register, login, authenticated, logout, test]
}
