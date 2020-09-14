//! Filtering option which allows or not an `Order` according to its *category* subscription.
use super::ItemSelector;
use CategoryFilter::{CategoryIgnored, Enabled};

/// Key-value tuple struct which manages either *tag* or *resource*.
#[derive(Clone, PartialEq, Debug)]
pub struct Category(pub String, pub ItemSelector);

/// Filtering options for tags or resources.
#[derive(PartialEq, Debug)]
pub enum CategoryFilter {
    CategoryIgnored,
    Enabled(Vec<Category>),
}

impl CategoryFilter {
    /// Initializes the filtered categories.
    pub fn set(&mut self, categories: std::vec::IntoIter<Category>) {
        if let CategoryIgnored = self {
            *self = Enabled(Vec::new());
        }
        if let Enabled(items) = self {
            items.clear();
            categories.for_each(|category| items.push(category));
        }
    }

    /// Pushes a new category.
    pub fn add(&mut self, category: Category) {
        if let Enabled(items) = self {
            items.push(category);
        } else {
            *self = Enabled(vec![category]);
        }
    }

    /// Deletes a category.
    pub fn remove(&mut self, category_name: &str) -> bool {
        if let Enabled(items) = self {
            if let Some(index) = items.iter().position(|item| item.0 == category_name) {
                if items.len() > 1 {
                    items.remove(index);
                } else {
                    *self = CategoryIgnored;
                }
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Toggles the state of a given category.
    pub fn toggle(&mut self, category: &str) -> Option<&ItemSelector> {
        if let Enabled(items) = self {
            if let Some(index) = items.iter().position(|item| item.0 == category) {
                items[index].1.toggle();
                Some(&items[index].1)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ItemSelector::{Discarded, Selected};
    use super::*;

    #[test]
    fn toggle_selection() {
        let categories = [
            "Category to be toggled 1 time only 😭".to_string(),
            "Category to be toggled 2 times!".to_string(),
        ];
        let intial_categories = [
            Category(categories[0].clone(), Selected),
            Category(categories[1].clone(), Selected),
        ];
        let mut category_filter = Enabled(intial_categories.to_vec());
        let final_categories = [
            Category(categories[0].clone(), Selected),
            Category(categories[1].clone(), Discarded),
        ];
        category_filter.toggle(intial_categories[1].0.as_str());

        assert_eq!(category_filter, Enabled(final_categories.to_vec()));

        let final_categories = [
            Category(categories[0].clone(), Discarded),
            Category(categories[1].clone(), Selected),
        ];
        category_filter.toggle(intial_categories[0].0.as_str());
        category_filter.toggle(intial_categories[1].0.as_str());

        assert_eq!(category_filter, Enabled(final_categories.to_vec()));
    }

    #[test]
    fn set_categories_to_disabled() {
        let categories = vec![
            Category("First category".into(), Selected),
            Category("Second category".into(), Selected),
            Category("Last category!".into(), Selected),
        ];
        let mut category_filter = CategoryIgnored;
        category_filter.set(categories.clone().into_iter());

        assert_eq!(category_filter, Enabled(categories));
    }

    #[test]
    fn update_categories_to_enabled() {
        let intial_categories = vec![
            Category("First category".into(), Selected),
            Category("Last category!".into(), Selected),
        ];
        let final_categories = vec![
            Category("First category".into(), Selected),
            Category("Second category".into(), Selected),
            Category("Last category!".into(), Selected),
        ];
        let mut category_filter = Enabled(intial_categories);
        category_filter.set(final_categories.clone().into_iter());

        assert_eq!(category_filter, Enabled(final_categories));
    }

    #[test]
    fn add_category_to_enabled() {
        let intial_categories = vec![
            Category("First category".into(), Selected),
            Category("Last category!".into(), Selected),
        ];
        let final_categories = vec![
            Category("First category".into(), Selected),
            Category("Last category!".into(), Selected),
            Category("The (new) last category".into(), Selected),
        ];
        let mut category_filter = Enabled(intial_categories);
        category_filter.add(final_categories[2].clone());

        assert_eq!(category_filter, Enabled(final_categories));
    }

    #[test]
    fn enable_when_first_category_added() {
        let final_categories = vec![Category("First category".into(), Selected)];
        let mut category_filter = CategoryIgnored;
        category_filter.add(final_categories[0].clone());

        assert_eq!(category_filter, Enabled(final_categories));
    }

    #[test]
    fn remove_category_to_enabled() {
        let initial_categories = [
            Category("First category".into(), Selected),
            Category("Last category!".into(), Selected),
        ];
        let final_categories = vec![Category("Last category!".into(), Selected)];
        let mut category_filter = Enabled(initial_categories.to_vec());

        assert_eq!(
            category_filter.remove(initial_categories[0].0.as_str()),
            true
        );
        assert_eq!(category_filter, Enabled(final_categories));
    }

    #[test]
    fn disable_when_last_category_removed() {
        let initial_categories = [Category("Last category!".into(), Selected)];
        let mut category_filter = Enabled(initial_categories.to_vec());

        assert_eq!(
            category_filter.remove(initial_categories[0].0.as_str()),
            true
        );
        assert_eq!(category_filter, CategoryIgnored);
    }

    #[test]
    fn attempt_to_remove_unknown_category() {
        let initial_categories = vec![
            Category("First category".into(), Selected),
            Category("Last category!".into(), Selected),
        ];
        let mut category_filter = Enabled(initial_categories.clone());

        assert_eq!(category_filter.remove("Unknown category"), false);
        assert_eq!(category_filter, Enabled(initial_categories));
    }
}
