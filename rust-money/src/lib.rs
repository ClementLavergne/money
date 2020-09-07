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

/// Manages account data.
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
#[derive(PartialEq, Debug)]
pub enum RequestFailure {
    /// User input is empty.
    IncorrectArgument,
    /// Specified item can not be removed as it does not exist.
    UnknownItem,
    /// Specified item can not be added as it already did.
    ExistingItem,
    /// Incorrect index argument.
    OutOfBoundIndex,
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

impl Account {
    /// Instantiates a new object.
    pub fn create() -> Account {
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

    /// Returns available tags.
    pub fn tags(&self) -> &BTreeMap<String, ItemSelector> {
        &self.tags
    }

    /// Adds a new resource.
    pub fn add_resource(&mut self, resource: &str) -> Option<RequestFailure> {
        if !resource.is_empty() {
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

    /// Returns available resources.
    pub fn resources(&self) -> &BTreeMap<String, ItemSelector> {
        &self.resources
    }

    // Filters in or out the selected transaction state.
    pub fn toggle_state(&mut self, state: TransactionState) {
        self.states[state as usize].toggle();
    }

    /// Creates a default order.
    pub fn add_order(&mut self) {
        self.orders.push(Order::default());
    }

    /// Offers access to a given order
    pub fn get_order_mut(&mut self, index: usize) -> Option<&mut Order> {
        self.orders.get_mut(index)
    }

    /// Removes all hidden orders; in other words, it acts like a trash.
    pub fn delete_hidden_orders(&mut self) {
        self.orders.retain(|x| x.visible);
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
                    let tag_match = order.tags().iter().any(|tag| selected_tags.contains(tag));

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

    /// Resets all filter selection.
    pub fn clear_filters(&mut self) {
        self.tags
            .iter_mut()
            .chain(self.resources.iter_mut())
            .for_each(|(_, state)| state.clear());

        self.states.iter_mut().for_each(|state| state.clear());
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
