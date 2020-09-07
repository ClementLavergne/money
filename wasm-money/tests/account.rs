use wasm_money::AccountClient;

#[cfg(test)]
mod account {
    use super::*;

    #[test]
    fn manage_tags() {
        let mut account = AccountClient::create();

        assert_eq!(account.add_tag("Restaurant"), true);
        assert_eq!(account.add_tag("Car"), true);
        assert_eq!(account.add_tag("Sport"), true);
        assert_eq!(account.add_tag("Car"), false);
        assert_eq!(account.add_tag(""), false);
        assert_eq!(account.remove_tag("Car"), true);
        assert_eq!(account.remove_tag("Car"), false);
        assert_eq!(account.remove_tag("Restaurant"), true);
        assert_eq!(account.remove_tag("Services"), false);
    }

    #[test]
    fn manage_resources() {
        let mut account = AccountClient::create();

        assert_eq!(account.add_resource("Bank 1"), true);
        assert_eq!(account.add_resource("Bank 2"), true);
        assert_eq!(account.add_resource("Bank Check"), true);
        assert_eq!(account.add_resource("Bank 1"), false);
        assert_eq!(account.add_resource(""), false);
        assert_eq!(account.remove_resource("Bank 1"), true);
        assert_eq!(account.remove_resource("Bank 1"), false);
        assert_eq!(account.remove_resource("Bank 2"), true);
        assert_eq!(account.remove_resource("Bank 3"), false);
    }
}
