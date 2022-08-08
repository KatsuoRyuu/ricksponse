use crate::entity::response::{Response, ResponseStatus};
use crate::entity::ricksponse::ricksponse_extract_fut::RicksponseExtractFut;
use crate::error::Error;
use actix_http::body::BoxBody;
use actix_web::{FromRequest, HttpRequest, HttpResponse, HttpResponseBuilder, Responder};
use http::StatusCode;
use railsgun::Merge;
use serde::de::DeserializeOwned;
use serde::Serialize;
use simple_serde::{ContentType, SimpleEncoder};
use std::any::Any;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};
use uuid::Uuid;

pub trait DebuggableAny: Debug + Any {}

pub enum Ricksponse<T> {
    Data {
        data: T,
        http_code: Option<u16>,
        message: Option<String>,
    },
    Error {
        error: Option<Box<dyn DebuggableAny>>,
        http_code: Option<u16>,
        message: Option<String>,
    },
}

impl<T> Ricksponse<T> {
    pub fn http_code(&mut self, code: u16) {
        match self {
            Ricksponse::Data { http_code, .. } => *http_code = Some(code),
            Ricksponse::Error { http_code, .. } => *http_code = Some(code),
        }
    }
    pub fn message(&mut self, msg: String) {
        match self {
            Ricksponse::Data { message, .. } => *message = Some(msg),
            Ricksponse::Error { message, .. } => *message = Some(msg),
        }
    }
}

impl<T> Ricksponse<T> {
    pub fn new(t: T) -> Self {
        Self::Data {
            data: t,
            http_code: None,
            message: None,
        }
    }
}

impl<T, E: DebuggableAny> From<Result<T, E>> for Ricksponse<T> {
    fn from(r: Result<T, E>) -> Self {
        let message = if let Err(e) = &r {
            Some(format!("{:?}", e))
        } else {
            None
        };
        match r {
            Err(e) => Self::Error {
                error: Some(Box::new(e)),
                http_code: None,
                message,
            },
            Ok(t) => Self::Data {
                data: t,
                http_code: None,
                message,
            },
        }
    }
}

impl<T: Serialize> From<Response<T>> for Ricksponse<Response<T>> {
    fn from(r: Response<T>) -> Self {
        let http_code = r.metadata.http_status_code;
        let message = r.metadata.message.clone();
        Self::Data {
            data: r,
            http_code,
            message,
        }
    }
}

impl<T> From<Ricksponse<T>> for Result<T, Option<Box<dyn DebuggableAny>>> {
    fn from(r: Ricksponse<T>) -> Self {
        match r {
            Ricksponse::Data { data, .. } => Ok(data),
            Ricksponse::Error { error, .. } => Err(error),
        }
    }
}

// impl<T> Deref for Ricksponse<T> {
//     type Target = Result<T>;
//
//     fn deref(&self) -> &Self::Target {
//         &self
//     }
// }
//
// impl<T> DerefMut for Ricksponse<T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.inner
//     }
// }

impl<T> Debug for Ricksponse<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Json: {:?}", self)
    }
}

impl<T> Display for Ricksponse<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Ricksponse::Error { error, .. } => Display::fmt(&format!("{:?}", error), f),
            Ricksponse::Data { data, .. } => Display::fmt(&format!("{}", data), f),
        }
    }
}

pub trait ToHateoasResponse<T> {
    fn to_hateoas_response(self) -> T;
}

impl<T: Serialize> ToHateoasResponse<Ricksponse<Response<T>>> for Ricksponse<T> {
    fn to_hateoas_response(self) -> Ricksponse<Response<T>> {
        match self {
            Ricksponse::Data {
                data,
                http_code,
                message,
            } => Ricksponse::new(Response {
                content: Some(data),
                metadata: ResponseStatus {
                    message,
                    code: None,
                    http_status_code: http_code,
                    session: None,
                },
            }),
            Self::Error {
                error,
                http_code,
                message,
            } => Ricksponse::from(Response {
                content: None,
                metadata: ResponseStatus {
                    message: message.or_else(|| error.map(|e| format!("{:?}", e))),
                    code: None,
                    http_status_code: http_code,
                    session: None,
                },
            }),
        }
    }
}

pub trait HateoasResponse {
    fn message(&mut self, m: String) -> &mut Self;
    fn status_code(&mut self, m: u32) -> &mut Self;
    fn http_code(&mut self, m: u16) -> &mut Self;
    fn session(&mut self, m: uuid::Uuid) -> &mut Self;
}

impl<T: Serialize> HateoasResponse for Ricksponse<Response<T>> {
    fn message(&mut self, m: String) -> &mut Self {
        match self {
            Self::Data { data, .. } => data.metadata.message = Some(m),
            _ => {}
        }
        self
    }

    fn status_code(&mut self, c: u32) -> &mut Self {
        match self {
            Self::Data { data, .. } => data.metadata.code = Some(c),
            _ => {}
        }
        self
    }

    fn http_code(&mut self, h: u16) -> &mut Self {
        match self {
            Self::Data { data, .. } => data.metadata.http_status_code = Some(h),
            _ => {}
        }
        self
    }

    fn session(&mut self, u: Uuid) -> &mut Self {
        match self {
            Self::Data { data, .. } => data.metadata.session = Some(u),
            _ => {}
        }
        self
    }
}

// pub trait ToResponse<R> {
//     fn to_response(self) -> R;
// }
//
// impl<T: Serialize> ToResponse<Ricksponse<Option<T>>> for Ricksponse<Response<T>> {
//     fn to_response(self) -> Ricksponse<Option<T>> {
//         match self {
//             None => Ricksponse::new(None),
//             Some(t) => Ricksponse::new(t.content),
//         }
//     }
// }

impl<T> From<Ricksponse<T>> for HttpResponseBuilder {
    fn from(r: Ricksponse<T>) -> Self {
        match r {
            Ricksponse::Data {
                data, http_code, ..
            } => {
                let response_code = match http_code {
                    Some(code) => StatusCode::from_u16(code).unwrap_or(StatusCode::OK),
                    None => StatusCode::OK,
                };
                HttpResponseBuilder::new(response_code)
            },
            Ricksponse::Error { http_code, .. } => match http_code {
                Some(code) => HttpResponseBuilder::new(
                    StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                ),
                None => HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR),
            },
        }
    }
}

impl<T: Serialize> Responder for Ricksponse<T> {
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
        match self {
            Ricksponse::Data {
                data, http_code, ..
            } => {
                let response_code = match http_code {
                    Some(code) => StatusCode::from_u16(code).unwrap_or(StatusCode::OK),
                    None => StatusCode::OK,
                };
                content_type_collection.reverse();
                content_type_collection
                    .pop()
                    .ok_or_else(|| HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR))
                    .and_then(|content_type| {
                        data.encode(&content_type)
                            .map(|t| {
                               HttpResponseBuilder::new(response_code)
                                    .content_type(content_type)
                                    .body(t.to_vec())
                            })
                            .map_err(|_| HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR))
                    })
                    .unwrap_or_else(|e| e)
            },
            Ricksponse::Error { http_code, .. } => match http_code {
                Some(code) => HttpResponse::new(
                    StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                ),
                None => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
            },
        }

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
/// format!("Welcome {}!", info.unwrap().username)
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
