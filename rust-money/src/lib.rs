//! # Money - Core Rust API
//!
//! `money` is a collection of utilities to make tracking money expenses.

pub mod ext;
pub mod order;

use ext::{Category, ExclusiveItemExt, ItemSelector, RequestFailure};
use order::{Order, TransactionState};
use serde::{Deserialize, Serialize};
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
    tags: Vec<Category>,
    resources: Vec<Category>,
    states: [ItemSelector; 3],
    orders: Vec<Order>,
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
            tags: Vec::new(),
            resources: Vec::new(),
            states: [
                ItemSelector::Selected,
                ItemSelector::Selected,
                ItemSelector::Selected,
            ],
            orders: Vec::new(),
        }
    }

    /// Adds a valid tag if it doesn't exist yet.
    pub fn add_tag(&mut self, tag: &str) -> Option<RequestFailure> {
        self.tags.add_exclusive(tag)
    }

    /// Removes a tag everywhere.
    pub fn remove_tag(&mut self, tag: &str) -> Option<RequestFailure> {
        if self.tags.remove_exclusive(tag).is_none() {
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
    pub fn toggle_tag_selection(&mut self, tag: &str) -> Option<RequestFailure> {
        self.tags.toggle_selection(tag)
    }

    /// Adds a valid resource if it doesn't exist yet.
    pub fn add_resource(&mut self, resource: &str) -> Option<RequestFailure> {
        self.resources.add_exclusive(resource)
    }

    /// Removes a resource evrywhere.
    pub fn remove_resource(&mut self, resource: &str) -> Option<RequestFailure> {
        if self.resources.remove_exclusive(resource).is_none() {
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
    pub fn toggle_resource_selection(&mut self, resource: &str) -> Option<RequestFailure> {
        self.resources.toggle_selection(resource)
    }

    // Filters in or out the selected transaction state.
    pub fn toggle_order_state_selection(&mut self, state: TransactionState) {
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
            .for_each(|item| item.1 = ItemSelector::Selected);

        self.states
            .iter_mut()
            .for_each(|state| *state = ItemSelector::Selected);
    }

    /// Sums each order amount.
    pub fn sum_orders(&self) -> f32 {
        self.orders.iter().map(|order| order.amount).sum()
    }
}

impl Account {
    /// Returns available tags.
    pub fn tags(&self) -> &Vec<Category> {
        &self.tags
    }

    /// Returns available resources.
    pub fn resources(&self) -> &Vec<Category> {
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
        let filter_selected = |vector: &Vec<Category>| {
            vector
                .iter()
                .filter_map(|item| {
                    if let ItemSelector::Selected = item.1 {
                        Some(item.0.clone())
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
    use chrono::NaiveDate;

    mod account {
        use super::*;

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
            let mut filters: Vec<Category> = Vec::new();
            resources.iter().for_each(|resource| {
                filters.push(Category {
                    0: resource.clone(),
                    1: ItemSelector::Selected,
                });
            });
            let mut account = Account {
                resources: filters,
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
            let mut filters: Vec<Category> = Vec::new();
            tags.iter().for_each(|tag| {
                filters.push(Category {
                    0: tag.clone(),
                    1: ItemSelector::Selected,
                });
            });
            let mut account = Account {
                tags: filters,
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

            let mut filters: Vec<Category> = Vec::new();
            resources.iter().for_each(|resource| {
                filters.push(Category {
                    0: resource.clone(),
                    1: if resource == &resources[0] {
                        ItemSelector::Selected
                    } else {
                        ItemSelector::Discarded
                    },
                });
            });

            let account = Account {
                resources: filters,
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

            let mut filters: Vec<Category> = Vec::new();
            tags.iter().for_each(|tag| {
                filters.push(Category {
                    0: tag.clone(),
                    1: if tag == &tags[0] {
                        ItemSelector::Selected
                    } else {
                        ItemSelector::Discarded
                    },
                });
            });

            let account = Account {
                tags: filters,
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
            let resources = [
                "Bank I".to_string(),
                "Cash".to_string(),
                "Bank II".to_string(),
                "Vacation Check".to_string(),
                "Gift Card".to_string(),
            ];
            let tags = [
                "Food".to_string(),
                "Service".to_string(),
                "Video Games".to_string(),
                "Transport".to_string(),
                "My Awesome Car".to_string(),
                "Credits".to_string(),
                "House".to_string(),
                "Mum & Dad".to_string(),
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
            saved_account.orders[1].set_resource(resources[1].as_str(), &resources);
            saved_account.orders[1].add_tag(tags[1].as_str(), &tags);
            saved_account.orders[1].add_tag(tags[2].as_str(), &tags);
            saved_account.orders[1].amount = 14.99;
            saved_account.orders[1].set_state(TransactionState::Done);

            saved_account.add_order();
            saved_account.orders[2].description = "Loan".into();
            saved_account.orders[2].set_resource(resources[1].as_str(), &resources);
            saved_account.orders[2].add_tag(tags[5].as_str(), &tags);
            saved_account.orders[2].add_tag(tags[6].as_str(), &tags);
            saved_account.orders[2].amount = -600.00;
            saved_account.orders[2].set_state(TransactionState::Pending);
            saved_account.orders[2].date = Some(NaiveDate::from_ymd(2020, 10, 2));

            saved_account.add_order();
            saved_account.orders[3].description = "My anniversary".into();
            saved_account.orders[3].set_resource(resources[4].as_str(), &resources);
            saved_account.orders[3].add_tag(tags[7].as_str(), &tags);
            saved_account.orders[3].amount = 50.00;
            saved_account.orders[3].set_state(TransactionState::Pending);
            saved_account.orders[3].date = Some(NaiveDate::from_ymd(2020, 10, 11));

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
