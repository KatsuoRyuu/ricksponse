use crate::entity::ricksponse::ricksponse_extract_fut::RicksponseExtractFut;
use crate::error::Error;
use crate::hateoas::{HateoasResponse, Status};
use actix_http::body::BoxBody;
use actix_web::{FromRequest, HttpRequest, HttpResponse, HttpResponseBuilder, Responder};
use http::StatusCode;
use serde::de::DeserializeOwned;
use serde::Serialize;
use simple_serde::{ContentType, SimpleEncoder};
use std::any::Any;
use std::fmt::{Debug, Display, Formatter};
use std::hint;
use std::marker::PhantomData;

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

    pub fn new_with_http(t: T, http: u16) -> Self {
        Self::Data {
            data: t,
            http_code: Some(http),
            message: None,
        }
    }

    pub fn new_with_http_and_message(t: T, http: u16, message: &str) -> Self {
        Self::Data {
            data: t,
            http_code: Some(http),
            message: Some(message.to_string()),
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

// impl<T: Serialize> From<HateoasResponse<T>> for Ricksponse<HateoasResponse<T>> {
//     fn from(r: HateoasResponse<T>) -> Self {
//         let http_code = r.status.as_ref().and_then(|status| status.http_status_code);
//         let message = r.status.as_ref().and_then(|status| status.message.clone());
//         Self::Data {
//             data: r,
//             http_code,
//             message,
//         }
//     }
// }

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

pub struct Response<T> {
    phantom: PhantomData<T>,
}

macro_rules! status_codes {
    (
        $(
            $(#[$docs:meta])*
            ($num:expr, $konst:ident, $phrase:expr);
        )+
    ) => {
        impl<T> Ricksponse<T> {
        $(
            $(#[$docs])*
            pub const fn $konst(t: T) -> Ricksponse<T> {
                Ricksponse::new_with_http_and_message(t, $num, $phrase)
            }
        )+
}
        impl<T> Response<T> {
        $(
            $(#[$docs])*
            pub const fn $konst(t: T) -> Ricksponse<T> {
                Ricksponse::new_with_http_and_message(t, $num, $phrase)
            }
        )+
        }

        fn canonical_reason(num: u16) -> Option<&'static str> {
            match num {
                $(
                $num => Some($phrase),
                )+
                _ => None
            }
        }
    }
}

status_codes! {
    /// 100 Continue
    /// [[RFC7231, Section 6.2.1](https://tools.ietf.org/html/rfc7231#section-6.2.1)]
    (100, CONTINUE, "Continue");
    /// 101 Switching Protocols
    /// [[RFC7231, Section 6.2.2](https://tools.ietf.org/html/rfc7231#section-6.2.2)]
    (101, SWITCHING_PROTOCOLS, "Switching Protocols");
    /// 102 Processing
    /// [[RFC2518](https://tools.ietf.org/html/rfc2518)]
    (102, PROCESSING, "Processing");

    /// 200 OK
    /// [[RFC7231, Section 6.3.1](https://tools.ietf.org/html/rfc7231#section-6.3.1)]
    (200, OK, "OK");
    /// 201 Created
    /// [[RFC7231, Section 6.3.2](https://tools.ietf.org/html/rfc7231#section-6.3.2)]
    (201, CREATED, "Created");
    /// 202 Accepted
    /// [[RFC7231, Section 6.3.3](https://tools.ietf.org/html/rfc7231#section-6.3.3)]
    (202, ACCEPTED, "Accepted");
    /// 203 Non-Authoritative Information
    /// [[RFC7231, Section 6.3.4](https://tools.ietf.org/html/rfc7231#section-6.3.4)]
    (203, NON_AUTHORITATIVE_INFORMATION, "Non Authoritative Information");
    /// 204 No Content
    /// [[RFC7231, Section 6.3.5](https://tools.ietf.org/html/rfc7231#section-6.3.5)]
    (204, NO_CONTENT, "No Content");
    /// 205 Reset Content
    /// [[RFC7231, Section 6.3.6](https://tools.ietf.org/html/rfc7231#section-6.3.6)]
    (205, RESET_CONTENT, "Reset Content");
    /// 206 Partial Content
    /// [[RFC7233, Section 4.1](https://tools.ietf.org/html/rfc7233#section-4.1)]
    (206, PARTIAL_CONTENT, "Partial Content");
    /// 207 Multi-Status
    /// [[RFC4918](https://tools.ietf.org/html/rfc4918)]
    (207, MULTI_STATUS, "Multi-Status");
    /// 208 Already Reported
    /// [[RFC5842](https://tools.ietf.org/html/rfc5842)]
    (208, ALREADY_REPORTED, "Already Reported");

    /// 226 IM Used
    /// [[RFC3229](https://tools.ietf.org/html/rfc3229)]
    (226, IM_USED, "IM Used");

    /// 300 Multiple Choices
    /// [[RFC7231, Section 6.4.1](https://tools.ietf.org/html/rfc7231#section-6.4.1)]
    (300, MULTIPLE_CHOICES, "Multiple Choices");
    /// 301 Moved Permanently
    /// [[RFC7231, Section 6.4.2](https://tools.ietf.org/html/rfc7231#section-6.4.2)]
    (301, MOVED_PERMANENTLY, "Moved Permanently");
    /// 302 Found
    /// [[RFC7231, Section 6.4.3](https://tools.ietf.org/html/rfc7231#section-6.4.3)]
    (302, FOUND, "Found");
    /// 303 See Other
    /// [[RFC7231, Section 6.4.4](https://tools.ietf.org/html/rfc7231#section-6.4.4)]
    (303, SEE_OTHER, "See Other");
    /// 304 Not Modified
    /// [[RFC7232, Section 4.1](https://tools.ietf.org/html/rfc7232#section-4.1)]
    (304, NOT_MODIFIED, "Not Modified");
    /// 305 Use Proxy
    /// [[RFC7231, Section 6.4.5](https://tools.ietf.org/html/rfc7231#section-6.4.5)]
    (305, USE_PROXY, "Use Proxy");
    /// 307 Temporary Redirect
    /// [[RFC7231, Section 6.4.7](https://tools.ietf.org/html/rfc7231#section-6.4.7)]
    (307, TEMPORARY_REDIRECT, "Temporary Redirect");
    /// 308 Permanent Redirect
    /// [[RFC7238](https://tools.ietf.org/html/rfc7238)]
    (308, PERMANENT_REDIRECT, "Permanent Redirect");

    /// 400 Bad Request
    /// [[RFC7231, Section 6.5.1](https://tools.ietf.org/html/rfc7231#section-6.5.1)]
    (400, BAD_REQUEST, "Bad Request");
    /// 401 Unauthorized
    /// [[RFC7235, Section 3.1](https://tools.ietf.org/html/rfc7235#section-3.1)]
    (401, UNAUTHORIZED, "Unauthorized");
    /// 402 Payment Required
    /// [[RFC7231, Section 6.5.2](https://tools.ietf.org/html/rfc7231#section-6.5.2)]
    (402, PAYMENT_REQUIRED, "Payment Required");
    /// 403 Forbidden
    /// [[RFC7231, Section 6.5.3](https://tools.ietf.org/html/rfc7231#section-6.5.3)]
    (403, FORBIDDEN, "Forbidden");
    /// 404 Not Found
    /// [[RFC7231, Section 6.5.4](https://tools.ietf.org/html/rfc7231#section-6.5.4)]
    (404, NOT_FOUND, "Not Found");
    /// 405 Method Not Allowed
    /// [[RFC7231, Section 6.5.5](https://tools.ietf.org/html/rfc7231#section-6.5.5)]
    (405, METHOD_NOT_ALLOWED, "Method Not Allowed");
    /// 406 Not Acceptable
    /// [[RFC7231, Section 6.5.6](https://tools.ietf.org/html/rfc7231#section-6.5.6)]
    (406, NOT_ACCEPTABLE, "Not Acceptable");
    /// 407 Proxy Authentication Required
    /// [[RFC7235, Section 3.2](https://tools.ietf.org/html/rfc7235#section-3.2)]
    (407, PROXY_AUTHENTICATION_REQUIRED, "Proxy Authentication Required");
    /// 408 Request Timeout
    /// [[RFC7231, Section 6.5.7](https://tools.ietf.org/html/rfc7231#section-6.5.7)]
    (408, REQUEST_TIMEOUT, "Request Timeout");
    /// 409 Conflict
    /// [[RFC7231, Section 6.5.8](https://tools.ietf.org/html/rfc7231#section-6.5.8)]
    (409, CONFLICT, "Conflict");
    /// 410 Gone
    /// [[RFC7231, Section 6.5.9](https://tools.ietf.org/html/rfc7231#section-6.5.9)]
    (410, GONE, "Gone");
    /// 411 Length Required
    /// [[RFC7231, Section 6.5.10](https://tools.ietf.org/html/rfc7231#section-6.5.10)]
    (411, LENGTH_REQUIRED, "Length Required");
    /// 412 Precondition Failed
    /// [[RFC7232, Section 4.2](https://tools.ietf.org/html/rfc7232#section-4.2)]
    (412, PRECONDITION_FAILED, "Precondition Failed");
    /// 413 Payload Too Large
    /// [[RFC7231, Section 6.5.11](https://tools.ietf.org/html/rfc7231#section-6.5.11)]
    (413, PAYLOAD_TOO_LARGE, "Payload Too Large");
    /// 414 URI Too Long
    /// [[RFC7231, Section 6.5.12](https://tools.ietf.org/html/rfc7231#section-6.5.12)]
    (414, URI_TOO_LONG, "URI Too Long");
    /// 415 Unsupported Media Type
    /// [[RFC7231, Section 6.5.13](https://tools.ietf.org/html/rfc7231#section-6.5.13)]
    (415, UNSUPPORTED_MEDIA_TYPE, "Unsupported Media Type");
    /// 416 Range Not Satisfiable
    /// [[RFC7233, Section 4.4](https://tools.ietf.org/html/rfc7233#section-4.4)]
    (416, RANGE_NOT_SATISFIABLE, "Range Not Satisfiable");
    /// 417 Expectation Failed
    /// [[RFC7231, Section 6.5.14](https://tools.ietf.org/html/rfc7231#section-6.5.14)]
    (417, EXPECTATION_FAILED, "Expectation Failed");
    /// 418 I'm a teapot
    /// [curiously not registered by IANA but [RFC2324](https://tools.ietf.org/html/rfc2324)]
    (418, IM_A_TEAPOT, "I'm a teapot");

    /// 421 Misdirected Request
    /// [RFC7540, Section 9.1.2](http://tools.ietf.org/html/rfc7540#section-9.1.2)
    (421, MISDIRECTED_REQUEST, "Misdirected Request");
    /// 422 Unprocessable Entity
    /// [[RFC4918](https://tools.ietf.org/html/rfc4918)]
    (422, UNPROCESSABLE_ENTITY, "Unprocessable Entity");
    /// 423 Locked
    /// [[RFC4918](https://tools.ietf.org/html/rfc4918)]
    (423, LOCKED, "Locked");
    /// 424 Failed Dependency
    /// [[RFC4918](https://tools.ietf.org/html/rfc4918)]
    (424, FAILED_DEPENDENCY, "Failed Dependency");

    /// 426 Upgrade Required
    /// [[RFC7231, Section 6.5.15](https://tools.ietf.org/html/rfc7231#section-6.5.15)]
    (426, UPGRADE_REQUIRED, "Upgrade Required");

    /// 428 Precondition Required
    /// [[RFC6585](https://tools.ietf.org/html/rfc6585)]
    (428, PRECONDITION_REQUIRED, "Precondition Required");
    /// 429 Too Many Requests
    /// [[RFC6585](https://tools.ietf.org/html/rfc6585)]
    (429, TOO_MANY_REQUESTS, "Too Many Requests");

    /// 431 Request Header Fields Too Large
    /// [[RFC6585](https://tools.ietf.org/html/rfc6585)]
    (431, REQUEST_HEADER_FIELDS_TOO_LARGE, "Request Header Fields Too Large");

    /// 451 Unavailable For Legal Reasons
    /// [[RFC7725](http://tools.ietf.org/html/rfc7725)]
    (451, UNAVAILABLE_FOR_LEGAL_REASONS, "Unavailable For Legal Reasons");

    /// 500 Internal Server Error
    /// [[RFC7231, Section 6.6.1](https://tools.ietf.org/html/rfc7231#section-6.6.1)]
    (500, INTERNAL_SERVER_ERROR, "Internal Server Error");
    /// 501 Not Implemented
    /// [[RFC7231, Section 6.6.2](https://tools.ietf.org/html/rfc7231#section-6.6.2)]
    (501, NOT_IMPLEMENTED, "Not Implemented");
    /// 502 Bad Gateway
    /// [[RFC7231, Section 6.6.3](https://tools.ietf.org/html/rfc7231#section-6.6.3)]
    (502, BAD_GATEWAY, "Bad Gateway");
    /// 503 Service Unavailable
    /// [[RFC7231, Section 6.6.4](https://tools.ietf.org/html/rfc7231#section-6.6.4)]
    (503, SERVICE_UNAVAILABLE, "Service Unavailable");
    /// 504 Gateway Timeout
    /// [[RFC7231, Section 6.6.5](https://tools.ietf.org/html/rfc7231#section-6.6.5)]
    (504, GATEWAY_TIMEOUT, "Gateway Timeout");
    /// 505 HTTP Version Not Supported
    /// [[RFC7231, Section 6.6.6](https://tools.ietf.org/html/rfc7231#section-6.6.6)]
    (505, HTTP_VERSION_NOT_SUPPORTED, "HTTP Version Not Supported");
    /// 506 Variant Also Negotiates
    /// [[RFC2295](https://tools.ietf.org/html/rfc2295)]
    (506, VARIANT_ALSO_NEGOTIATES, "Variant Also Negotiates");
    /// 507 Insufficient Storage
    /// [[RFC4918](https://tools.ietf.org/html/rfc4918)]
    (507, INSUFFICIENT_STORAGE, "Insufficient Storage");
    /// 508 Loop Detected
    /// [[RFC5842](https://tools.ietf.org/html/rfc5842)]
    (508, LOOP_DETECTED, "Loop Detected");

    /// 510 Not Extended
    /// [[RFC2774](https://tools.ietf.org/html/rfc2774)]
    (510, NOT_EXTENDED, "Not Extended");
    /// 511 Network Authentication Required
    /// [[RFC6585](https://tools.ietf.org/html/rfc6585)]
    (511, NETWORK_AUTHENTICATION_REQUIRED, "Network Authentication Required");
}

#[cfg(test)]
mod test {
    use crate::Response;
    use crate::Ricksponse;

    #[test]
    fn test_status_codes() {
        let ricksponse_continue = Ricksponse::CONTINUE(());
        let response_continue = Response::CONTINUE(());
    }
}
