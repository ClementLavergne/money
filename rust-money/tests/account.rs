use rust_money::order::{Order, TransactionState};
use rust_money::Account;

#[cfg(test)]
mod account {
    use super::*;

    #[test]
    fn add_hide_delete_orders() {
        let mut account = Account::create();

        account.add_order();
        account.get_order_mut(0).unwrap().description = "Car gas".into();
        account.add_order();
        account.get_order_mut(1).unwrap().description = "Gamepass Ultimate".into();
        account.add_order();
        account.get_order_mut(2).unwrap().description = "Home".into();

        assert_eq!(
            account
                .orders()
                .iter()
                .map(|x| (x.description.clone(), x.visible))
                .collect::<Vec<(String, bool)>>(),
            [
                (String::from("Car gas"), true),
                (String::from("Gamepass Ultimate"), true),
                (String::from("Home"), true)
            ]
        );

        account.get_order_mut(0).unwrap().visible = false;
        account.get_order_mut(1).unwrap().visible = false;

        assert_eq!(
            account
                .orders()
                .iter()
                .map(|x| (x.description.clone(), x.visible))
                .collect::<Vec<(String, bool)>>(),
            [
                (String::from("Car gas"), false),
                (String::from("Gamepass Ultimate"), false),
                (String::from("Home"), true)
            ]
        );

        account.delete_hidden_orders();

        assert_eq!(
            account
                .orders()
                .iter()
                .map(|x| (x.description.clone(), x.visible))
                .collect::<Vec<(String, bool)>>(),
            vec![(String::from("Home"), true)]
        );
    }

    #[test]
    fn check_filtered_orders() {
        let mut account = Account::create();
        let mut expected_orders: Vec<(usize, Order)> = vec![
            (0, Order::default()),
            (1, Order::default()),
            (2, Order::default()),
            (3, Order::default()),
        ];
        let resources = [String::from("Bank"), String::from("Cash")];
        let tags = [
            String::from("Food"),
            String::from("Service"),
            String::from("Transport"),
            String::from("Mom & Dad"),
            String::from("Supermarket"),
        ];

        resources.iter().for_each(|resource| {
            account.add_resource(resource.as_str());
        });
        tags.iter().for_each(|tag| {
            account.add_tag(tag.as_str());
        });

        account.add_order();
        account.get_order_mut(0).unwrap().description = "Car gas".into();
        account
            .get_order_mut(0)
            .unwrap()
            .set_resource(resources[1].as_str(), &resources);
        account
            .get_order_mut(0)
            .unwrap()
            .add_tag(tags[2].as_str(), &tags);
        account
            .get_order_mut(0)
            .unwrap()
            .add_tag(tags[3].as_str(), &tags);
        account.add_order();
        account.get_order_mut(1).unwrap().description = "Gamepass Ultimate".into();
        account
            .get_order_mut(1)
            .unwrap()
            .set_resource(resources[0].as_str(), &resources);
        account
            .get_order_mut(1)
            .unwrap()
            .add_tag(tags[1].as_str(), &tags);
        account.add_order();
        account.get_order_mut(2).unwrap().description = "Metro".into();
        account
            .get_order_mut(2)
            .unwrap()
            .set_resource(resources[0].as_str(), &resources);
        account
            .get_order_mut(2)
            .unwrap()
            .add_tag(tags[2].as_str(), &tags);
        account
            .get_order_mut(2)
            .unwrap()
            .add_tag(tags[3].as_str(), &tags);
        account.add_order();
        account.get_order_mut(3).unwrap().description = "Pasta & Eggs".into();
        account
            .get_order_mut(3)
            .unwrap()
            .add_tag(tags[0].as_str(), &tags);
        account
            .get_order_mut(3)
            .unwrap()
            .add_tag(tags[4].as_str(), &tags);
        account
            .get_order_mut(3)
            .unwrap()
            .add_tag(tags[3].as_str(), &tags);

        expected_orders[0].1.description = "Car gas".into();
        expected_orders[0]
            .1
            .set_resource(resources[1].as_str(), &resources);
        expected_orders[0].1.add_tag(tags[2].as_str(), &tags);
        expected_orders[0].1.add_tag(tags[3].as_str(), &tags);
        expected_orders[1].1.description = "Gamepass Ultimate".into();
        expected_orders[1]
            .1
            .set_resource(resources[0].as_str(), &resources);
        expected_orders[1].1.add_tag(tags[1].as_str(), &tags);
        expected_orders[2].1.description = "Metro".into();
        expected_orders[2]
            .1
            .set_resource(resources[0].as_str(), &resources);
        expected_orders[2].1.add_tag(tags[2].as_str(), &tags);
        expected_orders[2].1.add_tag(tags[3].as_str(), &tags);
        expected_orders[3].1.description = "Pasta & Eggs".into();
        expected_orders[3].1.add_tag(tags[0].as_str(), &tags);
        expected_orders[3].1.add_tag(tags[4].as_str(), &tags);
        expected_orders[3].1.add_tag(tags[3].as_str(), &tags);
        assert_eq!(
            account.filtered_orders(),
            [
                (expected_orders[0].0, &expected_orders[0].1),
                (expected_orders[1].0, &expected_orders[1].1),
                (expected_orders[2].0, &expected_orders[2].1),
            ]
        );

        account.toggle_resource("Bank");

        assert_eq!(
            account.filtered_orders(),
            [(expected_orders[0].0, &expected_orders[0].1),]
        );

        account.toggle_resource("Cash");

        assert_eq!(
            account.filtered_orders(),
            [(expected_orders[3].0, &expected_orders[3].1),]
        );

        account.toggle_resource("Bank");
        account.toggle_tag("Transport");

        assert_eq!(
            account.filtered_orders(),
            [
                (expected_orders[1].0, &expected_orders[1].1),
                (expected_orders[2].0, &expected_orders[2].1),
            ]
        );

        account
            .get_order_mut(3)
            .unwrap()
            .set_resource(resources[0].as_str(), &resources);
        account.toggle_resource("Cash");

        // Update expected values
        expected_orders[3]
            .1
            .set_resource(resources[0].as_str(), &resources);
        assert_eq!(
            account.filtered_orders(),
            [
                (expected_orders[0].0, &expected_orders[0].1),
                (expected_orders[1].0, &expected_orders[1].1),
                (expected_orders[2].0, &expected_orders[2].1),
                (expected_orders[3].0, &expected_orders[3].1),
            ]
        );

        account.toggle_tag("Food");
        account.toggle_tag("Service");
        account.toggle_tag("Supermarket");

        assert_eq!(
            account.filtered_orders(),
            [
                (expected_orders[0].0, &expected_orders[0].1),
                (expected_orders[2].0, &expected_orders[2].1),
                (expected_orders[3].0, &expected_orders[3].1),
            ]
        );

        account.toggle_tag("Mom & Dad");

        assert_eq!(account.filtered_orders().len(), 0);

        account.clear_filters();

        assert_eq!(
            account.filtered_orders(),
            [
                (expected_orders[0].0, &expected_orders[0].1),
                (expected_orders[1].0, &expected_orders[1].1),
                (expected_orders[2].0, &expected_orders[2].1),
                (expected_orders[3].0, &expected_orders[3].1),
            ]
        );

        account.get_order_mut(1).unwrap().visible = false;
        account.get_order_mut(2).unwrap().visible = false;

        assert_eq!(
            account.filtered_orders(),
            [
                (expected_orders[0].0, &expected_orders[0].1),
                (expected_orders[3].0, &expected_orders[3].1),
            ]
        );

        account.get_order_mut(1).unwrap().visible = true;
        account.get_order_mut(2).unwrap().visible = true;
        account
            .get_order_mut(0)
            .unwrap()
            .set_state(TransactionState::Done);
        account
            .get_order_mut(1)
            .unwrap()
            .set_state(TransactionState::InProgress);
        account
            .get_order_mut(2)
            .unwrap()
            .set_state(TransactionState::Done);

        // Update expected values
        expected_orders[0].1.set_state(TransactionState::Done);
        expected_orders[1].1.set_state(TransactionState::InProgress);
        expected_orders[2].1.set_state(TransactionState::Done);
        assert_eq!(
            account.filtered_orders(),
            [
                (expected_orders[0].0, &expected_orders[0].1),
                (expected_orders[1].0, &expected_orders[1].1),
                (expected_orders[2].0, &expected_orders[2].1),
                (expected_orders[3].0, &expected_orders[3].1),
            ]
        );

        account.toggle_state(TransactionState::Done);

        assert_eq!(
            account.filtered_orders(),
            [
                (expected_orders[1].0, &expected_orders[1].1),
                (expected_orders[3].0, &expected_orders[3].1),
            ]
        );

        account.toggle_state(TransactionState::InProgress);

        assert_eq!(
            account.filtered_orders(),
            [(expected_orders[3].0, &expected_orders[3].1),]
        );

        account.clear_filters();
        account.get_order_mut(0).unwrap().visible = false;
        account.get_order_mut(3).unwrap().visible = false;
        account.delete_hidden_orders();

        // Update expected values
        expected_orders[1].0 = 0;
        expected_orders[2].0 = 1;
        assert_eq!(
            account.filtered_orders(),
            [
                (expected_orders[1].0, &expected_orders[1].1),
                (expected_orders[2].0, &expected_orders[2].1),
            ]
        );
    }
}
