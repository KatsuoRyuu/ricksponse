use serde::Serialize;

#[derive(Serialize)]
pub struct ResponseStatus {
    pub(crate) message: Option<String>,
    pub(crate) code: Option<u32>,
    pub(crate) http_status_code: Option<u16>,
    pub(crate) session: Option<uuid::Uuid>,
}

impl Default for ResponseStatus {
    fn default() -> Self {
        ResponseStatus {
            message: Some("Default Response - Possibly API is only a placeholder".to_string()),
            code: Some(0),
            http_status_code: Some(200),
            session: None,
        }
    }
}

#[derive(Serialize)]
pub struct Response<T: Serialize> {
    pub content: Option<T>,
    pub(crate) status: ResponseStatus,
}

#[cfg(test)]
mod test {
    use super::Response;
    use simple_serde::SimpleEncoder;

    #[test]
    fn default_response_test() {
        let response: Response<String> = Response {
            content: None,
            status: Default::default(),
        };
        let response_ser = response.encode("yaml");

        // println!("{}", response_ser);
        // assert_eq!()
    }
}
