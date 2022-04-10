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

#[cfg(test)]
mod test {
    use crate::helper::get_from_hash_set;
    use std::collections::HashSet;

    #[test]
    fn test_get_from_hash_set() {
        let mut hash_set = HashSet::new();
        hash_set.insert("blabla");
        hash_set.insert("foo");
        hash_set.insert("bar");
        hash_set.insert("yaml");

        assert!(get_from_hash_set(hash_set).is_ok())
    }

    #[test]
    fn test_get_from_hash_set_failure() {
        let mut hash_set = HashSet::new();
        hash_set.insert("blabla");
        hash_set.insert("foo");
        hash_set.insert("bar");

        assert!(!get_from_hash_set(hash_set).is_ok())
    }
}
