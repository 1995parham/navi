use crate::common::hash::fnv;

#[derive(Default, Debug)]
pub struct Item {
    pub tags: String,
    pub comment: String,
    pub snippet: String,
    pub file_index: Option<usize>,
    pub path_filter: Option<String>,
    pub os_filter: Option<String>,
    pub hostname_filter: Option<String>,
    pub env_filter: Option<String>,
}

impl Item {
    pub fn new(file_index: Option<usize>) -> Self {
        Self {
            file_index,
            ..Default::default()
        }
    }

    /// Identifies an item for deduplication.
    ///
    /// The fields are hashed as a tuple rather than concatenated: `str`'s `Hash`
    /// terminates each field, so two items that differ only in where the
    /// boundary between fields falls no longer collide.
    pub fn hash(&self) -> u64 {
        fnv(&(self.tags.trim(), self.comment.trim(), self.snippet.trim()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(tags: &str, comment: &str, snippet: &str) -> Item {
        Item {
            tags: tags.to_string(),
            comment: comment.to_string(),
            snippet: snippet.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_hash_ignores_surrounding_whitespace() {
        assert_eq!(
            item("git", "c", "s").hash(),
            item(" git ", " c ", " s ").hash()
        );
    }

    #[test]
    fn test_hash_distinguishes_shifted_field_boundaries() {
        assert_ne!(item("ab", "", "cd").hash(), item("a", "", "bcd").hash());
        assert_ne!(item("git", "log", "").hash(), item("git", "", "log").hash());
    }
}
