//! # Money - Core Rust API
//!
//! `money` is a collection of utilities to make tracking money expenses.

mod ext;

use ext::ExclusiveItemExt;

/// Manages account data.
pub struct Account {
    tags: Vec<String>,
    resources: Vec<String>,
}

impl Account {
    /// Instantiates a new object.
    pub fn create() -> Account {
        Account {
            tags: Vec::new(),
            resources: Vec::new(),
        }
    }

    /// Add a new tag.
    pub fn add_tag(&mut self, tag: &str) {
        self.tags.add_exclusive(tag);
    }

    /// Returns available tags.
    pub fn tags(&self) -> &Vec<String> {
        &self.tags
    }

    /// Remove a tag.
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.remove_exclusive(tag);
    }

    /// Add a new resource.
    pub fn add_resource(&mut self, resource: &str) {
        self.resources.add_exclusive(resource);
    }

    /// Remove a resource.
    pub fn remove_resource(&mut self, resource: &str) {
        self.resources.remove_exclusive(resource);
    }

    /// Returns available resources.
    pub fn resources(&self) -> &Vec<String> {
        &self.resources
    }
}
