use crate::entity::ricksponse::payload_error::RicksponsePayloadError;
use crate::error::Error;
use crate::MAX_SIZE;
use actix_http::Payload;
use actix_web::HttpRequest;
use bytes::BytesMut;
use futures_core::Stream as _;
use http::header::CONTENT_LENGTH;
use serde::de::DeserializeOwned;
use simple_serde::{ContentType, SimpleDecoder};
use std::future::Future;
use std::io::ErrorKind;
use std::marker::PhantomData;
use std::ops::Deref;
use std::pin::Pin;
use std::task::{ready, Context, Poll};
use std::{ops, sync::Arc};

const DEFAULT_LIMIT: usize = 41_943_040; // 40 mb

pub enum RicksponseBody<T> {
    Error(Option<RicksponsePayloadError>),
    Body {
        limit: usize,
        /// Length as reported by `Content-Length` header, if present.
        length: Option<usize>,
        content_type: ContentType,
        payload: Payload,
        buf: BytesMut,
        _res: PhantomData<T>,
    },
}

impl<T> Unpin for RicksponseBody<T> {}

impl<T: DeserializeOwned> RicksponseBody<T> {
    /// Create a new future to decode a JSON request payload.
    #[allow(clippy::borrow_interior_mutable_const)]
    pub fn new(r: HttpRequest, payload: &mut Payload) -> Self {
        let length = r
            .headers()
            .get(&CONTENT_LENGTH)
            .ok_or(Error::NoPayloadSizeDefinitionInHeader)
            .and_then(|l| l.to_str().map_err(Error::from))
            .and_then(|s| s.parse::<usize>().map_err(Error::from));
        let content_type = r
            .headers()
            .get_all("Content-Type")
            .filter_map(|h| simple_serde::ContentType::try_from(h).ok())
            .collect::<Vec<ContentType>>()
            .try_into()
            .map_err(Error::from)
            .and_then(|mut t: Vec<ContentType>| {
                t.reverse();
                t.pop()
                    .ok_or_else(|| Error::FailedToGetContentTypeFromHeader)
            });

        let payload = payload.take();

        match (content_type, length) {
            (Ok(c), Ok(l)) => RicksponseBody::Body {
                limit: DEFAULT_LIMIT,
                content_type: c,
                length: Some(l),
                payload,
                buf: BytesMut::with_capacity(8192),
                _res: PhantomData,
            },
            (Ok(c), _) => RicksponseBody::Body {
                limit: DEFAULT_LIMIT,
                content_type: c,
                length: None,
                payload,
                buf: BytesMut::with_capacity(8192),
                _res: PhantomData,
            },
            (_, _) => RicksponseBody::Error(Some(RicksponsePayloadError::ContentType)),
        }
    }

    /// Set maximum accepted payload size. The default limit is 2MB.
    pub fn limit(self, limit: usize) -> Self {
        match self {
            RicksponseBody::Body {
                length,
                content_type,
                payload,
                buf,
                ..
            } => {
                if let Some(len) = length {
                    if len > limit {
                        return RicksponseBody::Error(Some(
                            RicksponsePayloadError::OverflowKnownLength { length: len, limit },
                        ));
                    }
                }

                RicksponseBody::Body {
                    limit,
                    content_type,
                    length,
                    payload,
                    buf,
                    _res: PhantomData,
                }
            }
            RicksponseBody::Error(e) => RicksponseBody::Error(e),
        }
    }
}

impl<T: DeserializeOwned> Future for RicksponseBody<T> {
    type Output = Result<T, RicksponsePayloadError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        match this {
            RicksponseBody::Body {
                limit,
                buf,
                payload,
                content_type,
                ..
            } => loop {
                let res = ready!(Pin::new(&mut *payload).poll_next(cx));
                match res {
                    Some(chunk) => {
                        let chunk = chunk?;
                        let buf_len = buf.len() + chunk.len();
                        if buf_len > *limit {
                            return Poll::Ready(Err(RicksponsePayloadError::Overflow {
                                limit: *limit,
                            }));
                        } else {
                            buf.extend_from_slice(&chunk);
                        }
                    }
                    None => {
                        let json = buf
                            .to_vec()
                            .as_slice()
                            .decode(content_type.deref())
                            .map_err(RicksponsePayloadError::Deserialize)?;
                        return Poll::Ready(Ok(json));
                    }
                }
            },
            RicksponseBody::Error(e) => Poll::Ready(Err(e.take().unwrap())),
        }
    }
}
