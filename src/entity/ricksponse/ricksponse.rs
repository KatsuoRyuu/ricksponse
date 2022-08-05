use crate::entity::response::{Response, ResponseStatus};
use crate::entity::ricksponse::ricksponse_extract_fut::RicksponseExtractFut;
use crate::error::Error;
use actix_http::body::BoxBody;
use actix_web::{FromRequest, HttpRequest, HttpResponse, Responder};
use http::StatusCode;
use railsgun::Merge;
use serde::de::DeserializeOwned;
use serde::Serialize;
use simple_serde::{ContentType, SimpleEncoder};
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};
use uuid::Uuid;

pub struct Ricksponse<T, E> {
    inner: Result<T, E>,
    http_code: Option<u16>,
    message: Option<String>,
}

impl<T, E> Ricksponse<T, E> {
    pub fn http_code(&mut self, code: u16) {
        self.http_code = Some(code);
    }
    pub fn message(&mut self, message: String) {
        self.message = Some(message);
    }
}

impl<T, E> Ricksponse<T, E> {
    pub fn new(t: T) -> Self {
        Self {
            inner: Ok(t),
            http_code: None,
            message: None,
        }
    }
}

impl<T, E: Display> From<Result<T, E>> for Ricksponse<T, E> {
    fn from(r: Result<T, E>) -> Self {
        let message = if let Err(e) = &r {
            Some(e.to_string())
        } else {
            None
        };
        Self {
            inner: r,
            http_code: None,
            message,
        }
    }
}

impl<T: Serialize> From<Response<T>> for Ricksponse<Response<T>, ()> {
    fn from(r: Response<T>) -> Self {
        let http_code = r.status.http_status_code;
        let message = r.status.message.clone();
        Self {
            inner: Ok(r),
            http_code,
            message,
        }
    }
}

impl<T, E> Deref for Ricksponse<T, E> {
    type Target = Result<T, E>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, E> DerefMut for Ricksponse<T, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T, E> Debug for Ricksponse<T, E>
where
    T: Debug,
    E: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Json: {:?}", self.inner)
    }
}

impl<T, E> Display for Ricksponse<T, E>
where
    T: Display,
    E: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            Err(e) => Display::fmt(e, f),
            Ok(t) => Display::fmt(t, f),
        }
    }
}

pub trait ToHateoasResponse<T> {
    fn to_hateoas_response(self) -> T;
}

impl<T: Serialize, E: Debug> ToHateoasResponse<Ricksponse<Response<T>, ()>> for Ricksponse<T, E> {
    fn to_hateoas_response(self) -> Ricksponse<Response<T>, ()> {
        match self.inner {
            Ok(t) => Ricksponse::new(Response {
                content: Some(t),
                status: ResponseStatus {
                    message: None,
                    code: None,
                    http_status_code: None,
                    session: None,
                },
            }),
            Err(e) => Ricksponse::from(Response {
                content: None,
                status: ResponseStatus {
                    message: Some(format!("{:?}", e)),
                    code: None,
                    http_status_code: None,
                    session: None,
                },
            }),
        }
    }
}

pub trait HateoasResponse {
    fn set_status_message(&mut self, m: String) -> &mut Self;
    fn set_status_code(&mut self, m: u32) -> &mut Self;
    fn set_status_http_code(&mut self, m: u16) -> &mut Self;
    fn set_session(&mut self, m: uuid::Uuid) -> &mut Self;
}

impl<T: Serialize> HateoasResponse for Ricksponse<Response<T>, ()> {
    fn set_status_message(&mut self, m: String) -> &mut Self {
        if let Ok(response) = &mut self.inner {
            response.status.message = Some(m);
        }
        self
    }

    fn set_status_code(&mut self, c: u32) -> &mut Self {
        if let Ok(response) = &mut self.inner {
            response.status.code = Some(c);
        }
        self
    }

    fn set_status_http_code(&mut self, h: u16) -> &mut Self {
        if let Ok(response) = &mut self.inner {
            response.status.http_status_code = Some(h);
        }
        self
    }

    fn set_session(&mut self, u: Uuid) -> &mut Self {
        if let Ok(response) = &mut self.inner {
            response.status.session = Some(u);
        }
        self
    }
}

pub trait ToResponse<R> {
    fn to_response(self) -> R;
}

impl<T: Serialize> ToResponse<Ricksponse<Option<T>, ()>> for Ricksponse<Response<T>, ()> {
    fn to_response(self) -> Ricksponse<Option<T>, ()> {
        match self.inner {
            Err(_) => Ricksponse::new(None),
            Ok(t) => Ricksponse::new(t.content),
        }
    }
}

impl<T: Serialize, E> Responder for Ricksponse<T, E> {
    type Body = BoxBody;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        let mut content_type_collection = req
            .headers()
            .get_all("Accept")
            .filter_map(|h| ContentType::try_from(h).ok())
            .collect::<Vec<ContentType>>();
        if content_type_collection.is_empty() {
            content_type_collection = vec![ContentType::Json];
        }
        let data_res = self.inner.map_err(|_| {
            if let Some(code) = self.http_code {
                let status_code =
                    StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
                HttpResponse::new(status_code)
            } else {
                HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
            }
        });
        content_type_collection.reverse();
        content_type_collection
            .pop()
            .ok_or_else(|| HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR))
            .merge(data_res, |content_type, data| {
                data.encode(&content_type)
                    .map(|t| {
                        HttpResponse::Ok()
                            .content_type(content_type)
                            .body(t.to_vec())
                    })
                    .map_err(|_| HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR))
            })
            .unwrap_or_else(|e| e)
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
/// async fn index(info: Ricksponse<Info, ()>) -> String {
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
impl<T, E> FromRequest for Ricksponse<T, E>
where
    T: DeserializeOwned,
    E: DeserializeOwned + Display,
{
    type Error = Error;
    type Future = RicksponseExtractFut<T, E>;

    #[inline]
    fn from_request(req: &HttpRequest, payload: &mut actix_http::Payload) -> Self::Future {
        RicksponseExtractFut::new(req.clone(), payload)
    }
}
