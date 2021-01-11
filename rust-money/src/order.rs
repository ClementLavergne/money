//! # Management of a *transaction*.

use super::ext::ExclusiveItemExt;
use crate::filter::category::CategoryFilter;
use crate::filter::date::NaiveDateFilter;
use crate::filter::{Filter, ItemSelector, VisibilityFilter};
use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasmbind")]
use wasm_bindgen::prelude::*;

/// Data associated to a unique transaction.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Order {
    pub date: Option<NaiveDate>,
    pub description: String,
    pub amount: f32,
    pub(crate) resource: Option<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) state: TransactionState,
    pub visible: bool,
}

/// Different states for a given transaction.
#[cfg_attr(feature = "wasmbind", wasm_bindgen)]
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

impl From<&Filter> for Order {
    fn from(filter: &Filter) -> Self {
        Order {
            date: match filter.date_option {
                NaiveDateFilter::DateIgnored => None,
                NaiveDateFilter::Since(date) | NaiveDateFilter::Between(date, _) => Some(date),
                NaiveDateFilter::Until(date) => Some(date),
            },
            // Pick-up the first selected one
            resource: if let CategoryFilter::Enabled(items) = &filter.resource_option {
                if let Some(first_selected) = items
                    .iter()
                    .find(|resource| resource.1 == ItemSelector::Selected)
                {
                    Some(first_selected.0.clone())
                } else {
                    None
                }
            } else {
                None
            },
            // Pick-up all selected ones
            tags: if let CategoryFilter::Enabled(items) = &filter.tag_option {
                items
                    .iter()
                    .filter_map(|tag| {
                        if tag.1 == ItemSelector::Selected {
                            Some(tag.0.clone())
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            },
            // Pick-up the first selected one
            state: if let Some(first_selected) = filter
                .state_option
                .iter()
                .enumerate()
                .find(|(_, &state)| state == ItemSelector::Selected)
            {
                unsafe { std::mem::transmute(first_selected.0 as u8) }
            } else {
                TransactionState::Pending
            },
            visible: !matches!(filter.visibility, VisibilityFilter::HiddenOnly),
            ..Order::default()
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

    /// Selects a tag among available ones.
    pub fn add_tag(&mut self, tag: &str, list: &[String]) -> bool {
        if list.contains(&tag.into()) {
            self.tags.add_exclusive(tag).is_none()
        } else {
            false
        }
    }

    /// Removes a tag among added ones.
    pub fn remove_tag(&mut self, tag: &str) -> bool {
        self.tags.remove_exclusive(tag).is_none()
    }

    /// Removes all tags.
    pub fn clear_tags(&mut self) {
        self.tags.clear();
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
    fn add_new_with_date_filtering() {
        let date = NaiveDate::from_ymd(2020, 9, 9);
        let filter = Filter {
            date_option: NaiveDateFilter::Since(date),
            ..Filter::default()
        };
        let order = Order::from(&filter);

        assert_eq!(order.date, Some(date));
    }

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
