//! # Extensions.

use crate::filter::{Filter, NaiveDateFilter, OptionNaiveDateRange};
use crate::order::Order;
use crate::order::TransactionState::{Done, InProgress, Pending};
#[cfg(feature = "wasmbind")]
use js_sys::Array;
#[cfg(feature = "wasmbind")]
use wasm_bindgen::prelude::*;
use CategoryType::{Resource, Tag};
use OrderingDirection::Ascending;
use OrderingPreference::{ByAmount, ByDate, ByDescription, ById};

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

/// Defines available *category* types.
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

/// All kinds of sorting preferences.
#[cfg_attr(feature = "wasmbind", wasm_bindgen)]
#[derive(Copy, Clone)]
pub enum OrderingPreference {
    ByDate,
    ByDescription,
    ByAmount,
    ById,
}

/// Direction when sorting orders.
#[cfg_attr(feature = "wasmbind", wasm_bindgen)]
#[derive(Copy, Clone, PartialEq)]
pub enum OrderingDirection {
    Ascending,
    Descending,
}

/// Extension for `Vec<String>` to manage unique keys.
pub trait ExclusiveItemExt {
    /// Adds a new item if not exists yet.
    fn add_exclusive(&mut self, key: &str) -> Option<RequestFailure>;

    /// Removes an existing item.
    fn remove_exclusive(&mut self, key: &str) -> Option<RequestFailure>;

    // Export sorted filter keys.
    #[cfg(feature = "wasmbind")]
    fn sorted_keys(&self) -> Array;
}

/// Extension for `Vec<Order>` to interpret existing data.
pub trait OrderListExt {
    /// Computes the different amounts of a *category* between a given range.
    fn calculate_category_amount(
        &self,
        kind: CategoryType,
        category: &str,
        date_range: OptionNaiveDateRange,
    ) -> Option<CategoryAmount>;

    /// Returns selected orders with their associated id.
    fn apply_filter(&self, filter: &Filter) -> Vec<(usize, &Order)>;
}

impl ExclusiveItemExt for Vec<String> {
    fn add_exclusive(&mut self, key: &str) -> Option<RequestFailure> {
        if !key.is_empty() {
            if !key.chars().all(char::is_whitespace) {
                if !self.iter().any(|item| item == key) {
                    self.push(key.into());
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

    fn remove_exclusive(&mut self, key: &str) -> Option<RequestFailure> {
        if let Some(index) = self.iter().position(|item| item == key) {
            self.remove(index);
            None
        } else {
            Some(RequestFailure::UnknownItem)
        }
    }

    #[cfg(feature = "wasmbind")]
    fn sorted_keys(&self) -> Array {
        let mut sorted_vec = self.clone();
        sorted_vec.sort_by_key(|a| a.to_lowercase());
        sorted_vec.iter().map(JsValue::from).collect()
    }
}

impl OrderListExt for Vec<Order> {
    fn calculate_category_amount(
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
                self.iter()
                    .filter(|order| order.visible && order.resource == Some(category.into()))
                    .filter(|order| date_filter.is_date_allowed(order.date))
                    .for_each(|order| update_amount(order));

                if nb_orders > 0 {
                    Some(result)
                } else {
                    None
                }
            }
            Tag => {
                self.iter()
                    .filter(|order| order.visible && order.tags.contains(&category.to_string()))
                    .filter(|order| date_filter.is_date_allowed(order.date))
                    .for_each(|order| update_amount(order));

                if nb_orders > 0 {
                    Some(result)
                } else {
                    None
                }
            }
        }
    }

    fn apply_filter(&self, filter: &Filter) -> Vec<(usize, &Order)> {
        // Retain matching orders
        let mut filtered_vector = self
            .iter()
            .enumerate()
            .filter(|(_, order)| filter.is_order_allowed(order))
            .collect::<Vec<(usize, &Order)>>();

        // Sort filtered orders by ordering preference
        match filter.ordering {
            ByDate => {
                if filter.direction == Ascending {
                    filtered_vector.sort_by(|a, b| a.1.date.cmp(&b.1.date));
                } else {
                    filtered_vector.sort_by(|a, b| b.1.date.cmp(&a.1.date));
                }
            }
            ByDescription => {
                if filter.direction == Ascending {
                    filtered_vector.sort_by(|a, b| {
                        a.1.description
                            .to_lowercase()
                            .cmp(&b.1.description.to_lowercase())
                    });
                } else {
                    filtered_vector.sort_by(|a, b| {
                        b.1.description
                            .to_lowercase()
                            .cmp(&a.1.description.to_lowercase())
                    });
                }
            }
            ByAmount => {
                if filter.direction == Ascending {
                    filtered_vector.sort_by(|a, b| {
                        a.1.amount
                            .partial_cmp(&b.1.amount)
                            .expect("Something goes wrong..")
                    });
                } else {
                    filtered_vector.sort_by(|a, b| {
                        b.1.amount
                            .partial_cmp(&a.1.amount)
                            .expect("Something goes wrong..")
                    });
                }
            }
            ById => {
                if filter.direction == Ascending {
                    filtered_vector.sort_by(|a, b| a.0.cmp(&b.0));
                } else {
                    filtered_vector.sort_by(|a, b| b.0.cmp(&a.0));
                }
            }
        }

        filtered_vector
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::NaiveDate;
    use OrderingDirection::Descending;

    #[test]
    fn add_valid_key() {
        let items = (0..3)
            .map(|id| format!("Key {}", id))
            .collect::<Vec<String>>();

        let mut list: Vec<String> = items.as_slice()[..2].to_vec();

        assert_eq!(list.add_exclusive(items[2].as_str()), None);
        assert_eq!(list, items);
    }

    #[test]
    fn remove_known_key() {
        let items = (0..3)
            .map(|id| format!("Key {}", id))
            .collect::<Vec<String>>();

        let mut list = items.clone();

        assert_eq!(list.remove_exclusive(items[2].as_str()), None);
        assert_eq!(list, items.as_slice()[..2].to_vec());
    }

    #[test]
    fn discard_adding_existing_key() {
        let items = (0..3)
            .map(|id| format!("Key {}", id))
            .collect::<Vec<String>>();

        let mut list = items.clone();

        assert_eq!(
            list.add_exclusive(items[2].as_str()),
            Some(RequestFailure::ExistingItem)
        );
        assert_eq!(list, items);
    }

    #[test]
    fn discard_adding_empty_key() {
        let mut list: Vec<String> = Vec::new();

        assert_eq!(list.add_exclusive(""), Some(RequestFailure::EmptyArgument));
        assert_eq!(list.is_empty(), true);
    }

    #[test]
    fn discard_adding_incorrect_key() {
        let mut list: Vec<String> = Vec::new();

        assert_eq!(
            list.add_exclusive("  "),
            Some(RequestFailure::IncorrectArgument)
        );
        assert_eq!(list.is_empty(), true);
    }

    #[test]
    fn discard_removing_unknown_key() {
        let items = (0..3)
            .map(|id| format!("Key {}", id))
            .collect::<Vec<String>>();

        let mut list = items.clone();

        assert_eq!(
            list.remove_exclusive("Key 4"),
            Some(RequestFailure::UnknownItem)
        );
        assert_eq!(list, items);
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

        assert_eq!(
            orders.calculate_category_amount(
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

        assert_eq!(
            orders.calculate_category_amount(
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

        assert_eq!(
            orders.calculate_category_amount(
                Resource,
                "Cash",
                OptionNaiveDateRange(None, Some(desired_date))
            ),
            None
        );
    }

    #[test]
    fn sort_orders_by_date() {
        let orders = vec![
            Order {
                date: Some(NaiveDate::from_ymd(2020, 6, 3)),
                ..Order::default()
            },
            Order {
                date: Some(NaiveDate::from_ymd(2020, 10, 11)),
                ..Order::default()
            },
            Order {
                date: None,
                ..Order::default()
            },
            Order {
                date: Some(NaiveDate::from_ymd(2020, 8, 23)),
                ..Order::default()
            },
        ];
        let result = [2, 0, 3, 1]
            .iter()
            .map(|&x| (x, &orders[x]))
            .collect::<Vec<(usize, &Order)>>();

        assert_eq!(
            orders.apply_filter(&Filter {
                ordering: ByDate,
                direction: Ascending,
                ..Filter::default()
            }),
            result
        );

        let result = [1, 3, 0, 2]
            .iter()
            .map(|&x| (x, &orders[x]))
            .collect::<Vec<(usize, &Order)>>();

        assert_eq!(
            orders.apply_filter(&Filter {
                ordering: ByDate,
                direction: Descending,
                ..Filter::default()
            }),
            result
        );
    }

    #[test]
    fn sort_orders_by_description() {
        let orders = vec![
            Order {
                description: "Shopping 🛍".into(),
                ..Order::default()
            },
            Order {
                description: "Restaurant 🥘".into(),
                ..Order::default()
            },
            Order {
                description: "Cinema 🍿".into(),
                ..Order::default()
            },
            Order {
                description: "Tennis 🎾".into(),
                ..Order::default()
            },
        ];

        let result = [2, 1, 0, 3]
            .iter()
            .map(|&x| (x, &orders[x]))
            .collect::<Vec<(usize, &Order)>>();

        assert_eq!(
            orders.apply_filter(&Filter {
                ordering: ByDescription,
                direction: Ascending,
                ..Filter::default()
            }),
            result
        );

        let result = [3, 0, 1, 2]
            .iter()
            .map(|&x| (x, &orders[x]))
            .collect::<Vec<(usize, &Order)>>();

        assert_eq!(
            orders.apply_filter(&Filter {
                ordering: ByDescription,
                direction: Descending,
                ..Filter::default()
            }),
            result
        );
    }

    #[test]
    fn sort_orders_by_id() {
        let orders = vec![
            Order {
                description: "Shopping 🛍".into(),
                ..Order::default()
            },
            Order {
                description: "Restaurant 🥘".into(),
                ..Order::default()
            },
            Order {
                description: "Cinema 🍿".into(),
                ..Order::default()
            },
            Order {
                description: "Tennis 🎾".into(),
                ..Order::default()
            },
        ];

        let result = [0, 1, 2, 3]
            .iter()
            .map(|&x| (x, &orders[x]))
            .collect::<Vec<(usize, &Order)>>();

        assert_eq!(
            orders.apply_filter(&Filter {
                ordering: ById,
                direction: Ascending,
                ..Filter::default()
            }),
            result
        );

        let result = [3, 2, 1, 0]
            .iter()
            .map(|&x| (x, &orders[x]))
            .collect::<Vec<(usize, &Order)>>();

        assert_eq!(
            orders.apply_filter(&Filter {
                ordering: ById,
                direction: Descending,
                ..Filter::default()
            }),
            result
        );
    }

    #[test]
    fn sort_orders_by_amount() {
        let orders = vec![
            Order {
                amount: 34.99,
                ..Order::default()
            },
            Order {
                amount: -5.5,
                ..Order::default()
            },
            Order {
                amount: -69.99,
                ..Order::default()
            },
            Order {
                amount: 15.00,
                ..Order::default()
            },
        ];

        let result = [2, 1, 3, 0]
            .iter()
            .map(|&x| (x, &orders[x]))
            .collect::<Vec<(usize, &Order)>>();

        assert_eq!(
            orders.apply_filter(&Filter {
                ordering: ByAmount,
                direction: Ascending,
                ..Filter::default()
            }),
            result
        );

        let result = [0, 3, 1, 2]
            .iter()
            .map(|&x| (x, &orders[x]))
            .collect::<Vec<(usize, &Order)>>();

        assert_eq!(
            orders.apply_filter(&Filter {
                ordering: ByAmount,
                direction: Descending,
                ..Filter::default()
            }),
            result
        );
    }
}
