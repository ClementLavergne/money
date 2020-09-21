//! # Money - Core Rust API
//!
//! `money` is a collection of utilities to make tracking money expenses.

pub mod ext;
pub mod filter;
pub mod order;

use ext::{ExclusiveItemExt, RequestFailure};
use filter::Filter;
use filter::{NaiveDateFilter, OptionNaiveDateRange};
use order::Order;
use order::TransactionState::{Done, InProgress, Pending};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
#[cfg(feature = "wasmbind")]
use wasm_bindgen::prelude::*;
use CategoryType::{Resource, Tag};

/// Defines available category types.
#[cfg_attr(feature = "wasmbind", wasm_bindgen)]
#[derive(PartialEq, Debug)]
pub enum CategoryType {
    /// A **resource** identifies something which represents/holds money.
    Resource,
    /// A **tag** identifies a category of expense.
    /// Could be: an object, a person, a firm, .. it's up to you!
    Tag,
}

/// Gather different amounts for a *category*.
#[cfg_attr(feature = "wasmbind", wasm_bindgen)]
#[derive(PartialEq, Debug)]
pub struct CategoryAmount {
    pub current: f32,
    pub pending: f32,
    pub in_progress: f32,
    pub expected: f32,
}

/// Manages account data.
#[cfg_attr(feature = "wasmbind", wasm_bindgen)]
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Account {
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
            tags: Vec::new(),
            resources: Vec::new(),
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
    pub fn add_order(&mut self) -> usize {
        self.orders.push(Order::default());
        self.orders.len() - 1
    }

    /// Deletes all hidden orders.
    pub fn delete_hidden_orders(&mut self) {
        self.orders.retain(|x| x.visible);
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
    pub fn orders(&self) -> &[Order] {
        self.orders.as_slice()
    }

    /// Returns selected orders with their associated id.
    pub fn filtered_orders(&self, filter: &Filter) -> Vec<(usize, &Order)> {
        self.orders
            .iter()
            .enumerate()
            .filter(|(_, order)| filter.is_order_allowed(order))
            .collect()
    }

    /// Computes the different amounts of a *category* between a given range.
    pub fn get_category_amount(
        &self,
        kind: CategoryType,
        category: &str,
        date_range: OptionNaiveDateRange,
    ) -> Option<CategoryAmount> {
        let mut result = CategoryAmount {
            current: 0.0,
            pending: 0.0,
            in_progress: 0.0,
            expected: 0.0,
        };
        let mut nb_orders = 0;
        let mut update_amount = |order: &Order| {
            match order.state {
                Pending => result.pending += order.amount,
                InProgress => result.in_progress += order.amount,
                Done => result.current += order.amount,
            }

            result.expected += order.amount;
            nb_orders += 1;
        };
        let date_filter = NaiveDateFilter::from(date_range);

        match kind {
            Resource => {
                if self.resources.contains(&category.to_string()) {
                    self.orders
                        .iter()
                        .filter(|order| order.visible && order.resource == Some(category.into()))
                        .filter(|order| date_filter.is_order_allowed(order))
                        .for_each(|order| update_amount(order));

                    if nb_orders > 0 {
                        Some(result)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Tag => {
                if self.tags.contains(&category.to_string()) {
                    self.orders
                        .iter()
                        .filter(|order| order.visible && order.tags.contains(&category.to_string()))
                        .filter(|order| date_filter.is_order_allowed(order))
                        .for_each(|order| update_amount(order));

                    if nb_orders > 0 {
                        Some(result)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
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
        use filter::category::{Category, CategoryFilter};
        use filter::date::NaiveDateFilter;
        use filter::{ItemSelector, VisibilityFilter};
        use order::TransactionState;

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
                account.filtered_orders(&filter_1),
                vec![orders[0], orders[3]]
            );
            assert_eq!(account.filtered_orders(&filter_2), vec![orders[4]]);
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

    #[test]
    fn compute_overall_resource_amount() {
        let resources = [String::from("Bank"), String::from("Cash")];
        let tuples = vec![
            (resources[0].clone(), -65.4, Done),
            (resources[1].clone(), -32.83, Done),
            (resources[1].clone(), -13.99, Pending),
            (resources[1].clone(), -7.44, InProgress),
            (resources[1].clone(), 15.00, Pending),
            (resources[1].clone(), -69.99, InProgress),
            (resources[1].clone(), 7.99, Done),
        ];
        let result = CategoryAmount {
            current: tuples
                .iter()
                .filter(|x| x.0 == resources[1] && x.2 == Done)
                .fold(0.0, |acc, x| acc + x.1),
            pending: tuples
                .iter()
                .filter(|x| x.0 == resources[1] && x.2 == Pending)
                .fold(0.0, |acc, x| acc + x.1),
            in_progress: tuples
                .iter()
                .filter(|x| x.0 == resources[1] && x.2 == InProgress)
                .fold(0.0, |acc, x| acc + x.1),
            expected: tuples
                .iter()
                .filter(|x| x.0 == resources[1])
                .fold(0.0, |acc, x| acc + x.1),
        };
        let orders = tuples
            .into_iter()
            .map(|x| Order {
                resource: Some(x.0),
                amount: x.1,
                state: x.2,
                ..Order::default()
            })
            .collect::<Vec<Order>>();
        let account = Account {
            resources: resources.to_vec(),
            orders: orders.to_vec(),
            ..Account::create()
        };

        assert_eq!(
            account.get_category_amount(
                Resource,
                resources[1].as_str(),
                OptionNaiveDateRange(None, None)
            ),
            Some(result)
        );
    }

    #[test]
    fn compute_resource_amount_at_date() {
        let resources = [String::from("Bank")];
        let tuples = vec![
            (
                Some(NaiveDate::from_ymd(2020, 1, 1)),
                resources[0].clone(),
                -65.4,
                Pending,
            ),
            (
                Some(NaiveDate::from_ymd(2020, 2, 1)),
                resources[0].clone(),
                -32.83,
                InProgress,
            ),
            (
                Some(NaiveDate::from_ymd(2020, 3, 1)),
                resources[0].clone(),
                -13.99,
                Done,
            ),
            (
                Some(NaiveDate::from_ymd(2020, 4, 1)),
                resources[0].clone(),
                -7.44,
                Done,
            ),
            (
                Some(NaiveDate::from_ymd(2020, 5, 1)),
                resources[0].clone(),
                15.00,
                Pending,
            ),
            (
                Some(NaiveDate::from_ymd(2020, 6, 1)),
                resources[0].clone(),
                -69.99,
                Pending,
            ),
            (
                Some(NaiveDate::from_ymd(2020, 7, 1)),
                resources[0].clone(),
                7.99,
                Pending,
            ),
        ];
        let desired_date = NaiveDate::from_ymd(2020, 6, 12);
        let result = CategoryAmount {
            current: tuples
                .iter()
                .filter(|x| {
                    desired_date.signed_duration_since(x.0.unwrap()).num_days() >= 0 && x.3 == Done
                })
                .fold(0.0, |acc, x| acc + x.2),
            pending: tuples
                .iter()
                .filter(|x| {
                    desired_date.signed_duration_since(x.0.unwrap()).num_days() >= 0
                        && x.3 == Pending
                })
                .fold(0.0, |acc, x| acc + x.2),
            in_progress: tuples
                .iter()
                .filter(|x| {
                    desired_date.signed_duration_since(x.0.unwrap()).num_days() >= 0
                        && x.3 == InProgress
                })
                .fold(0.0, |acc, x| acc + x.2),
            expected: tuples
                .iter()
                .filter(|x| desired_date.signed_duration_since(x.0.unwrap()).num_days() >= 0)
                .fold(0.0, |acc, x| acc + x.2),
        };
        let orders = tuples
            .into_iter()
            .map(|x| Order {
                date: x.0,
                resource: Some(x.1),
                amount: x.2,
                state: x.3,
                ..Order::default()
            })
            .collect::<Vec<Order>>();
        let account = Account {
            resources: resources.to_vec(),
            orders: orders.to_vec(),
            ..Account::create()
        };

        assert_eq!(
            account.get_category_amount(
                Resource,
                resources[0].as_str(),
                OptionNaiveDateRange(None, Some(desired_date))
            ),
            Some(result)
        );
    }

    #[test]
    fn no_category_amount_at_date() {
        let resources = [String::from("Bank")];
        let tuples = vec![
            (
                Some(NaiveDate::from_ymd(2020, 1, 1)),
                resources[0].clone(),
                -65.4,
                Pending,
            ),
            (
                Some(NaiveDate::from_ymd(2020, 2, 1)),
                resources[0].clone(),
                -32.83,
                InProgress,
            ),
        ];
        let desired_date = NaiveDate::from_ymd(2020, 6, 12);
        let orders = tuples
            .into_iter()
            .map(|x| Order {
                date: x.0,
                resource: Some(x.1),
                amount: x.2,
                state: x.3,
                ..Order::default()
            })
            .collect::<Vec<Order>>();
        let account = Account {
            resources: resources.to_vec(),
            orders: orders.to_vec(),
            ..Account::create()
        };

        assert_eq!(
            account.get_category_amount(
                Resource,
                "Cash",
                OptionNaiveDateRange(None, Some(desired_date))
            ),
            None
        );
    }
}
