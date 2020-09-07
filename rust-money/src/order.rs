//! # Management of a *transaction*.

use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};

/// Data associated to a unique transaction.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Order {
    pub date: Option<NaiveDate>,
    pub description: String,
    pub amount: f32,
    resource: Option<String>,
    tags: Vec<String>,
    state: TransactionState,
    pub visible: bool,
}

/// Different states for a given transaction.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum TransactionState {
    /// No payment performed yet.
    Pending = 0,
    /// Payment ordered but postponed.
    InProgress = 1,
    /// Payment done.
    Done = 2,
}

impl Default for Order {
    fn default() -> Self {
        Order {
            date: None,
            description: "".to_string(),
            amount: 0.0,
            resource: None,
            tags: Vec::new(),
            state: TransactionState::Pending,
            visible: true,
        }
    }
}

impl Order {
    /// Selects the resource among available ones.
    pub fn set_resource(&mut self, resource: &str, list: &[String]) -> bool {
        if list.contains(&resource.into()) {
            self.resource = Some(resource.into());
            true
        } else {
            false
        }
    }

    /// Resets the resource.
    pub fn clear_resource(&mut self) {
        self.resource = None;
    }

    /// Exports *resource* reference.
    pub fn resource(&self) -> &Option<String> {
        &self.resource
    }

    /// Selects a tag among available ones.
    pub fn add_tag(&mut self, tag: &str, list: &[String]) -> bool {
        if list.contains(&tag.into()) {
            let tag = tag.to_string();

            if !self.tags.contains(&tag) {
                self.tags.push(tag);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Removes a tag among added ones.
    pub fn remove_tag(&mut self, tag: &str) -> bool {
        let tag = tag.to_string();

        if self.tags.contains(&tag) {
            self.tags.retain(|x| x != &tag);
            true
        } else {
            false
        }
    }

    /// Removes all tags.
    pub fn clear_tags(&mut self) {
        self.tags.clear();
    }

    /// Exports *tags* reference.
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    /// Sets the current state of the order.
    /// `Done`triggers a default *date*.
    pub fn set_state(&mut self, state: TransactionState) {
        if let TransactionState::Done = state {
            if self.date.is_none() {
                self.date = Some(Local::today().naive_local());
            }
        }

        self.state = state;
    }

    /// Gets current state.
    pub fn state(&self) -> TransactionState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_valid_resource() {
        let resources = ["Bank".to_string(), "Cash".to_string()];
        let mut order = Order::default();

        assert_eq!(order.set_resource(resources[1].as_str(), &resources), true);
        assert_eq!(
            order,
            Order {
                resource: Some(resources[1].clone()),
                ..Order::default()
            }
        );
    }

    #[test]
    fn discard_invalid_resource() {
        let resources = ["Bank".to_string(), "Cash".to_string()];
        let mut order = Order::default();

        assert_eq!(order.set_resource("Gift card", &resources), false);
        assert_eq!(order, Order::default());
    }

    #[test]
    fn add_valid_tag() {
        let tags = ["Food".to_string(), "Service".to_string()];
        let mut order = Order::default();

        assert_eq!(order.add_tag(tags[1].as_str(), &tags), true);
        assert_eq!(
            order,
            Order {
                tags: vec![tags[1].clone()],
                ..Order::default()
            }
        );
    }

    #[test]
    fn discard_existing_tag() {
        let tags = ["Food".to_string(), "Service".to_string()];
        let mut order = Order::default();

        assert_eq!(order.add_tag(tags[1].as_str(), &tags), true);
        assert_eq!(order.add_tag(tags[1].as_str(), &tags), false);
        assert_eq!(
            order,
            Order {
                tags: vec![tags[1].clone()],
                ..Order::default()
            }
        );
    }

    #[test]
    fn discard_invalid_tag() {
        let tags = ["Food".to_string(), "Home".to_string()];
        let mut order = Order::default();

        assert_eq!(order.add_tag("Other tag", &tags), false);
        assert_eq!(order, Order::default());
    }

    #[test]
    fn remove_valid_tag() {
        let tags = [
            "Food".to_string(),
            "Service".to_string(),
            "Video Games".to_string(),
            "Home".to_string(),
        ];
        let mut order = Order {
            tags: tags.iter().map(|x| x.to_string()).collect(),
            ..Order::default()
        };

        assert_eq!(order.remove_tag(tags[0].as_str()), true);
        assert_eq!(
            order,
            Order {
                tags: tags[1..].to_vec(),
                ..Order::default()
            }
        );
    }

    #[test]
    fn remove_invalid_tag() {
        let tags = ["Service".to_string(), "Video Games".to_string()];
        let mut order = Order {
            tags: tags.iter().map(|x| x.to_string()).collect(),
            ..Order::default()
        };

        assert_eq!(order.remove_tag("Unknown"), false);
        assert_eq!(
            order,
            Order {
                tags: tags.to_vec(),
                ..Order::default()
            }
        );
    }
}