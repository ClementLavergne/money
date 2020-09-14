//! # Management of filtering options for an `Order` list.
pub mod category;
pub mod date;

use super::order::{Order, TransactionState};
use category::CategoryFilter;
use category::CategoryFilter::{CategoryIgnored, Enabled};
pub use chrono::NaiveDate;
use date::NaiveDateFilter::{Between, DateIgnored, Since, Until};
use date::{NaiveDateFilter, OptionNaiveDateRange};
use std::str::FromStr;
#[cfg(feature = "wasmbind")]
use wasm_bindgen::prelude::*;
use ItemSelector::{Discarded, Selected};
use VisibilityFilter::{HiddenOnly, VisibilityIgnored, VisibleOnly};

/// Stores current state of a given filter parameter.
#[cfg_attr(feature = "wasmbind", wasm_bindgen)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ItemSelector {
    /// Filter out.
    Discarded,
    /// Filter in.
    Selected,
}

impl ItemSelector {
    /// Toggles the state.
    pub fn toggle(&mut self) {
        *self = match *self {
            Discarded => Selected,
            Selected => Discarded,
        };
    }
}

/// Filtering options for visibility.
#[cfg_attr(feature = "wasmbind", wasm_bindgen)]
#[derive(Copy, Clone)]
pub enum VisibilityFilter {
    /// No visibility filtering is enabled.
    VisibilityIgnored,
    /// Filter visible orders.
    VisibleOnly,
    /// Filter hidden orders.
    HiddenOnly,
}

/// Stores all filtering options.
#[cfg_attr(feature = "wasmbind", wasm_bindgen)]
pub struct Filter {
    /// Keeps visible orders if `true`.
    pub visibility: VisibilityFilter,
    pub(crate) date_option: NaiveDateFilter,
    pub(crate) state_option: [ItemSelector; 3],
    pub(crate) resource_option: CategoryFilter,
    pub(crate) tag_option: CategoryFilter,
}

impl Default for Filter {
    fn default() -> Self {
        Filter {
            visibility: VisibleOnly,
            date_option: DateIgnored,
            state_option: [Selected, Selected, Selected],
            resource_option: CategoryIgnored,
            tag_option: CategoryIgnored,
        }
    }
}

/// Functions exclusive to `wasm-bindgen`
#[cfg_attr(feature = "wasmbind", wasm_bindgen)]
#[cfg(not(default))]
impl Filter {
    /// Traits are not supported by `wasm-bindgen`.
    #[cfg_attr(feature = "wasmbind", wasm_bindgen(constructor))]
    pub fn create() -> Filter {
        Filter::default()
    }

    /// Returns selected *state*
    pub fn get_state(&self, state: TransactionState) -> ItemSelector {
        self.state_option[state as usize]
    }
}

/// `wasm_bindgen` compatible functions.
#[cfg_attr(feature = "wasmbind", wasm_bindgen)]
impl Filter {
    /// Sets the date boundaries for allowed orders.
    ///
    /// # Output
    /// * `true` if the operation succeeded
    /// * `false` otherwise.
    pub fn set_date_option(&mut self, start: &str, stop: &str) -> bool {
        self.date_option.set_range(OptionNaiveDateRange(
            NaiveDate::from_str(start).ok(),
            NaiveDate::from_str(stop).ok(),
        ));

        if let Between(_, _) = self.date_option {
            true
        } else {
            false
        }
    }

    /// Sets the start date limit for allowed orders.
    ///
    /// # Output
    /// * `true` if the operation succeeded
    /// * `false` otherwise.
    pub fn set_date_beginning(&mut self, start: &str) -> bool {
        self.date_option
            .set_beginning(NaiveDate::from_str(start).ok());

        match self.date_option {
            DateIgnored | Until(_) => false,
            Since(_) | Between(_, _) => true,
        }
    }

    /// Sets the end date limit for allowed orders.
    ///
    /// # Output
    /// * `true` if the operation succeeded
    /// * `false` otherwise.
    pub fn set_date_end(&mut self, end: &str) -> bool {
        self.date_option.set_end(NaiveDate::from_str(end).ok());

        match self.date_option {
            DateIgnored | Since(_) => false,
            Until(_) | Between(_, _) => true,
        }
    }

    /// Disable *date* filter.
    pub fn disable_date_option(&mut self) {
        self.date_option = DateIgnored;
    }

    /// Toggles the selection of a given state.
    pub fn toggle_state(&mut self, state: TransactionState) {
        self.state_option[state as usize].toggle();
    }
}

impl Filter {
    /// Getter of attribute *tag_option*.
    pub fn tag_option(&self) -> &CategoryFilter {
        &self.tag_option
    }

    /// Getter of attribute *resource_option*.
    pub fn resource_option(&self) -> &CategoryFilter {
        &self.resource_option
    }

    /// Required to make the structure compatible with `wasm-bindgen`.
    pub fn get_tag_option_mut(&mut self) -> &mut CategoryFilter {
        &mut self.tag_option
    }

    /// Required to make the structure compatible with `wasm-bindgen`.
    pub fn get_resource_option_mut(&mut self) -> &mut CategoryFilter {
        &mut self.resource_option
    }

    /// Returns `true` if the *order* satisifies all filtering options; `false` otherwise.
    pub fn is_order_allowed(&self, order: &Order) -> bool {
        // Discard incompatible orders
        let visibility_match = match self.visibility {
            VisibilityIgnored => true,
            VisibleOnly => order.visible,
            HiddenOnly => !order.visible,
        };

        // Make sure the current state is among allowed ones
        let state_match = self.state_option[order.state() as usize] == Selected;

        // If the date does not satisfy the range, the order will be rejected.
        let date_match = match self.date_option {
            DateIgnored => true,
            Until(end) => {
                if let Some(date) = order.date {
                    end.signed_duration_since(date).num_days() >= 0
                } else {
                    false
                }
            }
            Since(start) => {
                if let Some(date) = order.date {
                    date.signed_duration_since(start).num_days() >= 0
                } else {
                    false
                }
            }
            Between(start, end) => {
                if let Some(date) = order.date {
                    date.signed_duration_since(start).num_days() >= 0
                        && end.signed_duration_since(date).num_days() >= 0
                } else {
                    false
                }
            }
        };

        // If all tags are discarded, the order will be rejected.
        // Unknown tags are not filtered.
        let tag_match = match &self.tag_option {
            CategoryIgnored => true,
            Enabled(tags) => {
                if !order.tags.is_empty() {
                    !order.tags.iter().all(|tag| {
                        if let Some(index) = tags.iter().position(|item| &item.0 == tag) {
                            tags[index].1 == Discarded
                        } else {
                            false
                        }
                    })
                } else {
                    true
                }
            }
        };

        // Make sure the resource is part of allowed ones
        let resource_match = match &self.resource_option {
            CategoryIgnored => true,
            Enabled(resources) => {
                if let Some(resource) = &order.resource {
                    if let Some(index) = resources.iter().position(|item| &item.0 == resource) {
                        resources[index].1 == Selected
                    } else {
                        true
                    }
                } else {
                    resources.iter().all(|item| item.1 == Discarded)
                }
            }
        };

        visibility_match && state_match && date_match && tag_match && resource_match
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use category::Category;

    #[test]
    fn allow_order_with_any_visibility() {
        let filter = Filter {
            visibility: VisibilityIgnored,
            ..Filter::default()
        };
        let allowed_order_1 = Order {
            visible: true,
            ..Order::default()
        };
        let allowed_order_2 = Order {
            visible: false,
            ..Order::default()
        };

        assert_eq!(filter.is_order_allowed(&allowed_order_1), true);
        assert_eq!(filter.is_order_allowed(&allowed_order_2), true);
    }

    #[test]
    fn allow_visible_order_only() {
        let filter = Filter {
            visibility: VisibleOnly,
            ..Filter::default()
        };
        let allowed_order = Order {
            visible: true,
            ..Order::default()
        };
        let rejected_order = Order {
            visible: false,
            ..Order::default()
        };

        assert_eq!(filter.is_order_allowed(&allowed_order), true);
        assert_eq!(filter.is_order_allowed(&rejected_order), false);
    }

    #[test]
    fn allow_hidden_order_only() {
        let filter = Filter {
            visibility: HiddenOnly,
            ..Filter::default()
        };
        let allowed_order = Order {
            visible: false,
            ..Order::default()
        };
        let rejected_order = Order {
            visible: true,
            ..Order::default()
        };

        assert_eq!(filter.is_order_allowed(&allowed_order), true);
        assert_eq!(filter.is_order_allowed(&rejected_order), false);
    }

    #[test]
    fn allow_order_with_any_resource() {
        let filter = Filter {
            resource_option: CategoryIgnored,
            ..Filter::default()
        };
        let allowed_order_1 = Order {
            resource: None,
            ..Order::default()
        };
        let allowed_order_2 = Order {
            resource: Some("Car".to_string()),
            ..Order::default()
        };

        assert_eq!(filter.is_order_allowed(&allowed_order_1), true);
        assert_eq!(filter.is_order_allowed(&allowed_order_2), true);
    }

    #[test]
    fn allow_order_with_resource_among_selected_only() {
        let categories = [
            Category("Bank".to_string(), Selected),
            Category("Cash".to_string(), Discarded),
        ];
        let filter = Filter {
            resource_option: Enabled(categories.to_vec()),
            ..Filter::default()
        };
        let allowed_order = Order {
            resource: Some(categories[0].0.clone()),
            ..Order::default()
        };
        let rejected_order = Order {
            resource: Some(categories[1].0.clone()),
            ..Order::default()
        };

        assert_eq!(filter.is_order_allowed(&allowed_order), true);
        assert_eq!(filter.is_order_allowed(&rejected_order), false);
    }

    #[test]
    fn allow_order_without_resource_only() {
        let categories = [
            Category("Bank".to_string(), Discarded),
            Category("Cash".to_string(), Discarded),
        ];
        let filter = Filter {
            resource_option: Enabled(categories.to_vec()),
            ..Filter::default()
        };
        let allowed_order = Order {
            resource: None,
            ..Order::default()
        };
        let rejected_order = Order {
            resource: Some(categories[1].0.clone()),
            ..Order::default()
        };

        assert_eq!(filter.is_order_allowed(&allowed_order), true);
        assert_eq!(filter.is_order_allowed(&rejected_order), false);
    }

    #[test]
    fn reject_order_without_resource_only() {
        let categories = [
            Category("Bank".to_string(), Selected),
            Category("Cash".to_string(), Selected),
        ];
        let filter = Filter {
            resource_option: Enabled(categories.to_vec()),
            ..Filter::default()
        };
        let allowed_order = Order {
            resource: Some(categories[0].0.clone()),
            ..Order::default()
        };
        let rejected_order = Order {
            resource: None,
            ..Order::default()
        };

        assert_eq!(filter.is_order_allowed(&allowed_order), true);
        assert_eq!(filter.is_order_allowed(&rejected_order), false);
    }

    #[test]
    fn allow_order_without_each_tag_discarded_only() {
        let categories = [
            Category("Car".to_string(), Discarded),
            Category("Mum".to_string(), Discarded),
            Category("Microsoft".to_string(), Selected),
        ];
        let allowed_order_1 = Order {
            tags: Vec::new(),
            ..Order::default()
        };
        let allowed_order_2 = Order {
            tags: vec![categories[0].0.clone(), "Unknown".to_string()],
            ..Order::default()
        };
        let allowed_order_3 = Order {
            tags: vec![
                categories[0].0.clone(),
                categories[1].0.clone(),
                categories[2].0.clone(),
            ],
            ..Order::default()
        };
        let rejected_order_1 = Order {
            tags: vec![categories[1].0.clone()],
            ..Order::default()
        };
        let rejected_order_2 = Order {
            tags: vec![categories[0].0.clone(), categories[1].0.clone()],
            ..Order::default()
        };
        let filter = Filter {
            tag_option: Enabled(categories.to_vec()),
            ..Filter::default()
        };

        assert_eq!(filter.is_order_allowed(&allowed_order_1), true);
        assert_eq!(filter.is_order_allowed(&allowed_order_2), true);
        assert_eq!(filter.is_order_allowed(&allowed_order_3), true);
        assert_eq!(filter.is_order_allowed(&rejected_order_1), false);
        assert_eq!(filter.is_order_allowed(&rejected_order_2), false);
    }

    #[test]
    fn allow_order_with_any_date() {
        let filter = Filter {
            date_option: DateIgnored,
            ..Filter::default()
        };
        let allowed_order_1 = Order {
            date: None,
            ..Order::default()
        };
        let allowed_order_2 = Order {
            date: Some(NaiveDate::from_ymd(2020, 9, 9)),
            ..Order::default()
        };

        assert_eq!(filter.is_order_allowed(&allowed_order_1), true);
        assert_eq!(filter.is_order_allowed(&allowed_order_2), true);
    }

    #[test]
    fn allow_order_with_date_before_end() {
        let filter = Filter {
            date_option: Until(NaiveDate::from_ymd(2020, 9, 9)),
            ..Filter::default()
        };
        let allowed_order_1 = Order {
            date: Some(NaiveDate::from_ymd(2020, 8, 8)),
            ..Order::default()
        };
        let allowed_order_2 = Order {
            date: Some(NaiveDate::from_ymd(2020, 9, 9)),
            ..Order::default()
        };
        let rejected_order = Order {
            date: Some(NaiveDate::from_ymd(2020, 9, 12)),
            ..Order::default()
        };

        assert_eq!(filter.is_order_allowed(&allowed_order_1), true);
        assert_eq!(filter.is_order_allowed(&allowed_order_2), true);
        assert_eq!(filter.is_order_allowed(&rejected_order), false);
    }

    #[test]
    fn allow_order_with_date_after_beginning() {
        let filter = Filter {
            date_option: Since(NaiveDate::from_ymd(2020, 9, 9)),
            ..Filter::default()
        };
        let allowed_order_1 = Order {
            date: Some(NaiveDate::from_ymd(2020, 9, 12)),
            ..Order::default()
        };
        let allowed_order_2 = Order {
            date: Some(NaiveDate::from_ymd(2020, 9, 9)),
            ..Order::default()
        };
        let rejected_order = Order {
            date: Some(NaiveDate::from_ymd(2020, 9, 6)),
            ..Order::default()
        };

        assert_eq!(filter.is_order_allowed(&allowed_order_1), true);
        assert_eq!(filter.is_order_allowed(&allowed_order_2), true);
        assert_eq!(filter.is_order_allowed(&rejected_order), false);
    }

    #[test]
    fn allow_order_with_date_between_range() {
        let filter = Filter {
            date_option: Between(
                NaiveDate::from_ymd(2020, 9, 1),
                NaiveDate::from_ymd(2020, 10, 1),
            ),
            ..Filter::default()
        };
        let allowed_order_1 = Order {
            date: Some(NaiveDate::from_ymd(2020, 9, 1)),
            ..Order::default()
        };
        let allowed_order_2 = Order {
            date: Some(NaiveDate::from_ymd(2020, 10, 1)),
            ..Order::default()
        };
        let allowed_order_3 = Order {
            date: Some(NaiveDate::from_ymd(2020, 9, 12)),
            ..Order::default()
        };
        let rejected_order_1 = Order {
            date: Some(NaiveDate::from_ymd(2020, 8, 30)),
            ..Order::default()
        };
        let rejected_order_2 = Order {
            date: Some(NaiveDate::from_ymd(2020, 10, 5)),
            ..Order::default()
        };

        assert_eq!(filter.is_order_allowed(&allowed_order_1), true);
        assert_eq!(filter.is_order_allowed(&allowed_order_2), true);
        assert_eq!(filter.is_order_allowed(&allowed_order_3), true);
        assert_eq!(filter.is_order_allowed(&rejected_order_1), false);
        assert_eq!(filter.is_order_allowed(&rejected_order_2), false);
    }
}
