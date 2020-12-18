//! # Money - Core Rust API
//!
//! `money` is a collection of utilities to make tracking money expenses.

pub mod ext;
pub mod filter;
pub mod order;

use ext::{ExclusiveItemExt, RequestFailure};
use order::Order;
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
    label: String,
    tags: Vec<String>,
    resources: Vec<String>,
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
            label: "MONEY".into(),
            tags: Vec::new(),
            resources: Vec::new(),
            orders: Vec::new(),
        }
    }

    /// Update the label of the account.
    pub fn set_label(&mut self, label: &str) {
        self.label = label.into();
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

    /// Adds a valid resource if it doesn't exist yet.
    pub fn add_resource(&mut self, resource: &str) -> Option<RequestFailure> {
        self.resources.add_exclusive(resource)
    }

    /// Removes a resource evrywhere.
    pub fn remove_resource(&mut self, resource: &str) -> Option<RequestFailure> {
        if self.resources.remove_exclusive(resource).is_none() {
            // Remove related resource from orders
            self.orders.iter_mut().for_each(|x| {
                if x.resource == Some(resource.to_string()) {
                    x.resource = None;
                }
            });
            None
        } else {
            Some(RequestFailure::UnknownItem)
        }
    }

    /// Creates a default order.
    pub fn add_order(&mut self) {
        self.orders.push(Order::default());
    }

    /// duplicates an existing order and returns its id.
    pub fn duplicate_order(&mut self, index: usize) -> bool {
        // Copy the order if it exists
        if let Some(order) = self.orders.get(index) {
            let copy = order.clone();
            self.orders.push(copy);
            true
        } else {
            false
        }
    }

    /// Deletes one order permanently.
    pub fn delete_order(&mut self, index: usize) -> bool {
        if self.orders.get(index).is_some() {
            self.orders.remove(index);
            true
        } else {
            false
        }
    }
}

impl Account {
    /// Returns the label.
    pub fn label(&self) -> &String {
        &self.label
    }

    /// Returns available tags.
    pub fn tags(&self) -> &Vec<String> {
        &self.tags
    }

    /// Returns available resources.
    pub fn resources(&self) -> &Vec<String> {
        &self.resources
    }

    /// Offers access to a given order
    pub fn get_order_mut(&mut self, index: usize) -> Option<&mut Order> {
        self.orders.get_mut(index)
    }

    /// Returns all orders
    pub fn orders(&self) -> &Vec<Order> {
        &self.orders
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
        use ext::OrderListExt;
        use filter::category::{Category, CategoryFilter};
        use filter::date::NaiveDateFilter;
        use filter::{Filter, ItemSelector, VisibilityFilter};
        use order::TransactionState;

        #[test]
        fn duplicate_existing_order() {
            let mut account = Account {
                orders: vec![
                    Order::default(),
                    Order {
                        description: "Test".into(),
                        amount: -2.99,
                        ..Order::default()
                    },
                ],
                ..Account::create()
            };

            assert_eq!(account.duplicate_order(2), false);
            assert_eq!(account.duplicate_order(1), true);
            assert_eq!(account.duplicate_order(0), true);
            assert_eq!(account.orders[1], account.orders[2]);
            assert_eq!(account.orders[0], account.orders[3]);
        }

        #[test]
        fn remove_resource_used_by_orders() {
            let resources = [
                String::from("Bank"),
                String::from("Cash"),
                String::from("Gift Check"),
            ];
            let mut orders = [
                Order {
                    resource: Some(resources[0].clone()),
                    ..Order::default()
                },
                Order {
                    resource: Some(resources[1].clone()),
                    ..Order::default()
                },
                Order {
                    resource: Some(resources[1].clone()),
                    ..Order::default()
                },
                Order {
                    resource: Some(resources[2].clone()),
                    ..Order::default()
                },
            ];
            let mut account = Account {
                resources: resources.to_vec(),
                orders: orders.to_vec(),
                ..Account::create()
            };

            orders[1].resource = None;
            orders[2].resource = None;
            assert_eq!(account.remove_resource(resources[1].as_str()), None);
            assert_eq!(account.orders, orders);
        }

        #[test]
        fn remove_tag_used_by_orders() {
            let tags = [
                String::from("Food"),
                String::from("Service"),
                String::from("Video Games"),
                String::from("Transport"),
            ];
            let mut orders = [
                Order {
                    tags: tags[..2].to_vec(),
                    ..Order::default()
                },
                Order {
                    tags: tags[1..3].to_vec(),
                    ..Order::default()
                },
            ];
            let mut account = Account {
                tags: tags.to_vec(),
                orders: orders.to_vec(),
                ..Account::create()
            };

            orders[0].tags.remove(1);
            orders[1].tags.remove(0);
            assert_eq!(account.remove_tag(tags[1].as_str()), None);
            assert_eq!(account.orders, orders);
        }

        #[test]
        fn export_orders() {
            let expected_orders = [Order::default(), Order::default(), Order::default()];
            let account = Account {
                orders: expected_orders.to_vec(),
                ..Account::create()
            };

            assert_eq!(account.orders, expected_orders);
        }

        #[test]
        fn filter_orders() {
            let resources = [
                Category("Bank".to_string(), ItemSelector::Discarded),
                Category("Cash".to_string(), ItemSelector::Selected),
            ];
            let tags = [
                Category("Home".to_string(), ItemSelector::Selected),
                Category("Sport".to_string(), ItemSelector::Discarded),
                Category("Gift".to_string(), ItemSelector::Selected),
                Category("Insurance".to_string(), ItemSelector::Selected),
            ];
            let orders = [
                (
                    0,
                    &Order {
                        date: Some(NaiveDate::from_ymd(2020, 4, 15)),
                        resource: None,
                        tags: tags.iter().map(|x| x.0.clone()).collect::<Vec<String>>(),
                        state: TransactionState::Pending,
                        visible: true,
                        ..Order::default()
                    },
                ),
                (
                    1,
                    &Order {
                        date: None,
                        resource: Some(resources[0].0.clone()),
                        tags: tags[..2]
                            .iter()
                            .map(|x| x.0.clone())
                            .collect::<Vec<String>>(),
                        state: TransactionState::InProgress,
                        visible: true,
                        ..Order::default()
                    },
                ),
                (
                    2,
                    &Order {
                        date: Some(NaiveDate::from_ymd(2019, 3, 3)),
                        resource: Some(resources[1].0.clone()),
                        tags: tags[1..]
                            .iter()
                            .map(|x| x.0.clone())
                            .collect::<Vec<String>>(),
                        state: TransactionState::Done,
                        visible: false,
                        ..Order::default()
                    },
                ),
                (
                    3,
                    &Order {
                        date: Some(NaiveDate::from_ymd(2020, 5, 20)),
                        resource: Some(resources[0].0.clone()),
                        tags: vec![tags[3].0.clone()],
                        state: TransactionState::Done,
                        visible: true,
                        ..Order::default()
                    },
                ),
                (
                    4,
                    &Order {
                        date: Some(NaiveDate::from_ymd(2021, 5, 30)),
                        resource: Some(resources[1].0.clone()),
                        tags: Vec::new(),
                        state: TransactionState::Pending,
                        visible: true,
                        ..Order::default()
                    },
                ),
            ];
            let account = Account {
                resources: resources
                    .iter()
                    .map(|x| x.0.clone())
                    .collect::<Vec<String>>(),
                orders: orders.iter().map(|x| x.1.clone()).collect::<Vec<Order>>(),
                ..Account::create()
            };
            let filter_1 = Filter {
                visibility: VisibilityFilter::VisibleOnly,
                date_option: NaiveDateFilter::Between(
                    NaiveDate::from_ymd(2020, 3, 14),
                    NaiveDate::from_ymd(2020, 5, 24),
                ),
                ..Filter::default()
            };
            let filter_2 = Filter {
                tag_option: CategoryFilter::Enabled(vec![tags[1].clone()]),
                resource_option: CategoryFilter::Enabled(resources.to_vec()),
                state_option: [
                    ItemSelector::Selected,
                    ItemSelector::Discarded,
                    ItemSelector::Selected,
                ],
                ..Filter::default()
            };

            assert_eq!(
                account.orders.apply_filter(&filter_1),
                vec![orders[0], orders[3]]
            );
            assert_eq!(account.orders.apply_filter(&filter_2), vec![orders[4]]);
        }

        #[test]
        fn save_load_data() {
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
                "Work".to_string(),
            ];
            let mut saved_account = Account {
                label: "A year of wonderful things! 🙏".into(),
                resources: resources.to_vec(),
                tags: tags.to_vec(),
                orders: vec![
                    Order {
                        description: "Initial amount".into(),
                        date: Some(NaiveDate::from_ymd(2020, 1, 1)),
                        resource: Some(resources[0].clone()),
                        tags: Vec::new(),
                        amount: 1000.0,
                        state: TransactionState::Done,
                        visible: true,
                    },
                    Order {
                        description: "Initial amount".into(),
                        date: Some(NaiveDate::from_ymd(2020, 1, 1)),
                        resource: Some(resources[1].clone()),
                        tags: Vec::new(),
                        amount: 53.5,
                        state: TransactionState::Done,
                        visible: true,
                    },
                    Order {
                        description: "Initial amount".into(),
                        date: Some(NaiveDate::from_ymd(2020, 1, 1)),
                        resource: Some(resources[2].clone()),
                        tags: Vec::new(),
                        amount: 250.0,
                        state: TransactionState::Done,
                        visible: true,
                    },
                    Order {
                        description: "My Anniversary 🎂".into(),
                        date: Some(NaiveDate::from_ymd(2020, 11, 10)),
                        resource: Some(resources[1].clone()),
                        tags: vec![tags[7].clone()],
                        amount: 50.0,
                        state: TransactionState::Pending,
                        visible: true,
                    },
                    Order {
                        description: "Gift".into(),
                        date: Some(NaiveDate::from_ymd(2020, 6, 20)),
                        resource: Some(resources[4].clone()),
                        tags: vec![tags[7].clone()],
                        amount: 50.0,
                        state: TransactionState::Pending,
                        visible: true,
                    },
                    Order {
                        description: "Restaurant".into(),
                        date: Some(NaiveDate::from_ymd(2020, 3, 4)),
                        resource: Some(resources[1].clone()),
                        tags: vec![tags[0].clone()],
                        amount: -44.7,
                        state: TransactionState::InProgress,
                        visible: true,
                    },
                    Order {
                        description: "Metro".into(),
                        date: Some(NaiveDate::from_ymd(2020, 3, 4)),
                        resource: Some(resources[1].clone()),
                        tags: vec![tags[3].clone()],
                        amount: -12.99,
                        state: TransactionState::Done,
                        visible: true,
                    },
                    Order {
                        description: "Music".into(),
                        date: Some(NaiveDate::from_ymd(2020, 3, 10)),
                        resource: Some(resources[0].clone()),
                        tags: vec![tags[1].clone()],
                        amount: -13.99,
                        state: TransactionState::InProgress,
                        visible: true,
                    },
                    Order {
                        description: "Music II".into(),
                        date: Some(NaiveDate::from_ymd(2020, 3, 10)),
                        resource: Some(resources[3].clone()),
                        tags: vec![tags[1].clone(), tags[7].clone()],
                        amount: -13.99,
                        state: TransactionState::InProgress,
                        visible: true,
                    },
                ],
            };

            (1..=12).for_each(|month| {
                let order_state = if month <= 3 {
                    TransactionState::Done
                } else {
                    TransactionState::Pending
                };

                saved_account.orders.push(Order {
                    description: "Salary".into(),
                    date: Some(NaiveDate::from_ymd(2020, month, 3)),
                    resource: Some(resources[0].clone()),
                    tags: vec![tags[8].clone()],
                    amount: 2500.0,
                    state: order_state,
                    visible: true,
                });
                saved_account.orders.push(Order {
                    description: "Loan".into(),
                    date: Some(NaiveDate::from_ymd(2020, month, 6)),
                    resource: Some(resources[0].clone()),
                    tags: tags[5..=6].to_vec(),
                    amount: -600.0,
                    state: order_state,
                    visible: true,
                });
                saved_account.orders.push(Order {
                    description: "GamePass Ultimate".into(),
                    date: Some(NaiveDate::from_ymd(2020, month, 15)),
                    resource: Some(resources[2].clone()),
                    tags: tags[1..=2].to_vec(),
                    amount: -14.99,
                    state: order_state,
                    visible: true,
                });
                saved_account.orders.push(Order {
                    description: "Transfert".into(),
                    date: Some(NaiveDate::from_ymd(2020, month, 25)),
                    resource: Some(resources[0].clone()),
                    tags: Vec::new(),
                    amount: -20.0,
                    state: order_state,
                    visible: true,
                });
                saved_account.orders.push(Order {
                    description: "Transfert".into(),
                    date: Some(NaiveDate::from_ymd(2020, month, 25)),
                    resource: Some(resources[2].clone()),
                    tags: Vec::new(),
                    amount: 20.0,
                    state: order_state,
                    visible: true,
                });
                saved_account.orders.push(Order {
                    description: "Gazoline".into(),
                    date: Some(NaiveDate::from_ymd(2020, month, 23)),
                    resource: Some(resources[0].clone()),
                    tags: tags[3..=5].to_vec(),
                    amount: -62.5,
                    state: order_state,
                    visible: true,
                });
            });

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
