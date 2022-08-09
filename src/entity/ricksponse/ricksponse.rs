use crate::entity::response::{HateoasResponse, Metadata, Status};
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
use std::hint;
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

    pub fn get_or_with_data(&mut self, data: T) -> &mut T {
        match self {
            Ricksponse::Error {
                http_code, message, ..
            } => {
                *self = Ricksponse::Data {
                    data,
                    http_code: *http_code,
                    message: message.clone(),
                }
            }
            _ => {}
        }
        // SAFETY: a `None` variant for `self` would have been replaced by a `Some`
        // variant in the code above.
        match self {
            Ricksponse::Data { data, .. } => data,
            Ricksponse::Error { .. } => unsafe { hint::unreachable_unchecked() },
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

impl<T: Serialize> From<HateoasResponse<T>> for Ricksponse<HateoasResponse<T>> {
    fn from(r: HateoasResponse<T>) -> Self {
        let http_code = r.status.as_ref().and_then(|status| status.http_status_code);
        let message = r.status.as_ref().and_then(|status| status.message.clone());
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

impl<T: Serialize> ToHateoasResponse<Ricksponse<HateoasResponse<T>>> for Ricksponse<T> {
    fn to_hateoas_response(self) -> Ricksponse<HateoasResponse<T>> {
        match self {
            Ricksponse::Data {
                data,
                http_code,
                message,
            } => Ricksponse::new(HateoasResponse {
                api_version: "Unknown.invalid/0.0.0".to_string(),
                kind: "".to_string(),
                metadata: None,
                spec: None,
                status: Some(Status {
                    message,
                    code: None,
                    http_status_code: http_code,
                    session: None,
                }),
            }),
            Self::Error {
                error,
                http_code,
                message,
            } => Ricksponse::from(HateoasResponse {
                api_version: "Unknown.invalid/0.0.0".to_string(),
                kind: "".to_string(),
                metadata: None,
                spec: None,
                status: Some(Status {
                    message,
                    code: None,
                    http_status_code: http_code,
                    session: None,
                }),
            }),
        }
    }
}

pub trait AsHateoasResponse<T>
where
    T: Serialize,
{
    fn as_response(&mut self) -> &mut HateoasResponse<T>;
}

impl<T: Serialize> AsHateoasResponse<T> for Ricksponse<HateoasResponse<T>> {
    fn as_response(&mut self) -> &mut HateoasResponse<T> {
        self.get_or_with_data(HateoasResponse::default())
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
            }
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
            }
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
/// #[derive(Deserialize, Debug)]
/// struct Info {
///     pub username: String,
/// }
///
/// /// deserialize `Info` from request's body
/// async fn index(info: Ricksponse<Info>) -> String {
/// format!("Welcome {:?}!", info)
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
