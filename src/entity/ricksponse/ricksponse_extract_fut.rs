use crate::entity::ricksponse::ricksponse::Ricksponse;
use crate::entity::ricksponse::ricksponse_body::RicksponseBody;
use crate::error::Error;
use crate::RicksponsePayloadError;
use actix_http::Payload;
use actix_web::HttpRequest;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct RicksponseExtractFut<T> {
    pub(crate) _req: Option<HttpRequest>,
    pub(crate) fut: RicksponseBody<T>,
}

impl<T: DeserializeOwned> RicksponseExtractFut<T> {
    pub(crate) fn new(r: HttpRequest, p: &mut Payload) -> RicksponseExtractFut<T> {
        RicksponseExtractFut {
            _req: Some(r.clone()),
            fut: RicksponseBody::new(r, p),
        }
    }
}

impl<T: DeserializeOwned> Future for RicksponseExtractFut<T> {
    type Output = Result<Ricksponse<T>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let res = match Pin::new(&mut this.fut).poll(cx) {
            std::task::Poll::Ready(t) => t,
            std::task::Poll::Pending => {
                return std::task::Poll::Pending;
            }
        };
        Poll::Ready(match res {
            Err(err) => Ok(Ricksponse::from(
                Err(err.into()) as Result<T, RicksponsePayloadError>
            )),
            Ok(data) => Ok(Ricksponse::from(
                Ok(data) as Result<T, RicksponsePayloadError>
            )),
        })
    }
}
