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

    /// Returns true if input list holds (at leat) all selected categories, false otherwise.
    pub fn with_each_selected(&self, category_names: &[String]) -> bool {
        match self {
            CategoryIgnored => true,
            Enabled(categories) => categories
                .iter()
                .filter(|category| category.1 == ItemSelector::Selected)
                .all(|category| category_names.contains(&category.0)),
        }
    }

    /// Returns true if *some* input category name is among selected ones.
    pub fn among_any_selected(&self, category_name: &Option<String>) -> bool {
        match self {
            CategoryIgnored => true,
            Enabled(categories) if category_name == &None => categories
                .iter()
                .all(|category| category.1 == ItemSelector::Discarded),
            Enabled(categories) => categories
                .iter()
                .filter(|category| category.1 == ItemSelector::Selected)
                .any(|category| category.0 == *category_name.as_ref().unwrap()),
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

    #[test]
    fn allow_any_category() {
        let category_filter = CategoryIgnored;
        let allowed_category_1 = None;
        let allowed_category_2 = Some("Car".to_string());

        assert_eq!(
            category_filter.among_any_selected(&allowed_category_1),
            true
        );
        assert_eq!(
            category_filter.among_any_selected(&allowed_category_2),
            true
        );
    }

    #[test]
    fn allow_selected_category() {
        let categories = vec![
            Category("Bank".to_string(), Selected),
            Category("Cash".to_string(), Discarded),
        ];
        let allowed_category = Some(categories[0].0.clone());
        let rejected_category_1 = Some(categories[1].0.clone());
        let rejected_category_2 = Some("Unknown".to_string());
        let rejected_category_3 = None;
        let category_filter = Enabled(categories);

        assert_eq!(category_filter.among_any_selected(&allowed_category), true);
        assert_eq!(
            category_filter.among_any_selected(&rejected_category_1),
            false
        );
        assert_eq!(
            category_filter.among_any_selected(&rejected_category_2),
            false
        );
        assert_eq!(
            category_filter.among_any_selected(&rejected_category_3),
            false
        );
    }

    #[test]
    fn allow_empty_category_only() {
        let categories = vec![
            Category("Bank".to_string(), Discarded),
            Category("Cash".to_string(), Discarded),
        ];
        let allowed_category = None;
        let rejected_category_1 = Some(categories[0].0.clone());
        let rejected_category_2 = Some(categories[1].0.clone());
        let rejected_category_3 = Some("Unknown".to_string());
        let category_filter = Enabled(categories);

        assert_eq!(category_filter.among_any_selected(&allowed_category), true);
        assert_eq!(
            category_filter.among_any_selected(&rejected_category_1),
            false
        );
        assert_eq!(
            category_filter.among_any_selected(&rejected_category_2),
            false
        );
        assert_eq!(
            category_filter.among_any_selected(&rejected_category_3),
            false
        );
    }

    #[test]
    fn allow_any_list() {
        let category_filter = CategoryIgnored;
        let allowed_category_1 = [];
        let allowed_category_2 = ["Car".to_string(), "Insurance".to_string()];

        assert_eq!(
            category_filter.with_each_selected(&allowed_category_1),
            true
        );
        assert_eq!(
            category_filter.with_each_selected(&allowed_category_2),
            true
        );
    }

    #[test]
    fn allow_list_with_each_selected_categories() {
        let categories = vec![
            Category("Car".to_string(), Selected),
            Category("Mum".to_string(), Discarded),
            Category("Microsoft".to_string(), Selected),
        ];
        let allowed_category_1 = [
            categories[0].0.clone(),
            categories[1].0.clone(),
            categories[2].0.clone(),
        ];
        let allowed_category_2 = [categories[0].0.clone(), categories[2].0.clone()];
        let rejected_category_1 = [categories[1].0.clone()];
        let rejected_category_2 = ["Unknown".to_string()];
        let rejected_category_3 = [];
        let rejected_category_4 = [categories[0].0.clone()];
        let rejected_category_5 = [categories[2].0.clone()];
        let category_filter = Enabled(categories);

        assert_eq!(
            category_filter.with_each_selected(&allowed_category_1),
            true
        );
        assert_eq!(
            category_filter.with_each_selected(&allowed_category_2),
            true
        );
        assert_eq!(
            category_filter.with_each_selected(&rejected_category_1),
            false
        );
        assert_eq!(
            category_filter.with_each_selected(&rejected_category_2),
            false
        );
        assert_eq!(
            category_filter.with_each_selected(&rejected_category_3),
            false
        );
        assert_eq!(
            category_filter.with_each_selected(&rejected_category_4),
            false
        );
        assert_eq!(
            category_filter.with_each_selected(&rejected_category_5),
            false
        );
    }
}
