//! # Extensions.

use crate::filter::Filter;
use crate::filter::{NaiveDateFilter, OptionNaiveDateRange};
use crate::order::Order;
use crate::order::TransactionState::{Done, InProgress, Pending};
#[cfg(feature = "wasmbind")]
use js_sys::Array;
#[cfg(feature = "wasmbind")]
use wasm_bindgen::prelude::*;
use CategoryType::{Resource, Tag};

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
    fn filtered_orders(&self, filter: &Filter) -> Vec<(usize, &Order)>;
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
        sorted_vec.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
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
                    .filter(|order| date_filter.is_order_allowed(order))
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
                    .filter(|order| date_filter.is_order_allowed(order))
                    .for_each(|order| update_amount(order));

                if nb_orders > 0 {
                    Some(result)
                } else {
                    None
                }
            }
        }
    }

    fn filtered_orders(&self, filter: &Filter) -> Vec<(usize, &Order)> {
        self.iter()
            .enumerate()
            .filter(|(_, order)| filter.is_order_allowed(order))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::NaiveDate;

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
}
