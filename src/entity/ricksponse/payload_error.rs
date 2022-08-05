use actix_http::error::PayloadError;
use actix_web::ResponseError;
use derive_more::Display;
use http::StatusCode;

#[derive(Debug, Display)]
#[non_exhaustive]
pub enum RicksponsePayloadError {
    /// Payload size is bigger than allowed & content length header set. (default: 2MB)
    #[display(
        fmt = "Ricksponse payload ({} bytes) is larger than allowed (limit: {} bytes).",
        length,
        limit
    )]
    OverflowKnownLength { length: usize, limit: usize },

    /// Payload size is bigger than allowed but no content length header set. (default: 2MB)
    #[display(fmt = "payload has exceeded limit ({} bytes).", limit)]
    Overflow { limit: usize },

    /// Content type error
    #[display(fmt = "Content type error")]
    ContentType,

    /// Deserialize error
    #[display(fmt = "Deserialize error: {:?}", _0)]
    Deserialize(simple_serde::Error),

    /// Serialize error
    #[display(fmt = "Serialize error: {:?}", _0)]
    Serialize(simple_serde::Error),

    /// Payload error
    #[display(fmt = "Error that occur during reading payload: {}", _0)]
    Payload(PayloadError),

    /// Payload error
    #[display(
        fmt = "Failed to deserialize payload under future stream assembly. Request path {}, Error {}",
        _0,
        _1
    )]
    RickspnsePayloadError(String, Box<RicksponsePayloadError>),
}

impl From<PayloadError> for RicksponsePayloadError {
    fn from(err: PayloadError) -> Self {
        Self::Payload(err)
    }
}

impl ResponseError for RicksponsePayloadError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::OverflowKnownLength {
                length: _,
                limit: _,
            } => StatusCode::PAYLOAD_TOO_LARGE,
            Self::Overflow { limit: _ } => StatusCode::PAYLOAD_TOO_LARGE,
            Self::Serialize(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Payload(err) => err.status_code(),
            Self::RickspnsePayloadError(_, _) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::BAD_REQUEST,
        }
    }
}
