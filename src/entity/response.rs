use crate::helpers_prelude::OptionsExtended;
use serde::{Serialize, Serializer};
use std::collections::HashMap;

#[derive(Serialize)]
pub struct Metadata {
    pub(crate) annotations: HashMap<String, String>,
}

impl Metadata {
    pub fn add_annotation(&mut self, key: String, value: String) {
        self.annotations.insert(key, value);
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            annotations: HashMap::new(),
        }
    }
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
    pub status: Option<Status>,
}

impl<T: Serialize> Response<T> {
    fn status(&mut self) -> &mut Option<Status> {
        if self.status.is_none() {
            self.status = Some(Status::default());
        }
        &mut self.status
    }

    fn spec(&mut self) -> &mut Option<Content<T>> {
        if self.spec.is_none() {
            self.spec = Some(Content::default());
        }
        &mut self.spec
    }
}

#[derive(Serialize)]
pub struct Content<T> {
    content: Option<T>,
    rel: Option<RelLinkCollection>,
}
impl<T> Content<T> {
    pub fn content(&mut self, content: T) {
        self.content = Some(content);
    }

    pub fn has_content(&self) -> bool {
        self.content.is_some()
    }

    pub fn rel(&mut self) -> &mut RelLinkCollection {
        if self.rel.is_none() {
            self.rel = Some(RelLinkCollection::default());
        }

        self.rel.get_or_insert_default()
    }
}

impl<T: Serialize> Default for Content<T> {
    fn default() -> Self {
        Content {
            content: None,
            rel: None,
        }
    }
}

#[derive(Serialize)]
pub struct RelLinkCollection(Vec<RelLink>);

impl RelLinkCollection {
    fn get(&self, rel: String) -> Option<&RelLink> {
        self.0.iter().find(|rl| rl.rel == rel)
    }

    fn has(&self, rel: String) -> bool {
        self.get(rel).is_some()
    }

    fn get_mut(&mut self, rel: String) -> Option<&mut RelLink> {
        self.0.iter_mut().find(|rl| rl.rel == rel)
    }

    fn add(&mut self, rel: String, link: RelLink) -> Option<RelLink> {
        let mut new_link = link;
        new_link.rel = rel.clone();
        let mut old_link = None;
        if let Some(found_rel) = self.get_mut(rel) {
            old_link = Some(found_rel.clone());
            *found_rel = new_link;
        } else {
            self.0.push(new_link)
        }
        old_link
    }
}

impl Default for RelLinkCollection {
    fn default() -> Self {
        Self(Vec::new())
    }
}

#[derive(Clone, Serialize)]
pub struct RelLink {
    href: String,
    rel: String,
    method: HttpMethod,
}

impl RelLink {
    pub fn new(href: String, rel: String, method: HttpMethod) -> Self {
        RelLink { href, rel, method }
    }
}

#[derive(Clone, Serialize)]
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

    // #[test]
    // fn default_response_test() {
    //     let response: Response<String> = Response {
    //         content: None,
    //         metadata: Default::default(),
    //     };
    //     let response_ser = response.encode("yaml");
    //
    //     // println!("{}", response_ser);
    //     // assert_eq!()
    // }
}
