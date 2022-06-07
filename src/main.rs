use rocket::tokio::sync::broadcast::{channel, Sender};
use rocket_cors::CorsOptions;
use spook_chat_db::models::Channel;
use sqlx::{
    postgres::{PgPool, PgPoolOptions},
    types::Uuid,
};
use std::collections::HashMap;

mod auth;
mod chat;
mod guards;
mod servers;

pub fn quick_response<'a, S: Into<String>>(
    status: rocket::http::Status,
    respose_text: S,
) -> rocket::Response<'a> {
    let msg: String = respose_text.into();
    rocket::Response::build()
        .status(status)
        .sized_body(msg.len(), std::io::Cursor::new(msg))
        .finalize()
}

#[macro_use]
extern crate rocket;

struct MyState {
    conn: PgPool,
    channels: HashMap<Uuid, Sender<String>>,
}

#[launch]
async fn rocket() -> _ {
    dotenv::dotenv().ok();

    let cors = CorsOptions::default();

    let conn = PgPoolOptions::new()
        .max_connections(10)
        .connect(std::env::var("DATABASE_URL").unwrap().as_str())
        .await
        .unwrap();

    let mut channels = HashMap::new();
    let channel_ids = match Channel::fetch_all_ids(&conn).await {
        Ok(ids) => ids,
        Err(e) => {
            if let sqlx::Error::RowNotFound = e {
                vec![]
            } else {
                panic!("{}", e);
            }
        }
    };

    for channel_id in channel_ids {
        let (tx, _) = channel::<String>(15);
        channels.insert(channel_id, tx);
    }

    rocket::build()
        .mount("/auth", auth::routes())
        .mount("/chat", chat::routes())
        .mount("/server", servers::routes())
        .manage(MyState { conn, channels })
        .attach(cors.to_cors().unwrap())
}
