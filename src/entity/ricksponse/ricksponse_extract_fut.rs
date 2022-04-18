use crate::entity::ricksponse::ricksponse::Ricksponse;
use crate::entity::ricksponse::ricksponse_body::RicksponseBody;
use crate::error::Error;
use actix_http::Payload;
use actix_web::HttpRequest;
use serde::de::DeserializeOwned;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct RicksponseExtractFut<T> {
    pub(crate) req: Option<HttpRequest>,
    pub(crate) fut: RicksponseBody<T>,
}

impl<T: DeserializeOwned> RicksponseExtractFut<T> {
    pub(crate) fn new(r: HttpRequest, p: &mut Payload) -> RicksponseExtractFut<T> {
        RicksponseExtractFut {
            req: Some(r.clone()),
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
            Err(err) => {
                let req = this.req.take().unwrap();
                println!(
                    "Failed to deserialize payload. \
                         Request path: {}; error: {:?}",
                    req.path(),
                    err
                );
                Err(err.into())
            }
            Ok(data) => Ok(Ricksponse::new(data)),
        })
    }
}
