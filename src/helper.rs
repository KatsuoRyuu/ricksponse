use crate::error::Error::*;
use std::collections::HashSet;

fn get_from_hash_set(hash: HashSet<&str>) -> crate::Result<simple_serde::ContentType> {
    for i in hash {
        if let Ok(content_type) = simple_serde::ContentType::try_from(i) {
            return Ok(content_type);
        }
    }
    Err(FailedToMatchAnyContentType)
}
