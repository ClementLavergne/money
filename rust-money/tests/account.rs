#[cfg(test)]
mod account {
    use rust_money::ext::OrderListExt;
    use rust_money::filter::category::Category;
    use rust_money::filter::{Filter, ItemSelector, VisibilityFilter};
    use rust_money::order::{Order, TransactionState};
    use rust_money::Account;

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
    }

    #[test]
    fn check_filtered_orders() {
        let mut filter = Filter::default();
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
        filter.get_resource_option_mut().set(
            resources
                .iter()
                .map(|resource| Category(resource.clone(), ItemSelector::Selected))
                .collect::<Vec<Category>>()
                .into_iter(),
        );
        filter.get_tag_option_mut().set(
            tags.iter()
                .map(|tag| Category(tag.clone(), ItemSelector::Discarded))
                .collect::<Vec<Category>>()
                .into_iter(),
        );

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
            account.orders().apply_filter(&filter),
            [
                (expected_orders[0].0, &expected_orders[0].1),
                (expected_orders[1].0, &expected_orders[1].1),
                (expected_orders[2].0, &expected_orders[2].1),
            ]
        );

        filter.get_resource_option_mut().toggle("Bank");
        assert_eq!(
            account.orders().apply_filter(&filter),
            [(expected_orders[0].0, &expected_orders[0].1),]
        );
        filter.get_resource_option_mut().toggle("Cash");
        assert_eq!(
            account.orders().apply_filter(&filter),
            [(expected_orders[3].0, &expected_orders[3].1),]
        );

        filter.get_resource_option_mut().toggle("Bank");
        filter.get_tag_option_mut().toggle("Transport");

        assert_eq!(
            account.orders().apply_filter(&filter),
            [(expected_orders[2].0, &expected_orders[2].1),]
        );

        account
            .get_order_mut(3)
            .unwrap()
            .set_resource(resources[0].as_str(), &resources);
        filter.get_resource_option_mut().toggle("Cash");

        // Update expected values
        expected_orders[3]
            .1
            .set_resource(resources[0].as_str(), &resources);
        assert_eq!(
            account.orders().apply_filter(&filter),
            [
                (expected_orders[0].0, &expected_orders[0].1),
                (expected_orders[2].0, &expected_orders[2].1),
            ]
        );

        filter.get_tag_option_mut().toggle("Food");
        filter.get_tag_option_mut().toggle("Service");
        filter.get_tag_option_mut().toggle("Supermarket");

        assert_eq!(account.orders().apply_filter(&filter), []);

        filter.get_tag_option_mut().toggle("Mom & Dad");

        assert_eq!(account.orders().apply_filter(&filter).len(), 0);

        filter = Filter::default();

        assert_eq!(
            account.orders().apply_filter(&filter),
            [
                (expected_orders[0].0, &expected_orders[0].1),
                (expected_orders[1].0, &expected_orders[1].1),
                (expected_orders[2].0, &expected_orders[2].1),
                (expected_orders[3].0, &expected_orders[3].1),
            ]
        );

        filter.visibility = VisibilityFilter::VisibleOnly;
        account.get_order_mut(1).unwrap().visible = false;
        account.get_order_mut(2).unwrap().visible = false;

        assert_eq!(
            account.orders().apply_filter(&filter),
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
            account.orders().apply_filter(&filter),
            [
                (expected_orders[0].0, &expected_orders[0].1),
                (expected_orders[1].0, &expected_orders[1].1),
                (expected_orders[2].0, &expected_orders[2].1),
                (expected_orders[3].0, &expected_orders[3].1),
            ]
        );

        filter.toggle_state(TransactionState::Done);

        assert_eq!(
            account.orders().apply_filter(&filter),
            [
                (expected_orders[1].0, &expected_orders[1].1),
                (expected_orders[3].0, &expected_orders[3].1),
            ]
        );

        filter.toggle_state(TransactionState::InProgress);

        assert_eq!(
            account.orders().apply_filter(&filter),
            [(expected_orders[3].0, &expected_orders[3].1),]
        );
    }
}
