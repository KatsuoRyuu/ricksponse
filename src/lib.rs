extern crate actix_http;
extern crate actix_web;
extern crate futures;
extern crate futures_core;
#[cfg(feature = "hateoas")]
extern crate hateoas as hateoas_response;
extern crate railsgun;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate simple_serde;

#[cfg(test)]
extern crate serde_json;

mod entity;
mod error;
mod helpers;

pub type Result<T> = std::result::Result<T, error::Error>;

const MAX_SIZE: usize = 41_943_040;

pub use entity::{payload_control::*, payload_error::*, ricksponse::ricksponse::*};

pub use error::Error;
pub mod helpers_prelude {
    pub use crate::helpers::get_from_hash_set;
}

pub mod hateoas {
    pub use crate::entity::hateoas::*;
    pub mod prelude {
        pub use hateoas_response::*;
    }
}

#[cfg(test)]
mod test {
    use crate::Ricksponse;
    use actix_web::{http::header, test, web, App};

    #[derive(Serialize, Deserialize)]
    pub struct RubberBullet {
        pub name: String,
        pub title: String,
        pub chapter: String,
    }

    impl Default for RubberBullet {
        fn default() -> Self {
            RubberBullet {
                name: "Rubber Bullet".to_string(),
                title: "The Bullet".to_string(),
                chapter: "A Rubber Bullet Hurts".to_string(),
            }
        }
    }

    const RICKSPONSE_1: &str = r##"


    "##;

    #[actix_web::test]
    async fn test_ricksponse_string() {
        let app = test::init_service(
            App::new().service(
                web::resource("/index.html")
                    .route(web::post().to(|| async { Ricksponse::new("welcome!".to_string()) })),
            ),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/index.html")
            .insert_header(header::ContentType::json())
            .to_request();

        let res = test::call_service(&app, req).await;
        let result = test::read_body(res).await;

        let string =
            serde_json::from_str::<String>(std::str::from_utf8(&*result).unwrap()).unwrap();
        println!("{}", string);
        assert_eq!(string, "welcome!");
    }
}
