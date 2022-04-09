use crate::entity::response::{Response, ResponseStatus};
use crate::entity::ricksponse::ricksponse_body::RicksponseBody;
use crate::entity::ricksponse::ricksponse_extract_fut::RicksponseExtractFut;
use crate::error::Error;
use crate::MAX_SIZE;
use actix_http::body::BoxBody;
use actix_web::{FromRequest, HttpRequest, HttpResponse, Responder};
use http::header::CONTENT_LENGTH;
use railsgun::Merge;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use simple_serde::{ContentType, SimpleDecoder, SimpleEncoder};
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};
use uuid::Uuid;

pub struct Ricksponse<T> {
    inner: T,
}

impl<T> Ricksponse<T> {
    pub fn new(t: T) -> Self {
        Self { inner: t }
    }
    /// Deconstruct to an inner value
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> Deref for Ricksponse<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T> DerefMut for Ricksponse<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T> Debug for Ricksponse<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Json: {:?}", self.inner)
    }
}

impl<T> Display for Ricksponse<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

trait ToHateoasResponse<T> {
    fn to_hateoas_response(self) -> T;
}

impl<T: Serialize> ToHateoasResponse<Ricksponse<Response<T>>> for Ricksponse<T> {
    fn to_hateoas_response(mut self) -> Ricksponse<Response<T>> {
        Ricksponse::new(Response {
            content: Some(self.inner),
            status: ResponseStatus {
                message: None,
                code: None,
                http_status_code: None,
                session: None,
            },
        })
    }
}

trait HateoasResponse {
    fn set_status_message(&mut self, m: String) -> &mut Self;
    fn set_status_code(&mut self, m: u32) -> &mut Self;
    fn set_status_http_code(&mut self, m: u32) -> &mut Self;
    fn set_session(&mut self, m: uuid::Uuid) -> &mut Self;
}

impl<T: Serialize> HateoasResponse for Ricksponse<Response<T>> {
    fn set_status_message(&mut self, m: String) -> &mut Self {
        self.inner.status.message = Some(m);
        self
    }

    fn set_status_code(&mut self, c: u32) -> &mut Self {
        self.inner.status.code = Some(c);
        self
    }

    fn set_status_http_code(&mut self, h: u32) -> &mut Self {
        self.inner.status.http_status_code = Some(h);
        self
    }

    fn set_session(&mut self, u: Uuid) -> &mut Self {
        self.inner.status.session = Some(u);
        self
    }
}

trait ToResponse<R> {
    fn to_response(self) -> R;
}

impl<T: Serialize> ToResponse<Ricksponse<Option<T>>> for Ricksponse<Response<T>> {
    fn to_response(mut self) -> Ricksponse<Option<T>> {
        Ricksponse::new(self.inner.content)
    }
}

impl<T: Serialize> Responder for Ricksponse<T> {
    type Body = BoxBody;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        req.headers()
            .get_all("Accept")
            .filter_map(|h| ContentType::try_from(h).ok())
            .collect::<Vec<ContentType>>()
            .try_into()
            .map_err(Error::from)
            .and_then(|mut t: Vec<ContentType>| {
                t.reverse();
                t.pop()
                    .ok_or_else(|| Error::FailedToGetContentTypeFromHeader)
            })
            .and_then(|content_type| {
                self.inner
                    .encode(&content_type)
                    .map(|t| HttpResponse::Ok().content_type(content_type).body(t))
                    .map_err(Error::from)
            })
            .unwrap()
    }
}

/// ## Example
///
/// ```rust
/// use actix_web::{web, App};
/// use serde_derive::Deserialize;
/// use ricksponse::Ricksponse;
///
/// #[derive(Deserialize)]
/// struct Info {
///     username: String,
/// }
///
/// /// deserialize `Info` from request's body
/// async fn index(info: Ricksponse<Info>) -> String {
/// format!("Welcome {}!", info.username)
/// }
///
/// fn main() {
///     let app = App::new().service(
///         web::resource("/index.html").route(
///            web::post().to(index))
///     );
/// }
/// ```
///

impl<T> FromRequest for Ricksponse<T>
where
    T: DeserializeOwned,
{
    type Error = Error;
    type Future = RicksponseExtractFut<T>;

    #[inline]
    fn from_request(req: &HttpRequest, payload: &mut actix_http::Payload) -> Self::Future {
        RicksponseExtractFut::new(req.clone(), payload)
    }
}
