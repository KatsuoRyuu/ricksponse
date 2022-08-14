use crate::entity::payload_control::PayloadControl;
use crate::entity::payload_error::PayloadError;
use crate::error::Error;
use actix_http::Payload;
use actix_web::HttpRequest;
use bytes::BytesMut;
use futures_core::Stream as _;
use http::header::CONTENT_LENGTH;
use serde::de::DeserializeOwned;
use simple_serde::{ContentType, Decoded, SimpleDecoder};
use std::future::Future;
use std::marker::PhantomData;
use std::ops::Deref;
use std::pin::Pin;
use std::task::{Context, Poll};

const DEFAULT_LIMIT: usize = 41_943_040; // 40 mb

pub enum PayloadBody<T, O> {
    Error(Option<PayloadError>),
    Body {
        limit: usize,
        /// Length as reported by `Content-Length` header, if present.
        length: Option<usize>,
        content_type: ContentType,
        payload: Payload,
        buf: BytesMut,
        _res: PhantomData<T>,
        _payload_res: PhantomData<O>,
    },
}

impl<T, O> Unpin for PayloadBody<T, O> {}

impl<T: DeserializeOwned, O: PayloadControl> PayloadBody<T, O> {
    /// Create a new future to decode a JSON request payload.
    #[allow(clippy::borrow_interior_mutable_const)]
    pub fn new(r: HttpRequest, payload: &mut Payload) -> Self {
        let length = r
            .headers()
            .get(&CONTENT_LENGTH)
            .ok_or(Error::NoPayloadSizeDefinitionInHeader)
            .and_then(|l| l.to_str().map_err(Error::from))
            .and_then(|s| s.parse::<usize>().map_err(Error::from));
        let content_type = Ok(r
            .headers()
            .get_all("Content-Type")
            .filter_map(|h| simple_serde::ContentType::try_from(h).ok())
            .collect::<Vec<ContentType>>())
        .and_then(|mut t: Vec<ContentType>| {
            t.reverse();
            t.pop().ok_or(Error::FailedToGetContentTypeFromHeader)
        });

        let payload = payload.take();

        match (content_type, length) {
            (Ok(c), Ok(l)) => PayloadBody::Body {
                limit: O::MAX_PAYLOAD_SIZE.unwrap_or(DEFAULT_LIMIT),
                content_type: c,
                length: Some(l),
                payload,
                buf: BytesMut::with_capacity(O::BUFFER_CAPACITY.unwrap_or(8192)),
                _res: PhantomData,
                _payload_res: PhantomData,
            },
            (Ok(c), _) => PayloadBody::Body {
                limit: O::MAX_PAYLOAD_SIZE.unwrap_or(DEFAULT_LIMIT),
                content_type: c,
                length: None,
                payload,
                buf: BytesMut::with_capacity(O::BUFFER_CAPACITY.unwrap_or(8192)),
                _res: PhantomData,
                _payload_res: PhantomData,
            },
            (_, _) => PayloadBody::Error(Some(PayloadError::ContentType)),
        }
    }

    /// Set maximum accepted payload size. The default limit is 2MB.
    pub fn limit(self, limit: usize) -> Self {
        match self {
            PayloadBody::Body {
                length,
                content_type,
                payload,
                buf,
                ..
            } => {
                if let Some(len) = length {
                    if len > limit {
                        return PayloadBody::Error(Some(PayloadError::OverflowKnownLength {
                            length: len,
                            limit,
                        }));
                    }
                }

                PayloadBody::Body {
                    limit,
                    content_type,
                    length,
                    payload,
                    buf,
                    _res: PhantomData,
                    _payload_res: PhantomData,
                }
            }
            PayloadBody::Error(e) => PayloadBody::Error(e),
        }
    }
}

impl<T: DeserializeOwned, O: PayloadControl> Future for PayloadBody<T, O> {
    type Output = Result<T, PayloadError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        match this {
            PayloadBody::Body {
                limit,
                buf,
                payload,
                content_type,
                ..
            } => loop {
                let res = match Pin::new(&mut *payload).poll_next(cx) {
                    std::task::Poll::Ready(t) => t,
                    std::task::Poll::Pending => {
                        return std::task::Poll::Pending;
                    }
                };
                match res {
                    Some(chunk) => {
                        let chunk = chunk?;
                        let buf_len = buf.len() + chunk.len();
                        if buf_len > *limit {
                            return Poll::Ready(Err(PayloadError::Overflow { limit: *limit }));
                        } else {
                            buf.extend_from_slice(&chunk);
                        }
                    }
                    None => {
                        let json = buf
                            .to_vec()
                            .as_slice()
                            .decode(content_type.deref())
                            .map(|d: Decoded<T>| d.into())
                            .map_err(PayloadError::Deserialize)?;
                        return Poll::Ready(Ok(json));
                    }
                }
            },
            PayloadBody::Error(e) => Poll::Ready(Err(e.take().unwrap())),
        }
    }
}
