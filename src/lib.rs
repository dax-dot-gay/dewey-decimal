#![warn(missing_docs)]

//! Simple wrapper around the Dewey Decimal Classification system
//!
//! Provides functionality for fetching information about Dewey Decimal classes, along with methods for traversing the class hierarchy.
//!
//! Classes are automatically generated from [OpenLibrary](https://raw.githubusercontent.com/internetarchive/openlibrary/refs/heads/master/openlibrary/components/LibraryExplorer/ddc.json), or generated from an included JSON file if unable.

use trie_rs::map::Trie;
pub use trie_rs;

include!(concat!(env!("OUT_DIR"), "/classes.rs"));

static CLASSES: std::sync::LazyLock<Trie<u8, Class>> = std::sync::LazyLock::new(||
    make_class_static()
);

/// Stateless struct for getting [Class] instances
pub struct Dewey;

impl Dewey {
    /// Gets the underlying prefix trie ([crate::trie_rs::map::Trie])
    ///
    /// # Returns
    ///
    /// - `Trie<u8, Class>` - The underlying prefix trie
    pub fn map(&self) -> Trie<u8, Class> {
        CLASSES.to_owned()
    }

    fn as_label(&self, code: impl AsRef<str>) -> Vec<u8> {
        code.as_ref()
            .to_string()
            .trim_matches('X')
            .chars()
            .map(|c| c.to_string().parse::<u8>().unwrap())
            .collect()
    }

    /// Gets a class by exact code match
    ///
    /// # Arguments
    ///
    /// - `code` (`impl AsRef<str>`) - Code to search for
    ///
    /// # Returns
    ///
    /// - `Option<Class>` - The [Class] that matches the provided code, or [None] if not found.
    pub fn get_class(&self, code: impl AsRef<str>) -> Option<Class> {
        self.map().exact_match(self.as_label(code)).cloned()
    }

    /// Returns all classes matching the provided prefix
    ///
    /// # Arguments
    ///
    /// - `code` (`impl AsRef<str>`) - Code to search for
    ///
    /// # Returns
    ///
    /// - `Vec<Class>` - [Vec] of [Class] instances matching the prefix
    pub fn get_matches(&self, code: impl AsRef<str>) -> Vec<Class> {
        self.map()
            .predictive_search(self.as_label(code))
            .map(|item: (Vec<u8>, &Class)| item.1.clone())
            .collect()
    }

    /// Gets all the direct children of the class with the provided code
    ///
    /// # Arguments
    ///
    /// - `code` (`impl AsRef<str>`) - Code to search for
    ///
    /// # Returns
    ///
    /// - `Vec<Class>` - [Vec] of [Class] instances that are direct children of the specified prefix
    pub fn get_direct_children(&self, code: impl AsRef<str>) -> Vec<Class> {
        let code = code.as_ref().to_string();
        self.get_matches(code.clone())
            .into_iter()
            .filter_map(|c| {
                if c.code.len() == code.len() + 1 { Some(c) } else { None }
            })
            .collect()
    }

    /// Gets all children (not including the exact match itself)
    ///
    /// # Arguments
    ///
    /// - `code` (`impl AsRef<str>`) - Code to search for
    ///
    /// # Returns
    ///
    /// - `Vec<Class>` - [Vec] of all children of this prefix
    pub fn get_all_children(&self, code: impl AsRef<str>) -> Vec<Class> {
        let code = code.as_ref().to_string();
        self.get_matches(code.clone())
            .into_iter()
            .filter_map(|c| {
                if c.code == code { None } else { Some(c) }
            })
            .collect()
    }

    /// Gets the parent of the selected prefix, if any
    ///
    /// # Arguments
    ///
    /// - `code` (`impl AsRef<str>`) - Code to search for
    ///
    /// # Returns
    ///
    /// - `Option<Class>` - Parent of the selected [Class], if any
    pub fn get_parent(&self, code: impl AsRef<str>) -> Option<Class> {
        let mut code = code.as_ref().to_string();
        if code.len() > 1 {
            let _ = code.pop();
            self.get_class(code)
        } else {
            None
        }
    }

    /// Gets the top-level categories (codes `0` through `9`)
    ///
    /// # Returns
    ///
    /// - `Vec<Class>` - [Vec] of top-level classes
    pub fn categories(&self) -> Vec<Class> {
        "0123456789"
            .chars()
            .map(|c| self.get_class(c.to_string()).unwrap())
            .collect()
    }
}

impl Class {
    /// Gets a class based on a provided code (exact match)
    ///
    /// # Arguments
    ///
    /// - `code` (`impl AsRef<str>`) - Code to search for
    ///
    /// # Returns
    ///
    /// - `Option<Self>` - A new [Class] if found, otherwise [None]
    pub fn get(code: impl AsRef<str>) -> Option<Self> {
        Dewey.get_class(code)
    }

    /// See [Dewey::get_matches]
    pub fn matches(&self) -> Vec<Class> {
        Dewey.get_matches(self.code.clone())
    }

    /// See [Dewey::get_all_children]
    pub fn all_children(&self) -> Vec<Class> {
        Dewey.get_all_children(self.code.clone())
    }

    /// See [Dewey::get_direct_children]
    pub fn children(&self) -> Vec<Class> {
        Dewey.get_direct_children(self.code.clone())
    }

    /// See [Dewey::get_parent]
    pub fn parent(&self) -> Option<Class> {
        Dewey.get_parent(self.code.clone())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get() {
        for (code, name) in vec![
            ("247", "Church furnishings & related articles"),
            ("19", "Modern Western philosophy (19th-century, 20th-century)"),
            ("0", "Computer science, information & general works")
        ] {
            let result = Class::get(code);
            assert!(result.is_some(), "Expected Some(...)!");
            assert_eq!(result.unwrap().name, name.to_string(), "Names didn't match!");
        }

        assert!(Class::get("008").is_none(), "This code is unused!");
    }

    #[test]
    fn test_matches() {
        for (code, matches) in vec![("247", 1usize), ("09", 11usize), ("0", 98usize)] {
            let result = Class::get(code);
            assert!(result.is_some(), "Expected Some(...)!");
            assert_eq!(result.unwrap().matches().len(), matches, "Unexpected number of matches");
        }
    }
}
