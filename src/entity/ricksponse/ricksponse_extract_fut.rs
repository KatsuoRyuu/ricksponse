use crate::entity::ricksponse::ricksponse::Ricksponse;
use crate::entity::ricksponse::ricksponse_body::RicksponseBody;
use crate::error::Error;
use actix_http::Payload;
use actix_web::HttpRequest;
use serde::de::DeserializeOwned;
use std::fmt::Display;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct RicksponseExtractFut<T, E> {
    pub(crate) _req: Option<HttpRequest>,
    pub(crate) fut: RicksponseBody<T>,
    pub(crate) _phantom: Pin<Box<PhantomData<E>>>,
}

impl<T: DeserializeOwned, E: DeserializeOwned> RicksponseExtractFut<T, E> {
    pub(crate) fn new(r: HttpRequest, p: &mut Payload) -> RicksponseExtractFut<T, E> {
        RicksponseExtractFut {
            _req: Some(r.clone()),
            fut: RicksponseBody::new(r, p),
            _phantom: Box::pin(Default::default()),
        }
    }
}

impl<T: DeserializeOwned, E: DeserializeOwned + Display> Future for RicksponseExtractFut<T, E> {
    type Output = Result<Ricksponse<T, E>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let res = match Pin::new(&mut this.fut).poll(cx) {
            std::task::Poll::Ready(t) => t,
            std::task::Poll::Pending => {
                return std::task::Poll::Pending;
            }
        };
        Poll::Ready(match res {
            Err(err) => Err(err.into()),
            Ok(data) => Ok(Ricksponse::from(Ok(data))),
        })
    }
}
