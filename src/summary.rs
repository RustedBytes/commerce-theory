//! Summary theorem surface for the Rust mirror.
//!
//! Lean theorem names are represented by executable constructors/predicates in
//! the domain modules and by unit tests that exercise the headline guarantees.

#[cfg(test)]
mod tests {
    use crate::*;

    fn sku() -> Sku {
        Sku::new(10)
    }

    fn supplier() -> DropshipSupplier {
        DropshipSupplier::new(
            SupplierId::new(1),
            "supplier".to_owned(),
            Currency::USD,
            true,
            false,
            2,
            true,
            100,
        )
    }

    fn cart_line() -> CartLine {
        CartLine::try_new(sku(), 100, 40, 2, 20, 3).unwrap()
    }

    fn shipping() -> ShippingMethod {
        ShippingMethod::new(15, 500, 20)
    }

    fn order() -> Order {
        let items = vec![cart_line()];
        let total = order_total(&shipping(), 10, 5, &items).unwrap();
        Order::try_new(
            OrderId::new(7),
            items,
            10,
            shipping(),
            5,
            Currency::USD,
            OrderStatus::New,
            total,
        )
        .unwrap()
    }

    fn accounts() -> AccountingAccounts {
        let a = |id, name: &str| LedgerAccount::new(id, name.to_owned());
        AccountingAccounts::new(
            a(1, "cash"),
            a(2, "deferred"),
            a(3, "revenue"),
            a(4, "refunds"),
            a(5, "inventory"),
            a(6, "cogs"),
        )
    }

    fn profit_costs() -> DropshipProfitCosts {
        DropshipProfitCosts::new(40, 5, 3, 2, 0, 1, 0, 0)
    }

    #[test]
    fn pricing_and_refund_safety() {
        let order = order();
        let gross_plus_shipping_plus_tax =
            cart_gross_total(order.items()).unwrap() + shipping().price + order.tax();
        assert!(order.total() <= gross_plus_shipping_plus_tax);

        let ledger = PaymentLedger::try_new(120, 20).unwrap();
        let next = issue_refund(&ledger, 40).unwrap();
        assert_eq!(remaining_refund_amount(&next), 60);
        assert!(issue_refund(&next, 100).is_err());
    }

    #[test]
    fn accounting_and_payment_projection_safety() {
        let journal = payment_captured_journal(&accounts(), 100).unwrap();
        assert_eq!(debit_total(journal.postings()).unwrap(), 100);
        assert_eq!(credit_total(journal.postings()).unwrap(), 100);

        let created = TypedPayment::<CreatedPayment>::try_new(
            PaymentId::new(1),
            OrderId::new(7),
            110,
            Currency::USD,
        )
        .unwrap();
        let (_captured, receipt) = capture_payment(authorize_payment(created));
        let typed_order =
            TypedOrder::<NewOrder>::try_new(OrderId::new(7), 110, Currency::USD).unwrap();
        assert!(mark_paid(typed_order, &receipt).is_ok());
    }

    #[test]
    fn inventory_feed_and_keyed_totals_are_safe() {
        let stock = StockState::try_new(sku(), 10, 3).unwrap();
        let warehouse = Warehouse::new(1, "main".to_owned());
        let node = InventoryNode::new(warehouse, stock.clone());
        let allocation = Allocation::try_new(node, 4).unwrap();
        let plan = FulfillmentPlan::try_new(4, vec![allocation.clone()]).unwrap();
        assert!(plan.requested() <= allocations_available_total(plan.allocations()).unwrap());

        let policy = ChannelPricePolicy::try_new(50, 200).unwrap();
        let feed = SafeProductFeedLine::try_new(
            sku(),
            SalesChannel::OwnWebsite,
            100,
            Currency::USD,
            3,
            stock,
            policy,
        )
        .unwrap();
        assert!(PublishableFeedLine::try_new(feed).is_ok());

        let totals = allocation_quantity_by_key(&[allocation]).unwrap();
        assert_eq!(totals.values().copied().sum::<u128>(), 4);
    }

    #[test]
    fn dropship_supplier_quality_and_opportunities() {
        let offer =
            DropshipOffer::try_new(sku(), supplier(), 40, 100, 2, 10, Currency::USD, true).unwrap();
        let line = DropshipLine::try_new(offer.clone(), 1, 10).unwrap();
        assert!(
            dropship_line_supplier_cost(&line).unwrap()
                <= dropship_line_customer_net(&line).unwrap()
        );

        let metrics = SupplierQualityMetrics::new(10, 20, 30);
        let policy = SupplierRiskPolicy::new(10, 25, 40);
        let quality = ApprovedSupplierQuality::try_new(supplier(), metrics, policy).unwrap();
        assert!(ApprovedOrderableSupplierQuality::try_new(quality).is_ok());

        let candidate =
            DropshipOpportunityCandidate::try_new(sku(), 1, 60, 10, 20, 5, 70, profit_costs())
                .unwrap();
        let ranked = rank_opportunity_keys(&[candidate]);
        assert_eq!(ranked.ret, vec![20]);
    }

    #[test]
    fn event_and_workflow_safety() {
        let events = vec![
            EventEnvelope::new(1, DomainEvent::OrderPlaced(OrderId::new(1), 100)),
            EventEnvelope::new(2, DomainEvent::PaymentCaptured(OrderId::new(1), 100)),
            EventEnvelope::new(3, DomainEvent::OrderShipped(OrderId::new(1))),
        ];
        let stream = EventStream::new(events.clone(), 3);
        assert!(ValidEventStream::try_new(stream).is_ok());
        assert!(replay_webhook_stream(WebhookOrderingState::new(0), &events).is_ok());

        let symbols: Vec<_> = events
            .iter()
            .map(|event| domain_event_symbol(event.event()))
            .collect();
        assert!(order_event_word_accepted(&symbols));
        assert!(order_event_validator().accepts(&symbols));

        let states = execute_order_trace(OrderStatus::New, &paid_fulfillment_trace()).unwrap();
        assert_eq!(states.last(), Some(&OrderStatus::Delivered));
        assert_eq!(
            CanOrderTransition::from_statuses(OrderStatus::New, OrderStatus::Paid)
                .unwrap()
                .target(),
            OrderStatus::Paid
        );
        assert_eq!(
            order_status_lts()
                .execute(OrderStatus::New, &paid_fulfillment_trace())
                .unwrap()
                .last(),
            Some(&OrderStatus::Delivered)
        );

        let po_states =
            execute_dropship_po_trace(DropshipPOStatus::Created, &dropship_po_delivery_trace())
                .unwrap();
        assert_eq!(po_states.last(), Some(&DropshipPOStatus::Delivered));
        assert_eq!(
            dropship_polts()
                .execute(DropshipPOStatus::Created, &dropship_po_delivery_trace())
                .unwrap()
                .last(),
            Some(&DropshipPOStatus::Delivered)
        );

        assert!(
            webhook_replay_within_steps(WebhookOrderingState::new(0), &events, 3)
                .unwrap()
                .is_some()
        );
        assert!(
            webhook_replay_within_steps(WebhookOrderingState::new(0), &events, 2)
                .unwrap()
                .is_none()
        );

        let system_state = ValidSystemState::new(
            StockState::try_new(sku(), 10, 0).unwrap(),
            PaymentLedger::try_new(100, 0).unwrap(),
        );
        let replay = valid_system_replay_within_steps(
            system_state,
            &[
                ValidSystemEvent::StockReserved(sku(), 2),
                ValidSystemEvent::RefundIssued(10),
            ],
            2,
        )
        .unwrap();
        assert!(replay.is_some());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_feature_round_trips_representative_value() {
        let line = cart_line();
        let json = serde_json::to_string(&line).unwrap();
        let decoded: CartLine = serde_json::from_str(&json).unwrap();
        assert_eq!(line, decoded);
    }
}
