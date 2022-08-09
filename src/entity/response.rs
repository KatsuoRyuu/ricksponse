use railsgun::OptionsExtended;
use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[derive(Serialize, PartialEq, Debug, Default)]
pub struct Metadata {
    pub(crate) annotations: HashMap<String, String>,
}
/// # Metadata
/// Metadata is a collection of alternative information send/recieved from the system
///
/// ## Adding annotation
/// ```
/// use ricksponse::response::{Metadata};
///
/// let mut metadata = Metadata::default();
///
/// metadata.add_annotation("foo", "bar");
/// ```
impl Metadata {
    /// # Metadata - adding annotations
    ///
    /// ```
    /// use ricksponse::response::{Metadata};
    ///
    /// let mut metadata = Metadata::default();
    ///
    /// metadata.add_annotation("foo", "bar");
    /// ```
    pub fn add_annotation(&mut self, key: &str, value: &str) {
        self.annotations.insert(key.to_string(), value.to_string());
    }
}

#[derive(Serialize, Debug, PartialEq, Default)]
pub struct Status {
    pub(crate) message: Option<String>,
    pub(crate) code: Option<u32>,
    pub(crate) http_status_code: Option<u16>,
    pub(crate) session: Option<uuid::Uuid>,
}

impl Status {
    pub fn new(
        message: Option<String>,
        code: Option<u32>,
        http_status_code: Option<u16>,
        session: Option<uuid::Uuid>,
    ) -> Self {
        Status {
            message,
            code,
            http_status_code,
            session,
        }
    }
    /// Status - Message
    /// This is for setting a message on the status.
    ///
    /// ```
    /// use ricksponse::response::Status;
    ///
    /// let mut status = Status::default();
    /// status.message("hello world");
    ///
    /// assert_eq!(status, Status::new(Some("hello world".to_string()), None, None, None));
    /// ```
    pub fn message(&mut self, message: &str) {
        self.message = Some(message.to_string());
    }

    /// ```
    /// use ricksponse::response::Status;
    ///
    /// let mut status = Status::default();
    /// status.code(&100);
    ///
    /// assert_eq!(status, Status::new(None, Some(100), None, None));
    /// ```
    pub fn code(&mut self, code: &u32) {
        self.code = Some(*code);
    }

    /// ```
    /// use ricksponse::response::Status;
    ///
    /// let mut status = Status::default();
    /// status.http_status_code(&200);
    ///
    /// assert_eq!(status, Status::new(None, None, Some(200), None));
    /// ```
    pub fn http_status_code(&mut self, http_status_code: &u16) {
        self.http_status_code = Some(*http_status_code);
    }

    /// ```
    /// use ricksponse::response::Status;
    ///
    /// let mut status = Status::default();
    /// let uuid = uuid::Uuid::new_v4();
    /// status.session(&uuid);
    ///
    /// assert_eq!(status, Status::new(None, None, None, Some(uuid)));
    /// ```
    pub fn session(&mut self, session: &uuid::Uuid) {
        self.session = Some(*session);
    }
}

#[derive(Serialize)]
pub struct HateoasResponse<T: Serialize> {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub metadata: Option<Metadata>,
    pub spec: Option<Content<T>>,
    pub status: Option<Status>,
}

impl<T: Serialize + Default> HateoasResponse<T> {
    /// Getting the metadata from the response.
    /// By default metadata is not initialized and will be initialized upon usage.
    /// ```
    /// use ricksponse::response::{HateoasResponse, Metadata};
    ///
    /// let mut response: HateoasResponse<()> = HateoasResponse::default();
    /// let mut metadata = Metadata::default();
    ///
    /// assert_eq!(&mut metadata, response.metadata());
    /// ```
    pub fn metadata(&mut self) -> &mut Metadata {
        self.metadata.get_or_insert_default()
    }

    /// Get The status information from the response,
    /// If this is not initialized it will be initialized and returned.
    /// ```
    /// use ricksponse::response::{HateoasResponse, Status};
    ///
    /// let mut response: HateoasResponse<()> = HateoasResponse::default();
    ///
    /// let mut status = response.status();
    /// assert_eq!(&mut Status::default(), status)
    /// ```
    pub fn status(&mut self) -> &mut Status {
        self.status.get_or_insert_default()
    }

    /// Get the spec information form the Response payload
    ///
    /// ```
    /// use ricksponse::response::{Content, HateoasResponse};
    /// let mut response: HateoasResponse<String> = HateoasResponse::default();
    ///
    /// // Here spec will be None at initialization time.
    /// // at [Response.spec()] Spec will be initialized and returned.
    ///
    /// let mut spec = response.spec();
    /// assert_eq!(&mut Content::default(), spec)
    /// ```
    pub fn spec(&mut self) -> &mut Content<T> {
        self.spec.get_or_insert_default()
    }
}

impl<T: Serialize> Default for HateoasResponse<T> {
    fn default() -> Self {
        HateoasResponse {
            api_version: "".to_string(),
            kind: "".to_string(),
            metadata: None,
            spec: None,
            status: None,
        }
    }
}

#[derive(Serialize, PartialEq, Debug)]
pub struct Content<T> {
    content: Option<T>,
    rel: Option<RelLinkCollection>,
}
impl<T> Content<T> {
    /// Setting the content on the Content container
    ///
    /// ```
    /// use ricksponse::response::Content;
    /// let mut ctn_with_content = Content::default();
    /// ctn_with_content.content(());
    ///
    /// assert_eq!(ctn_with_content.has_content(), true);
    /// assert_eq!(ctn_with_content.get_content(), &Some(()));
    /// ```
    pub fn content(&mut self, content: T) {
        self.content = Some(content);
    }

    /// Checking if the content has any information in it, eg. is not none
    ///
    /// ```
    /// use ricksponse::response::Content;
    /// let ctn: Content<()> = Content::default();
    /// let mut ctn_with_content = Content::default();
    /// ctn_with_content.content(());
    ///
    /// assert_eq!(ctn.has_content(), false);
    /// assert_eq!(ctn_with_content.has_content(), true);
    /// ```
    pub fn has_content(&self) -> bool {
        self.content.is_some()
    }

    /// Getting a mut reference of the current spec content
    /// This will get a Option<&mut T> of the current contents spec piece.
    /// This will allow for modification of the internal content in the spec
    /// ```
    /// use ricksponse::response::{Content};
    /// let mut ctn: Content<String> = Content::default();
    ///
    /// assert_eq!(ctn.get_content(), &None);
    ///
    /// ctn.content("foo".to_string());
    ///
    /// assert_eq!(ctn.get_content(), &Some("foo".to_string()));
    ///
    /// let mut_ctn = ctn.get_mut_content();
    /// mut_ctn.map(|t| *t = "bar".to_string());
    ///
    /// assert_eq!(ctn.get_content(), &Some("bar".to_string()));
    /// ```
    pub fn get_mut_content(&mut self) -> Option<&mut T> {
        self.content.as_mut()
    }

    /// Getting a reference of the current spec content
    /// This will get a Option<T> of the current contents spec piece
    /// ```
    /// use ricksponse::response::{Content, RelLinkCollection};
    /// let mut ctn = Content::default();
    ///
    /// assert_eq!(ctn.get_content(), &None);
    ///
    /// ctn.content(());
    ///
    /// assert_eq!(ctn.get_content(), &Some(()))
    /// ```
    pub fn get_content(&self) -> &Option<T> {
        &self.content
    }

    /// Get the rel even if not set.
    ///
    /// ```
    /// use ricksponse::response::{Content, RelLinkCollection};
    ///
    /// let mut content: Content<()> = Content::default();
    /// let rel = content.rel();
    ///
    /// assert_eq!(rel, &mut RelLinkCollection::default())
    /// ```
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

impl<T> Deref for Content<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl<T> DerefMut for Content<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }
}

/// # RelLinkCollection
///
///
/// ## Adding new data to the collection
/// ```
/// use ricksponse::response::{HttpMethod, RelLink, RelLinkCollection};
///
/// let rel_vec = vec![
///     RelLink::new("foo", "foo", HttpMethod::Get),
///     RelLink::new("bar", "bar", HttpMethod::Get)
/// ];
/// let rlc_check = RelLinkCollection::new(rel_vec);
///
/// let mut rlc = RelLinkCollection::default();
/// rlc.add("foo", RelLink::new("foo", "foo", HttpMethod::Get));
/// rlc.add("bar", RelLink::new("bar", "bar", HttpMethod::Get));
/// ```
///
/// ## Adding new data but data is overwritten
/// ```
/// use ricksponse::response::{HttpMethod, RelLink, RelLinkCollection};
///
/// let rel_vec = vec![
///     RelLink::new("foo", "foo", HttpMethod::Get),
///     RelLink::new("bar", "bar", HttpMethod::Get)
/// ];
/// let mut rlc = RelLinkCollection::new(rel_vec);
///
/// let old_rel = rlc.add("foo", RelLink::new("foo-bar", "foo", HttpMethod::Get));
///
/// assert_eq!(old_rel, Some(("foo", "foo", HttpMethod::Get).into()));
/// ```
///
/// ## Get RelLink from collection
/// ```
/// use ricksponse::response::{HttpMethod, RelLink, RelLinkCollection};
///
/// let rel_vec = vec![
///     RelLink::new("foo", "foo", HttpMethod::Get),
///     RelLink::new("bar", "bar", HttpMethod::Get)
/// ];
/// let mut rlc = RelLinkCollection::new(rel_vec);
///
/// let rel = rlc.get("foo");
///
/// assert_eq!(rel, Some(&("foo", "foo", HttpMethod::Get).into()));
/// ```
///
/// ## Get Mutable RelLink from and updateing it.
/// ```
/// use ricksponse::response::{HttpMethod, RelLink, RelLinkCollection};
///
/// let rel_vec = vec![
///     RelLink::new("foo", "foo", HttpMethod::Get),
///     RelLink::new("bar", "bar", HttpMethod::Get)
/// ];
/// let mut rlc = RelLinkCollection::new(rel_vec);
///
/// let mut rel = rlc.get_mut("foo");
///
/// assert_eq!(rel, Some(&mut ("foo", "foo", HttpMethod::Get).into()));
///
/// rel.map(|t| *t = ("foo-bar", "foo-bar", HttpMethod::Connect).into());
///
/// let updated_rel = rlc.get("foo-bar");
///
/// assert_eq!(updated_rel, Some(&("foo-bar", "foo-bar", HttpMethod::Connect).into()));
/// ```

#[derive(Debug, Serialize, PartialEq, Default)]
pub struct RelLinkCollection(Vec<RelLink>);

impl RelLinkCollection {
    pub fn new(v_rel: Vec<RelLink>) -> Self {
        RelLinkCollection(v_rel)
    }

    pub fn get(&self, rel: &str) -> Option<&RelLink> {
        self.0.iter().find(|rl| rl.rel == rel)
    }

    pub fn has(&self, rel: &str) -> bool {
        self.get(rel).is_some()
    }

    pub fn get_mut(&mut self, rel: &str) -> Option<&mut RelLink> {
        self.0.iter_mut().find(|rl| rl.rel == rel)
    }

    pub fn add(&mut self, rel: &str, link: RelLink) -> Option<RelLink> {
        let mut new_link = link;
        new_link.rel = rel.to_string();
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

impl<I: Into<RelLink>> From<I> for RelLinkCollection {
    fn from(r: I) -> Self {
        RelLinkCollection(vec![r.into()])
    }
}

impl<I: Into<RelLink>> From<Vec<I>> for RelLinkCollection {
    fn from(v_rel: Vec<I>) -> Self {
        RelLinkCollection(
            v_rel
                .into_iter()
                .map(|e| e.into())
                .collect::<Vec<RelLink>>(),
        )
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RelLink {
    href: String,
    rel: String,
    method: HttpMethod,
}

impl RelLink {
    pub fn new(href: &str, rel: &str, method: HttpMethod) -> Self {
        RelLink {
            href: href.to_string(),
            rel: rel.to_string(),
            method,
        }
    }
}

impl From<(String, String, HttpMethod)> for RelLink {
    fn from(r: (String, String, HttpMethod)) -> Self {
        Self::new(&r.0, &r.1, r.2)
    }
}

impl From<(&str, &str, HttpMethod)> for RelLink {
    fn from(r: (&str, &str, HttpMethod)) -> Self {
        Self::new(r.0, r.1, r.2)
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
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
    use crate::response::{Content, HateoasResponse, RelLinkCollection};
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

    #[test]
    fn test_content_rel() {
        let mut content: Content<()> = Content::default();
        let rel = content.rel();
        assert_eq!(&mut RelLinkCollection::default(), rel);
    }

    #[test]
    fn test_get_spec_on_none() {
        let mut response: HateoasResponse<String> = HateoasResponse::default();

        // Here spec will be None at initialization time.
        // at [Response.spec()] Spec will be initialized and returned.

        let mut spec = response.spec();
        assert_eq!(&mut Content::default(), spec)
    }
}
