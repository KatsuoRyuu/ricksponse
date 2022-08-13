use crate::entity::payload_control::PayloadControl;
use crate::entity::payload_error::PayloadError;
use crate::entity::payload_future::PayloadFuture;
use crate::error::Error;
use crate::Ricksponse;
use actix_http::body::BoxBody;
use actix_web::{FromRequest, HttpRequest, HttpResponse, HttpResponseBuilder, Responder};
use hateoas_response::{Hateoas, HateoasResource, Status};
use http::StatusCode;
use serde::de::DeserializeOwned;
use serde::Serialize;
use simple_serde::{ContentType, SimpleEncoder};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub struct RickHateOas<T>
where
    T: Serialize + HateoasResource + DeserializeOwned,
{
    inner: Hateoas<T>,
}

impl<T: Serialize + HateoasResource + DeserializeOwned> RickHateOas<T> {
    pub fn into_inner(self) -> Hateoas<T> {
        self.inner
    }
}

impl<T: Serialize + HateoasResource + DeserializeOwned> Deref for RickHateOas<T> {
    type Target = Hateoas<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Serialize + HateoasResource + DeserializeOwned> DerefMut for RickHateOas<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct Response<T: Serialize + HateoasResource + DeserializeOwned> {
    phantom: PhantomData<T>,
}

macro_rules! auto_impl_hateoas_response_wrapper {
    (
        $(
            $(#[$docs:meta])*
            ($konst:ident);
        )+
    ) => {
        impl<T: Serialize + HateoasResource + Default + DeserializeOwned> RickHateOas<T> {
            $(
            #[allow(non_snake_case)]
            pub fn $konst(t: T) -> RickHateOas<T> {
                RickHateOas {
                    inner: Hateoas::$konst(t),
                }
            }
            )+
        }

        impl<T: Serialize + HateoasResource + Default + DeserializeOwned> Response<T> {
            $(
            #[allow(non_snake_case)]
            pub fn $konst(t: T) -> RickHateOas<T> {
                RickHateOas {
                    inner: Hateoas::$konst(t),
                }
            }
            )+
        }
    };
}

auto_impl_hateoas_response_wrapper! {
    /// 100 Continue
    /// [[RFC7231, Section 6.2.1](https://tools.ietf.org/html/rfc7231#section-6.2.1)]
    (CONTINUE);
    /// 101 Switching Protocols
    /// [[RFC7231, Section 6.2.2](https://tools.ietf.org/html/rfc7231#section-6.2.2)]
    (SWITCHING_PROTOCOLS);
    /// 102 Processing
    /// [[RFC2518](https://tools.ietf.org/html/rfc2518)]
    (PROCESSING);

    /// 200 OK
    /// [[RFC7231, Section 6.3.1](https://tools.ietf.org/html/rfc7231#section-6.3.1)]
    (OK);
    /// 201 Created
    /// [[RFC7231, Section 6.3.2](https://tools.ietf.org/html/rfc7231#section-6.3.2)]
    (CREATED);
    /// 202 Accepted
    /// [[RFC7231, Section 6.3.3](https://tools.ietf.org/html/rfc7231#section-6.3.3)]
    (ACCEPTED);
    /// 203 Non-Authoritative Information
    /// [[RFC7231, Section 6.3.4](https://tools.ietf.org/html/rfc7231#section-6.3.4)]
    (NON_AUTHORITATIVE_INFORMATION);
    /// 204 No Content
    /// [[RFC7231, Section 6.3.5](https://tools.ietf.org/html/rfc7231#section-6.3.5)]
    (NO_CONTENT);
    /// 205 Reset Content
    /// [[RFC7231, Section 6.3.6](https://tools.ietf.org/html/rfc7231#section-6.3.6)]
    (RESET_CONTENT);
    /// 206 Partial Content
    /// [[RFC7233, Section 4.1](https://tools.ietf.org/html/rfc7233#section-4.1)]
    (PARTIAL_CONTENT);
    /// 207 Multi-Status
    /// [[RFC4918](https://tools.ietf.org/html/rfc4918)]
    (MULTI_STATUS);
    /// 208 Already Reported
    /// [[RFC5842](https://tools.ietf.org/html/rfc5842)]
    (ALREADY_REPORTED);

    /// 226 IM Used
    /// [[RFC3229](https://tools.ietf.org/html/rfc3229)]
    (IM_USED);

    /// 300 Multiple Choices
    /// [[RFC7231, Section 6.4.1](https://tools.ietf.org/html/rfc7231#section-6.4.1)]
    (MULTIPLE_CHOICES);
    /// 301 Moved Permanently
    /// [[RFC7231, Section 6.4.2](https://tools.ietf.org/html/rfc7231#section-6.4.2)]
    (MOVED_PERMANENTLY);
    /// 302 Found
    /// [[RFC7231, Section 6.4.3](https://tools.ietf.org/html/rfc7231#section-6.4.3)]
    (FOUND);
    /// 303 See Other
    /// [[RFC7231, Section 6.4.4](https://tools.ietf.org/html/rfc7231#section-6.4.4)]
    (SEE_OTHER);
    /// 304 Not Modified
    /// [[RFC7232, Section 4.1](https://tools.ietf.org/html/rfc7232#section-4.1)]
    (NOT_MODIFIED);
    /// 305 Use Proxy
    /// [[RFC7231, Section 6.4.5](https://tools.ietf.org/html/rfc7231#section-6.4.5)]
    (USE_PROXY);
    /// 307 Temporary Redirect
    /// [[RFC7231, Section 6.4.7](https://tools.ietf.org/html/rfc7231#section-6.4.7)]
    (TEMPORARY_REDIRECT);
    /// 308 Permanent Redirect
    /// [[RFC7238](https://tools.ietf.org/html/rfc7238)]
    (PERMANENT_REDIRECT);

    /// 400 Bad Request
    /// [[RFC7231, Section 6.5.1](https://tools.ietf.org/html/rfc7231#section-6.5.1)]
    (BAD_REQUEST);
    /// 401 Unauthorized
    /// [[RFC7235, Section 3.1](https://tools.ietf.org/html/rfc7235#section-3.1)]
    (UNAUTHORIZED);
    /// 402 Payment Required
    /// [[RFC7231, Section 6.5.2](https://tools.ietf.org/html/rfc7231#section-6.5.2)]
    (PAYMENT_REQUIRED);
    /// 403 Forbidden
    /// [[RFC7231, Section 6.5.3](https://tools.ietf.org/html/rfc7231#section-6.5.3)]
    (FORBIDDEN);
    /// 404 Not Found
    /// [[RFC7231, Section 6.5.4](https://tools.ietf.org/html/rfc7231#section-6.5.4)]
    (NOT_FOUND);
    /// 405 Method Not Allowed
    /// [[RFC7231, Section 6.5.5](https://tools.ietf.org/html/rfc7231#section-6.5.5)]
    (METHOD_NOT_ALLOWED);
    /// 406 Not Acceptable
    /// [[RFC7231, Section 6.5.6](https://tools.ietf.org/html/rfc7231#section-6.5.6)]
    (NOT_ACCEPTABLE);
    /// 407 Proxy Authentication Required
    /// [[RFC7235, Section 3.2](https://tools.ietf.org/html/rfc7235#section-3.2)]
    (PROXY_AUTHENTICATION_REQUIRED);
    /// 408 Request Timeout
    /// [[RFC7231, Section 6.5.7](https://tools.ietf.org/html/rfc7231#section-6.5.7)]
    (REQUEST_TIMEOUT);
    /// 409 Conflict
    /// [[RFC7231, Section 6.5.8](https://tools.ietf.org/html/rfc7231#section-6.5.8)]
    (CONFLICT);
    /// 410 Gone
    /// [[RFC7231, Section 6.5.9](https://tools.ietf.org/html/rfc7231#section-6.5.9)]
    (GONE);
    /// 411 Length Required
    /// [[RFC7231, Section 6.5.10](https://tools.ietf.org/html/rfc7231#section-6.5.10)]
    (LENGTH_REQUIRED);
    /// 412 Precondition Failed
    /// [[RFC7232, Section 4.2](https://tools.ietf.org/html/rfc7232#section-4.2)]
    (PRECONDITION_FAILED);
    /// 413 Payload Too Large
    /// [[RFC7231, Section 6.5.11](https://tools.ietf.org/html/rfc7231#section-6.5.11)]
    (PAYLOAD_TOO_LARGE);
    /// 414 URI Too Long
    /// [[RFC7231, Section 6.5.12](https://tools.ietf.org/html/rfc7231#section-6.5.12)]
    (URI_TOO_LONG);
    /// 415 Unsupported Media Type
    /// [[RFC7231, Section 6.5.13](https://tools.ietf.org/html/rfc7231#section-6.5.13)]
    (UNSUPPORTED_MEDIA_TYPE);
    /// 416 Range Not Satisfiable
    /// [[RFC7233, Section 4.4](https://tools.ietf.org/html/rfc7233#section-4.4)]
    (RANGE_NOT_SATISFIABLE);
    /// 417 Expectation Failed
    /// [[RFC7231, Section 6.5.14](https://tools.ietf.org/html/rfc7231#section-6.5.14)]
    (EXPECTATION_FAILED);
    /// 418 I'm a teapot
    /// [curiously not registered by IANA but [RFC2324](https://tools.ietf.org/html/rfc2324)]
    (IM_A_TEAPOT);

    /// 421 Misdirected Request
    /// [RFC7540, Section 9.1.2](http://tools.ietf.org/html/rfc7540#section-9.1.2)
    (MISDIRECTED_REQUEST);
    /// 422 Unprocessable Entity
    /// [[RFC4918](https://tools.ietf.org/html/rfc4918)]
    (UNPROCESSABLE_ENTITY);
    /// 423 Locked
    /// [[RFC4918](https://tools.ietf.org/html/rfc4918)]
    (LOCKED);
    /// 424 Failed Dependency
    /// [[RFC4918](https://tools.ietf.org/html/rfc4918)]
    (FAILED_DEPENDENCY);

    /// 426 Upgrade Required
    /// [[RFC7231, Section 6.5.15](https://tools.ietf.org/html/rfc7231#section-6.5.15)]
    (UPGRADE_REQUIRED);

    /// 428 Precondition Required
    /// [[RFC6585](https://tools.ietf.org/html/rfc6585)]
    (PRECONDITION_REQUIRED);
    /// 429 Too Many Requests
    /// [[RFC6585](https://tools.ietf.org/html/rfc6585)]
    (TOO_MANY_REQUESTS);

    /// 431 Request Header Fields Too Large
    /// [[RFC6585](https://tools.ietf.org/html/rfc6585)]
    (REQUEST_HEADER_FIELDS_TOO_LARGE);

    /// 451 Unavailable For Legal Reasons
    /// [[RFC7725](http://tools.ietf.org/html/rfc7725)]
    (UNAVAILABLE_FOR_LEGAL_REASONS);

    /// 500 Internal Server Error
    /// [[RFC7231, Section 6.6.1](https://tools.ietf.org/html/rfc7231#section-6.6.1)]
    (INTERNAL_SERVER_ERROR);
    /// 501 Not Implemented
    /// [[RFC7231, Section 6.6.2](https://tools.ietf.org/html/rfc7231#section-6.6.2)]
    (NOT_IMPLEMENTED);
    /// 502 Bad Gateway
    /// [[RFC7231, Section 6.6.3](https://tools.ietf.org/html/rfc7231#section-6.6.3)]
    (BAD_GATEWAY);
    /// 503 Service Unavailable
    /// [[RFC7231, Section 6.6.4](https://tools.ietf.org/html/rfc7231#section-6.6.4)]
    (SERVICE_UNAVAILABLE);
    /// 504 Gateway Timeout
    /// [[RFC7231, Section 6.6.5](https://tools.ietf.org/html/rfc7231#section-6.6.5)]
    (GATEWAY_TIMEOUT);
    /// 505 HTTP Version Not Supported
    /// [[RFC7231, Section 6.6.6](https://tools.ietf.org/html/rfc7231#section-6.6.6)]
    (HTTP_VERSION_NOT_SUPPORTED);
    /// 506 Variant Also Negotiates
    /// [[RFC2295](https://tools.ietf.org/html/rfc2295)]
    (VARIANT_ALSO_NEGOTIATES);
    /// 507 Insufficient Storage
    /// [[RFC4918](https://tools.ietf.org/html/rfc4918)]
    (INSUFFICIENT_STORAGE);
    /// 508 Loop Detected
    /// [[RFC5842](https://tools.ietf.org/html/rfc5842)]
    (LOOP_DETECTED);

    /// 510 Not Extended
    /// [[RFC2774](https://tools.ietf.org/html/rfc2774)]
    (NOT_EXTENDED);
    /// 511 Network Authentication Required
    /// [[RFC6585](https://tools.ietf.org/html/rfc6585)]
    (NETWORK_AUTHENTICATION_REQUIRED);
}

impl<T: Serialize + HateoasResource + DeserializeOwned + Default> From<RickHateOas<T>>
    for Ricksponse<Hateoas<T>>
{
    fn from(r: RickHateOas<T>) -> Self {
        Ricksponse::Data {
            http_code: r
                .status()
                .as_ref()
                .and_then(|t| t.http_status_code().clone()),
            message: r.status().as_ref().and_then(|t| t.message().clone()),
            data: r.inner,
        }
    }
}

impl<T: Serialize + HateoasResource + DeserializeOwned + Default> Responder for RickHateOas<T> {
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

        let ricksponse: Ricksponse<Hateoas<T>> = self.into();
        match ricksponse {
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

impl<T: Serialize + HateoasResource + DeserializeOwned + Default>
    From<Result<Hateoas<T>, PayloadError>> for RickHateOas<T>
{
    fn from(res: Result<Hateoas<T>, PayloadError>) -> Self {
        match res {
            Ok(inner) => RickHateOas { inner },
            Err(e) => {
                let mut status = Status::INTERNAL_SERVER_ERROR();
                *status.message_mut() = Some(format!("{:?}", e));
                let inner: Hateoas<T> = Hateoas::new(None, None, Some(status));
                RickHateOas { inner }
            }
        }
    }
}
impl<T> FromRequest for RickHateOas<T>
where
    T: Serialize + DeserializeOwned + HateoasResource + PayloadControl + Default,
{
    type Error = Error;
    type Future = PayloadFuture<T, Hateoas<T>, RickHateOas<T>>;

    #[inline]
    fn from_request(req: &HttpRequest, payload: &mut actix_http::Payload) -> Self::Future {
        PayloadFuture::new(req.clone(), payload)
    }
}
