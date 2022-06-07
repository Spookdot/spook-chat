use argon2::Argon2;
use lazy_static::lazy_static;

pub mod models;

lazy_static! {
    pub static ref ARGON2: Argon2<'static> = Argon2::default();
}
