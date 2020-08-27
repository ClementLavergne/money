//! # module `ext`
//!
//! Adds new functions to `Vec<String>` type.

pub trait ExclusiveItemExt {
    fn add_exclusive(&mut self, tag: &str);
    fn remove_exclusive(&mut self, tag: &str);
}

impl ExclusiveItemExt for Vec<String> {
    /// Adds a new item if not exists yet.
    fn add_exclusive(&mut self, tag: &str) {
        if !tag.is_empty() && !self.iter().any(|i| i == tag) {
            self.push(tag.to_string())
        }
    }

    /// Removes an existing item.
    fn remove_exclusive(&mut self, tag: &str) {
        let index = self.iter().position(|x| x == tag);
        if let Some(i) = index {
            self.remove(i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignore_empty_item() {
        let mut vec = Vec::new();

        assert_eq!(vec.len(), 0);
        vec.add_exclusive("");
        assert_eq!(vec.len(), 0);
    }

    #[test]
    fn ignore_redundant_item() {
        let mut vec = Vec::new();

        assert_eq!(vec.len(), 0);
        vec.add_exclusive("Food");
        assert_eq!(vec![String::from("Food")], vec);
        vec.add_exclusive("Transport");
        assert_eq!(vec![String::from("Food"), String::from("Transport")], vec);
        vec.add_exclusive("Food");
        vec.add_exclusive("Service");
        assert_eq!(
            vec![
                String::from("Food"),
                String::from("Transport"),
                String::from("Service")
            ],
            vec
        );
    }

    #[test]
    fn remove_existing_item() {
        let mut vec = Vec::new();

        vec.add_exclusive("Food");
        vec.add_exclusive("Transport");
        vec.add_exclusive("Service");
        assert_eq!(
            vec![
                String::from("Food"),
                String::from("Transport"),
                String::from("Service")
            ],
            vec
        );
        vec.remove_exclusive("Food");
        assert_eq!(
            vec![String::from("Transport"), String::from("Service")],
            vec
        );
        vec.remove_exclusive("Hangout");
        vec.remove_exclusive("");
        assert_eq!(
            vec![String::from("Transport"), String::from("Service")],
            vec
        );
    }
}
