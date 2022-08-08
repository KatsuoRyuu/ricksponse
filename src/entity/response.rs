use std::collections::HashMap;
use serde::Serialize;

#[derive(Serialize)]
pub struct Metadata {
    pub(crate) annotations: Option<HashMap<String, String>>
}

#[derive(Serialize)]
pub struct Status {
    pub(crate) message: Option<String>,
    pub(crate) code: Option<u32>,
    pub(crate) http_status_code: Option<u16>,
    pub(crate) session: Option<uuid::Uuid>,
}

impl Default for Status {
    fn default() -> Self {
        Status {
            message: Some("Default Response - Possibly API is only a placeholder".to_string()),
            code: Some(0),
            http_status_code: Some(200),
            session: None,
        }
    }
}

#[derive(Serialize)]
pub struct Response<T: Serialize> {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub metadata: Metadata,
    pub spec: Option<Content<T>>,
    pub rel: Option<Vec<RelLink>>,
    pub status: Option<Status>
}

#[derive(Serialize)]
pub struct Content<T> {
    content: Option<T>,
    rel: Option<Vec<RelLink>>,
}

impl Content


#[derive(Serialize)]
pub struct RelLink {
    href: String,
    rel: String,
    method: HttpMethod
}


#[derive(Serialize)]
pub enum HttpMethod {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
    Patch,
}

#[cfg(test)]
mod test {
    use super::Response;
    use simple_serde::SimpleEncoder;

    #[test]
    fn default_response_test() {
        let response: Response<String> = Response {
            content: None,
            metadata: Default::default(),
        };
        let response_ser = response.encode("yaml");

        // println!("{}", response_ser);
        // assert_eq!()
    }
}
