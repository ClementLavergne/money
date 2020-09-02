//! # Money - Core Rust API
//!
//! `money` is a collection of utilities to make tracking money expenses.

pub mod order;

use order::{Order, TransactionState};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
#[cfg(feature = "wasmbind")]
use wasm_bindgen::prelude::*;

/// Manages account data.
#[cfg_attr(feature = "wasmbind", wasm_bindgen)]
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Account {
    tags: BTreeMap<String, ItemSelector>,
    resources: BTreeMap<String, ItemSelector>,
    states: [ItemSelector; 3],
    orders: Vec<Order>,
}

/// Stores current state of a given filter parameter.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum ItemSelector {
    Discarded,
    Selected,
}

/// Defines error types.
#[cfg_attr(feature = "wasmbind", wasm_bindgen)]
#[derive(PartialEq, Debug)]
pub enum RequestFailure {
    /// User input is incorrect.
    IncorrectArgument,
    /// User input is empty.
    EmptyArgument,
    /// Specified item can not be removed as it does not exist.
    UnknownItem,
    /// Specified item can not be added as it already did.
    ExistingItem,
}

impl ItemSelector {
    /// Toggles the state.
    fn toggle(&mut self) {
        *self = match *self {
            ItemSelector::Discarded => ItemSelector::Selected,
            ItemSelector::Selected => ItemSelector::Discarded,
        };
    }

    /// Forces the state to `Selected`.
    fn clear(&mut self) {
        *self = ItemSelector::Selected;
    }
}

/// `wasm_bindgen` compatible functions.
#[cfg_attr(feature = "wasmbind", wasm_bindgen)]
impl Account {
    /// Instantiates a new object.
    #[cfg_attr(feature = "wasmbind", wasm_bindgen(constructor))]
    pub fn create() -> Account {
        #[cfg(feature = "wasmbind")]
        console_error_panic_hook::set_once();

        Account {
            tags: BTreeMap::new(),
            resources: BTreeMap::new(),
            states: [
                ItemSelector::Selected,
                ItemSelector::Selected,
                ItemSelector::Selected,
            ],
            orders: Vec::new(),
        }
    }

    /// Adds a new tag.
    pub fn add_tag(&mut self, tag: &str) -> Option<RequestFailure> {
        if !tag.is_empty() {
            if !tag.chars().all(char::is_whitespace) {
                if self
                    .tags
                    .insert(tag.to_string(), ItemSelector::Selected)
                    .is_none()
                {
                    None
                } else {
                    Some(RequestFailure::ExistingItem)
                }
            } else {
                Some(RequestFailure::IncorrectArgument)
            }
        } else {
            Some(RequestFailure::EmptyArgument)
        }
    }

    /// Removes a tag.
    pub fn remove_tag(&mut self, tag: &str) -> Option<RequestFailure> {
        if self.tags.remove(tag).is_some() {
            // Remove related tag from orders
            self.orders.iter_mut().for_each(|x| {
                x.remove_tag(tag);
            });
            None
        } else {
            Some(RequestFailure::UnknownItem)
        }
    }

    // Filters in or out the selected tag.
    pub fn toggle_tag(&mut self, tag: &str) {
        if let Some(value) = self.tags.get_mut(tag) {
            value.toggle();
        }
    }

    /// Adds a new resource.
    pub fn add_resource(&mut self, resource: &str) -> Option<RequestFailure> {
        if !resource.is_empty() {
            if !resource.chars().all(char::is_whitespace) {
                if self
                    .resources
                    .insert(resource.to_string(), ItemSelector::Selected)
                    .is_none()
                {
                    None
                } else {
                    Some(RequestFailure::ExistingItem)
                }
            } else {
                Some(RequestFailure::IncorrectArgument)
            }
        } else {
            Some(RequestFailure::EmptyArgument)
        }
    }

    /// Removes a resource.
    pub fn remove_resource(&mut self, resource: &str) -> Option<RequestFailure> {
        if self.resources.remove(resource).is_some() {
            // Remove related resource from orders
            self.orders.iter_mut().for_each(|x| {
                if x.resource() == &Some(resource.to_string()) {
                    x.clear_resource();
                }
            });
            None
        } else {
            Some(RequestFailure::UnknownItem)
        }
    }

    // Filters in or out the selected resource.
    pub fn toggle_resource(&mut self, resource: &str) {
        if let Some(value) = self.resources.get_mut(resource) {
            value.toggle();
        }
    }

    // Filters in or out the selected transaction state.
    pub fn toggle_state(&mut self, state: TransactionState) {
        self.states[state as usize].toggle();
    }

    /// Creates a default order.
    pub fn add_order(&mut self) {
        self.orders.push(Order::default());
    }

    /// Deletes all hidden orders.
    pub fn delete_hidden_orders(&mut self) {
        self.orders.retain(|x| x.visible);
    }

    /// Resets all filter selection.
    pub fn clear_filters(&mut self) {
        self.tags
            .iter_mut()
            .chain(self.resources.iter_mut())
            .for_each(|(_, state)| state.clear());

        self.states.iter_mut().for_each(|state| state.clear());
    }

    /// Sums each order amount.
    pub fn sum_orders(&self) -> f32 {
        self.orders.iter().map(|order| order.amount).sum()
    }
}

impl Account {
    /// Returns available tags.
    pub fn tags(&self) -> &BTreeMap<String, ItemSelector> {
        &self.tags
    }

    /// Returns available resources.
    pub fn resources(&self) -> &BTreeMap<String, ItemSelector> {
        &self.resources
    }

    /// Offers access to a given order
    pub fn get_order_mut(&mut self, index: usize) -> Option<&mut Order> {
        self.orders.get_mut(index)
    }

    /// Returns all orders
    pub fn orders(&self) -> &[Order] {
        self.orders.as_slice()
    }

    /// Returns selected orders with their associated id.
    pub fn filtered_orders(&self) -> Vec<(usize, &Order)> {
        let filter_selected = |hash: &BTreeMap<String, ItemSelector>| {
            hash.iter()
                .filter_map(|(key, value)| {
                    if let ItemSelector::Selected = value {
                        Some(key.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>()
        };

        let selected_resources = filter_selected(&self.resources);
        let selected_tags = filter_selected(&self.tags);

        self.orders
            .iter()
            .enumerate()
            .filter(|(_, order)| {
                // Discard removed orders
                if order.visible {
                    // Make sure that at least one tag is among allowed ones
                    let tag_match = if selected_tags.is_empty() {
                        order.tags().is_empty()
                    } else {
                        order.tags().iter().any(|tag| selected_tags.contains(tag))
                    };

                    // Make sure the current state is among allowed ones
                    let state_match =
                        if let ItemSelector::Selected = self.states[order.state() as usize] {
                            true
                        } else {
                            false
                        };

                    // Make sure the resource is part of allowed ones
                    let resource_match = if let Some(resource) = order.resource() {
                        selected_resources.contains(&resource)
                    } else {
                        selected_resources.is_empty()
                    };

                    tag_match && resource_match && state_match
                } else {
                    false
                }
            })
            .collect()
    }

    /// Stores data as YAML file.
    pub fn save_file(&self, path: &Path) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(serde_yaml::to_string(self).unwrap().as_bytes())?;
        Ok(())
    }

    /// Returns an initialized account from YAML file.
    pub fn load_file(path: &Path) -> std::io::Result<Account> {
        let file = File::open(path)?;

        match Account::try_from(file) {
            Ok(data) => Ok(data),
            Err(error) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("{}", error),
            )),
        }
    }
}

impl TryFrom<File> for Account {
    type Error = serde_yaml::Error;

    fn try_from(file: File) -> Result<Self, Self::Error> {
        serde_yaml::from_reader(file)
    }
}

impl TryFrom<&str> for Account {
    type Error = serde_yaml::Error;

    fn try_from(content: &str) -> Result<Self, Self::Error> {
        serde_yaml::from_str(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod account {
        use super::*;

        #[test]
        fn add_valid_resource() {
            let resources = [
                "Bank 1".to_string(),
                "Bank 2".to_string(),
                "Cash".to_string(),
            ];
            let mut account = Account {
                resources: resources[..2]
                    .iter()
                    .map(|resource| (resource.clone(), ItemSelector::Selected))
                    .collect(),
                ..Account::create()
            };

            assert_eq!(account.add_resource(&resources[2]), None);
            assert_eq!(
                account.resources,
                resources
                    .iter()
                    .map(|resource| (resource.clone(), ItemSelector::Selected))
                    .collect()
            );
        }

        #[test]
        fn remove_known_resource() {
            let resources = [
                "Bank 1".to_string(),
                "Bank 2".to_string(),
                "Cash".to_string(),
            ];
            let mut account = Account {
                resources: resources
                    .iter()
                    .map(|resource| (resource.clone(), ItemSelector::Selected))
                    .collect(),
                ..Account::create()
            };

            assert_eq!(account.remove_resource(&resources[2]), None);
            assert_eq!(
                account.resources,
                resources[..2]
                    .iter()
                    .map(|resource| (resource.clone(), ItemSelector::Selected))
                    .collect()
            );
        }

        #[test]
        fn discard_adding_existing_resource() {
            let resources = [
                "Bank 1".to_string(),
                "Bank 2".to_string(),
                "Cash".to_string(),
            ];
            let mut account = Account {
                resources: resources
                    .iter()
                    .map(|resource| (resource.clone(), ItemSelector::Selected))
                    .collect(),
                ..Account::create()
            };

            assert_eq!(
                account.add_resource(&resources[2]),
                Some(RequestFailure::ExistingItem)
            );
            assert_eq!(
                account.resources,
                resources
                    .iter()
                    .map(|resource| (resource.clone(), ItemSelector::Selected))
                    .collect()
            );
        }

        #[test]
        fn discard_adding_empty_resource() {
            let mut account = Account::create();

            assert_eq!(
                account.add_resource(""),
                Some(RequestFailure::EmptyArgument)
            );
            assert_eq!(account.resources, BTreeMap::new());
        }

        #[test]
        fn discard_adding_incorrect_resource() {
            let mut account = Account::create();

            assert_eq!(
                account.add_resource(" "),
                Some(RequestFailure::IncorrectArgument)
            );
            assert_eq!(account.tags, BTreeMap::new());
        }

        #[test]
        fn discard_removing_unknown_resource() {
            let resources = [
                "Bank 1".to_string(),
                "Bank 2".to_string(),
                "Cash".to_string(),
            ];
            let mut account = Account {
                resources: resources
                    .iter()
                    .map(|resource| (resource.clone(), ItemSelector::Selected))
                    .collect(),
                ..Account::create()
            };

            assert_eq!(
                account.remove_tag("Unknown tag!"),
                Some(RequestFailure::UnknownItem)
            );
            assert_eq!(
                account.resources,
                resources
                    .iter()
                    .map(|resource| (resource.clone(), ItemSelector::Selected))
                    .collect()
            );
        }

        #[test]
        fn add_valid_tag() {
            let tags = [
                "Food".to_string(),
                "Service".to_string(),
                "Games".to_string(),
            ];
            let mut account = Account {
                tags: tags[..2]
                    .iter()
                    .map(|tag| (tag.clone(), ItemSelector::Selected))
                    .collect(),
                ..Account::create()
            };

            assert_eq!(account.add_tag(&tags[2]), None);
            assert_eq!(
                account.tags,
                tags.iter()
                    .map(|tag| (tag.clone(), ItemSelector::Selected))
                    .collect()
            );
        }

        #[test]
        fn remove_known_tag() {
            let tags = [
                "Food".to_string(),
                "Service".to_string(),
                "Games".to_string(),
            ];
            let mut account = Account {
                tags: tags
                    .iter()
                    .map(|tag| (tag.clone(), ItemSelector::Selected))
                    .collect(),
                ..Account::create()
            };

            assert_eq!(account.remove_tag(&tags[2]), None);
            assert_eq!(
                account.tags,
                tags[..2]
                    .iter()
                    .map(|tag| (tag.clone(), ItemSelector::Selected))
                    .collect()
            );
        }

        #[test]
        fn discard_adding_existing_tag() {
            let tags = ["Food".to_string(), "Service".to_string()];
            let mut account = Account {
                tags: tags
                    .iter()
                    .map(|tag| (tag.clone(), ItemSelector::Selected))
                    .collect(),
                ..Account::create()
            };

            assert_eq!(
                account.add_tag(&tags[1]),
                Some(RequestFailure::ExistingItem)
            );
            assert_eq!(
                account.tags,
                tags.iter()
                    .map(|tag| (tag.clone(), ItemSelector::Selected))
                    .collect()
            );
        }

        #[test]
        fn discard_adding_empty_tag() {
            let mut account = Account::create();

            assert_eq!(account.add_tag(""), Some(RequestFailure::EmptyArgument));
            assert_eq!(account.tags, BTreeMap::new());
        }

        #[test]
        fn discard_adding_incorrect_tag() {
            let mut account = Account::create();

            assert_eq!(
                account.add_tag(" "),
                Some(RequestFailure::IncorrectArgument)
            );
            assert_eq!(account.tags, BTreeMap::new());
        }

        #[test]
        fn discard_removing_unknown_tag() {
            let tags = ["Food".to_string(), "Service".to_string()];
            let mut account = Account {
                tags: tags
                    .iter()
                    .map(|tag| (tag.clone(), ItemSelector::Selected))
                    .collect(),
                ..Account::create()
            };

            assert_eq!(
                account.remove_tag("Unknown tag!"),
                Some(RequestFailure::UnknownItem)
            );
            assert_eq!(
                account.tags,
                tags.iter()
                    .map(|tag| (tag.clone(), ItemSelector::Selected))
                    .collect()
            );
        }

        #[test]
        fn remove_resource_used_by_orders() {
            let mut expected_orders = [
                Order::default(),
                Order::default(),
                Order::default(),
                Order::default(),
            ];
            let resources = [
                String::from("Bank"),
                String::from("Cash"),
                String::from("Gift Check"),
            ];
            let mut hashmap: BTreeMap<String, ItemSelector> = BTreeMap::new();
            resources.iter().for_each(|resource| {
                hashmap.insert(resource.clone(), ItemSelector::Selected);
            });
            let mut account = Account {
                resources: hashmap,
                ..Account::create()
            };

            account.add_order();
            account.orders[0].set_resource(resources[0].as_str(), &resources);
            account.add_order();
            account.orders[1].set_resource(resources[1].as_str(), &resources);
            account.add_order();
            account.orders[2].set_resource(resources[1].as_str(), &resources);
            account.add_order();
            account.orders[3].set_resource(resources[2].as_str(), &resources);

            assert_eq!(account.remove_resource(resources[1].as_str()), None);

            expected_orders[0].set_resource(resources[0].as_str(), &resources);
            expected_orders[3].set_resource(resources[2].as_str(), &resources);
            assert_eq!(account.orders(), expected_orders);
        }

        #[test]
        fn remove_tag_used_by_orders() {
            let mut expected_orders = [Order::default(), Order::default()];
            let tags = [
                String::from("Food"),
                String::from("Service"),
                String::from("Video Games"),
                String::from("Transport"),
            ];
            let mut hashmap: BTreeMap<String, ItemSelector> = BTreeMap::new();
            tags.iter().for_each(|tag| {
                hashmap.insert(tag.clone(), ItemSelector::Selected);
            });
            let mut account = Account {
                tags: hashmap,
                ..Account::create()
            };

            account.add_order();
            account.orders[0].add_tag(tags[0].as_str(), &tags);
            account.orders[0].add_tag(tags[1].as_str(), &tags);
            account.add_order();
            account.orders[1].add_tag(tags[2].as_str(), &tags);
            account.orders[1].add_tag(tags[1].as_str(), &tags);

            expected_orders[0].add_tag(tags[0].as_str(), &tags);
            expected_orders[0].add_tag(tags[1].as_str(), &tags);
            expected_orders[1].add_tag(tags[2].as_str(), &tags);
            expected_orders[1].add_tag(tags[1].as_str(), &tags);
            assert_eq!(account.remove_tag(tags[1].as_str()), None);

            expected_orders[0].remove_tag(tags[1].as_str());
            expected_orders[1].remove_tag(tags[1].as_str());
            assert_eq!(account.orders(), expected_orders);
        }

        #[test]
        fn export_orders() {
            let expected_orders = [Order::default(), Order::default(), Order::default()];
            let account = Account {
                orders: expected_orders.to_vec(),
                ..Account::create()
            };

            assert_eq!(account.orders(), expected_orders);
        }

        #[test]
        fn filter_orders_by_visibility() {
            let mut expected_orders = [Order::default(), Order::default(), Order::default()];
            expected_orders[0].visible = false;

            let account = Account {
                orders: expected_orders.to_vec(),
                ..Account::create()
            };

            assert_eq!(
                account.filtered_orders(),
                expected_orders
                    .to_vec()
                    .iter()
                    .enumerate()
                    .filter(|(_, value)| value.visible)
                    .collect::<Vec<(usize, &Order)>>()
            );
        }

        #[test]
        fn filter_orders_by_resources() {
            let resources = vec!["Bank".to_string(), "Cash".to_string()];
            let mut expected_orders = [Order::default(), Order::default(), Order::default()];
            expected_orders[0].set_resource(resources[0].as_str(), &resources);
            expected_orders[1].set_resource(resources[1].as_str(), &resources);
            expected_orders[2].set_resource(resources[0].as_str(), &resources);

            let mut hashmap: BTreeMap<String, ItemSelector> = BTreeMap::new();
            resources.iter().for_each(|resource| {
                hashmap.insert(
                    resource.clone(),
                    if resource == &resources[0] {
                        ItemSelector::Selected
                    } else {
                        ItemSelector::Discarded
                    },
                );
            });

            let account = Account {
                resources: hashmap,
                orders: expected_orders.to_vec(),
                ..Account::create()
            };

            assert_eq!(
                account.filtered_orders(),
                expected_orders
                    .to_vec()
                    .iter()
                    .enumerate()
                    .filter(|(_, value)| value.resource() == &Some(resources[0].clone()))
                    .collect::<Vec<(usize, &Order)>>()
            );
        }

        #[test]
        fn filter_orders_by_tags() {
            let tags = vec!["Car".to_string(), "Sport".to_string(), "Games".to_string()];
            let mut expected_orders = [Order::default(), Order::default(), Order::default()];
            expected_orders[0].add_tag(tags[0].as_str(), &tags);
            expected_orders[0].add_tag(tags[1].as_str(), &tags);
            expected_orders[1].add_tag(tags[1].as_str(), &tags);
            expected_orders[2].add_tag(tags[2].as_str(), &tags);
            expected_orders[2].add_tag(tags[1].as_str(), &tags);
            expected_orders[2].add_tag(tags[0].as_str(), &tags);

            let mut hashmap: BTreeMap<String, ItemSelector> = BTreeMap::new();
            tags.iter().for_each(|tag| {
                hashmap.insert(
                    tag.clone(),
                    if tag == &tags[0] || tag == &tags[2] {
                        ItemSelector::Selected
                    } else {
                        ItemSelector::Discarded
                    },
                );
            });

            let account = Account {
                tags: hashmap,
                orders: expected_orders.to_vec(),
                ..Account::create()
            };

            assert_eq!(
                account.filtered_orders(),
                expected_orders
                    .to_vec()
                    .iter()
                    .enumerate()
                    .filter(|(_, value)| value.tags().contains(&tags[0])
                        || value.tags().contains(&tags[2]))
                    .collect::<Vec<(usize, &Order)>>()
            );
        }

        #[test]
        fn save_load_data() {
            let mut saved_account = Account::create();
            let resources = ["Bank".to_string(), "Cash".to_string()];
            let tags = [
                "Food".to_string(),
                "Service".to_string(),
                "Video Games".to_string(),
                "Transport".to_string(),
                "Car".to_string(),
            ];

            resources.iter().for_each(|resource| {
                saved_account.add_resource(resource);
            });
            tags.iter().for_each(|tag| {
                saved_account.add_tag(tag);
            });

            saved_account.add_order();
            saved_account.orders[0].description = "Gazoline".into();
            saved_account.orders[0].set_resource(resources[0].as_str(), &resources);
            saved_account.orders[0].add_tag(tags[3].as_str(), &tags);
            saved_account.orders[0].add_tag(tags[4].as_str(), &tags);
            saved_account.orders[0].amount = 62.5;
            saved_account.orders[0].set_state(TransactionState::InProgress);

            saved_account.add_order();
            saved_account.orders[1].description = "GamePass Ultimate".into();
            saved_account.orders[1].set_resource(resources[0].as_str(), &resources);
            saved_account.orders[1].add_tag(tags[1].as_str(), &tags);
            saved_account.orders[1].add_tag(tags[2].as_str(), &tags);
            saved_account.orders[1].amount = 14.99;
            saved_account.orders[1].set_state(TransactionState::Done);

            // Serialize over a file
            if let Err(error) = saved_account.save_file(Path::new("data.yml")) {
                println!("{}", error);
            }

            // Load previously generated file
            let mut loaded_account = Account::load_file(Path::new("data.yml")).unwrap();
            assert_eq!(loaded_account, saved_account);
            loaded_account = Account::try_from(File::open("data.yml").unwrap()).unwrap();

            assert_eq!(loaded_account, saved_account);
        }
    }
}
