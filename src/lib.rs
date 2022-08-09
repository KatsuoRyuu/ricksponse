extern crate actix_http;
extern crate actix_web;
extern crate futures;
extern crate futures_core;
extern crate railsgun;
extern crate serde;
extern crate serde_derive;
extern crate simple_serde;

mod entity;
mod error;
mod helpers;

pub type Result<T> = std::result::Result<T, error::Error>;

const MAX_SIZE: usize = 41_943_040;

pub use entity::ricksponse::payload_error::RicksponsePayloadError;
pub use entity::{
    response,
    ricksponse::ricksponse::{DebuggableAny, Ricksponse, Ricksponse, ToHateoasResponse},
};

pub use error::Error;
pub mod helpers_prelude {
    pub use crate::helpers::get_from_hash_set;
}
