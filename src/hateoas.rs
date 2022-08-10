impl<T: Serialize> AsHateoasResponse<T> for Ricksponse<HateoasResponse<T>> {
    fn as_response(&mut self) -> &mut HateoasResponse<T> {
        self.get_or_with_data(HateoasResponse::default())
    }
}

impl<T: Serialize> ToHateoasResponse<Ricksponse<HateoasResponse<T>>> for Ricksponse<T> {
    fn to_hateoas_response(self) -> Ricksponse<HateoasResponse<T>> {
        match self {
            Ricksponse::Data {
                data,
                http_code,
                message,
            } => Ricksponse::new(HateoasResponse {
                api_version: "Unknown.invalid/0.0.0".to_string(),
                kind: "".to_string(),
                metadata: None,
                spec: None,
                status: Some(Status {
                    message,
                    code: None,
                    http_status_code: http_code,
                    session: None,
                }),
            }),
            Self::Error {
                error,
                http_code,
                message,
            } => Ricksponse::from(HateoasResponse {
                api_version: "Unknown.invalid/0.0.0".to_string(),
                kind: "".to_string(),
                metadata: None,
                spec: None,
                status: Some(Status {
                    message,
                    code: None,
                    http_status_code: http_code,
                    session: None,
                }),
            }),
        }
    }
}
