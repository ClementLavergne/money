//! # Management of filtering options for an `Order` list.
pub mod category;
pub mod date;

use crate::ext::OrderingDirection::Ascending;
use crate::ext::OrderingPreference::ById;
use crate::ext::{OrderingDirection, OrderingPreference};
use crate::order::{Order, TransactionState};
use category::CategoryFilter;
use category::CategoryFilter::CategoryIgnored;
pub use chrono::NaiveDate;
use date::NaiveDateFilter::{Between, DateIgnored, Since, Until};
pub use date::{NaiveDateFilter, OptionNaiveDateRange};
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
    pub ordering: OrderingPreference,
    pub direction: OrderingDirection,
}

impl Default for Filter {
    fn default() -> Self {
        Filter {
            visibility: VisibleOnly,
            date_option: DateIgnored,
            state_option: [Selected, Selected, Selected],
            resource_option: CategoryIgnored,
            tag_option: CategoryIgnored,
            ordering: ById,
            direction: Ascending,
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

        matches!(self.date_option, Between(_, _))
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
    /// Getter of attribute *date_option*.
    pub fn date_option(&self) -> &NaiveDateFilter {
        &self.date_option
    }

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
        let date_match = self.date_option.is_date_allowed(order.date);

        // If some tags are selected, allowed orders are the ones which own them
        // at least.
        let tag_match = self.tag_option.with_each_selected(&order.tags);

        // Make sure the resource is part of allowed ones
        let resource_match = self.resource_option.among_any_selected(&order.resource);

        visibility_match && state_match && date_match && tag_match && resource_match
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
