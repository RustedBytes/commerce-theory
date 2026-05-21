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
            days(2),
            true,
            100,
        )
    }

    fn epoch() -> Timestamp {
        unix_epoch_timestamp()
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

    fn paid_order() -> Order {
        let items = vec![cart_line()];
        let total = order_total(&shipping(), 10, 5, &items).unwrap();
        Order::try_new(
            OrderId::new(8),
            items,
            10,
            shipping(),
            5,
            Currency::USD,
            OrderStatus::Paid,
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
        assert!(validate_captured_payment_journal_projection(accounts(), receipt).is_ok());
        assert!(
            validate_refund_journal_projection(
                accounts(),
                PaymentLedger::try_new(100, 0).unwrap(),
                10,
            )
            .is_ok()
        );
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

        let bundle_component = validate_bundle_component(sku(), 2, 6).unwrap();
        assert!(bundle_components_can_fulfill_all(3, &[bundle_component]).unwrap());
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
            0,
            0,
            0,
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

    #[test]
    fn executable_validation_examples_match_lean_guards() {
        let raw_line = RawCartLine {
            sku: sku(),
            price: 2_500,
            cost: 1_200,
            quantity: 2,
            discount: 500,
            weight: 3,
        };
        let line = validate_cart_line(raw_line.clone()).unwrap();
        assert_eq!(line_gross_total(&line).unwrap(), 5_000);
        assert_eq!(line_net_total(&line).unwrap(), 4_500);
        assert_eq!(line_weight_total(&line).unwrap(), 6);
        assert_eq!(
            validate_cart_line(RawCartLine {
                discount: 6_000,
                ..raw_line.clone()
            })
            .unwrap_err(),
            ValidationError::LineDiscountExceedsGross
        );

        let raw_order = RawOrder {
            id: OrderId::new(1),
            items: vec![raw_line],
            coupon_amount: 1_000,
            shipping_method: ShippingMethod::new(500, 10_000, 20),
            tax: 350,
            currency: Currency::USD,
            status: OrderStatus::New,
            total: 4_350,
        };
        assert_eq!(validate_order(raw_order.clone()).unwrap().total(), 4_350);
        assert_eq!(
            validate_order(RawOrder {
                total: 4_351,
                ..raw_order.clone()
            })
            .unwrap_err(),
            ValidationError::OrderTotalMismatch
        );
        assert_eq!(
            validate_order(RawOrder {
                shipping_method: ShippingMethod::new(500, 10_000, 5),
                ..raw_order
            })
            .unwrap_err(),
            ValidationError::ShippingUnavailable
        );

        let raw_stock = RawStockState {
            sku: sku(),
            total: 10,
            reserved: 3,
        };
        let stock = validate_stock_state(raw_stock.clone()).unwrap();
        let policy = RawChannelPricePolicy {
            min_price: 1_000,
            max_price: 2_000,
        };
        let feed = validate_feed_line(RawProductFeedLine {
            sku: sku(),
            channel: SalesChannel::MarketplaceChannel(Marketplace::EtsyLike),
            price: 1_500,
            currency: Currency::USD,
            stock: 5,
            stock_state: raw_stock.clone(),
            price_policy: policy,
        })
        .unwrap();
        assert_eq!(*feed.stock(), 5);
        assert_eq!(available_stock(&stock), 7);

        let ledger = validate_payment_ledger(RawPaymentLedger {
            captured: 10_000,
            refunded: 2_500,
        })
        .unwrap();
        let refund = validate_refund(RawRefund { amount: 5_000 }, ledger.clone()).unwrap();
        assert_eq!(issue_valid_refund(&refund).unwrap().refunded(), 7_500);
        assert_eq!(
            validate_refund(RawRefund { amount: 8_000 }, ledger).unwrap_err(),
            ValidationError::RefundExceedsRemaining
        );

        let versioned = validate_versioned_stock(raw_stock, 4).unwrap();
        let next = validate_compare_and_swap_reservation(versioned.clone(), 4, 4).unwrap();
        assert_eq!(next.version(), 5);
        assert_eq!(next.stock().reserved(), 7);
        assert!(validate_compare_and_swap_reservation(versioned, 4, 3).is_err());
    }

    #[test]
    fn finance_risk_post_purchase_and_tax_examples_match_lean() {
        let bps = BasisPoints::try_new(1_000).unwrap();
        let rate = ExchangeRate::try_new(Currency::USD, Currency::EUR, 9, 10, epoch()).unwrap();
        let tax_rate = TaxRate::new(bps);
        assert_eq!(convert_money_floor(100, &rate).unwrap(), 90);
        assert_eq!(
            tax_amount_rounded(RoundingMode::HalfUp, &tax_rate, 999).unwrap(),
            100
        );
        assert_eq!(abs_diff_nat(10, 4), 6);
        assert_eq!(abs_diff_nat(4, 10), 6);

        let policy = FraudPolicy::new(3, 10, 1);
        assert!(coupon_uses_allowed(&policy, 2));
        assert!(orders_per_hour_allowed(&policy, 10));
        assert!(can_perform(Role::Admin, Action::DeleteOrder));
        assert!(can_perform(Role::Warehouse, Action::AdjustStock));
        assert!(!can_perform(Role::Customer, Action::IssueRefund));
        assert!(can_perform(Role::Support, Action::CreateSupportCase));
        assert!(can_perform(Role::Finance, Action::ApproveReturn));

        let redemption = GiftCardRedemption::try_new(GiftCard::new(5_000, epoch()), 1_200).unwrap();
        assert_eq!(gift_card_balance_after_redeem(&redemption), 3_800);
        let events = vec![CashflowEvent::new(500, 0), CashflowEvent::new(125, 25)];
        assert_eq!(cashflow_inflows_total(&events).unwrap(), 625);
        assert_eq!(cashflow_outflows_total(&events).unwrap(), 25);

        assert_eq!(
            tax_for_treatment(TaxTreatment::Taxable, RoundingMode::HalfUp, &tax_rate, 999).unwrap(),
            100
        );
        assert_eq!(
            tax_for_treatment(TaxTreatment::Exempt, RoundingMode::HalfUp, &tax_rate, 999).unwrap(),
            0
        );
        assert_eq!(seller_tax_due_for_facilitator(true, 250), 0);
        assert_eq!(seller_tax_due_for_facilitator(false, 250), 250);
    }

    #[test]
    fn event_sourcing_semantic_projection_examples_match_lean() {
        let state = ValidSystemState::new(
            StockState::try_new(Sku::new(6_101), 10, 3).unwrap(),
            PaymentLedger::try_new(100, 20).unwrap(),
            11,
            2,
            5,
        );
        let events = vec![
            DomainEvent::PaymentCaptured(OrderId::new(1), 50),
            DomainEvent::RefundIssued(OrderId::new(1), 30),
            DomainEvent::StockReserved(Sku::new(6_101), 2),
            DomainEvent::TaxLiabilityRecorded(9, 4),
            DomainEvent::LeadConverted(LeadId::new(7), OpportunityId::new(8)),
            DomainEvent::ShipmentDelivered(ShipmentId::new(12)),
        ];
        let next = replay_domain_events(state.clone(), &events).unwrap();
        assert_eq!(next.stock().reserved(), 5);
        assert_eq!(next.ledger().captured(), 150);
        assert_eq!(next.ledger().refunded(), 50);
        assert_eq!(*next.tax_liability(), 15);
        assert_eq!(*next.crm_event_count(), 3);
        assert_eq!(*next.logistics_event_count(), 6);

        assert!(
            replay_domain_events(
                state.clone(),
                &[DomainEvent::RefundIssued(OrderId::new(1), 200)]
            )
            .is_err()
        );

        let key = IdempotencyKey::new(77);
        let event = DomainEvent::PaymentCaptured(OrderId::new(1), 25);
        let idempotency = IdempotencyState::new(vec![]);
        let (after_first, idempotency) =
            apply_idempotent_domain_event(key, &event, state, idempotency).unwrap();
        let (after_second, idempotency) =
            apply_idempotent_domain_event(key, &event, after_first.clone(), idempotency).unwrap();
        assert_eq!(after_first.ledger().captured(), 125);
        assert_eq!(after_second.ledger().captured(), 125);
        assert_eq!(idempotency.processed().len(), 1);

        let domain_step = ValidDomainEventStep::from_event(
            after_second.clone(),
            DomainEvent::LeadConverted(LeadId::new(7), OpportunityId::new(8)),
        )
        .unwrap();
        assert_eq!(*domain_step.after().crm_event_count(), 3);
        assert!(
            ValidDomainEventStep::from_event(
                after_second,
                DomainEvent::OrderPlaced(OrderId::new(1), 25)
            )
            .is_err()
        );
    }

    #[test]
    fn crm_logistics_and_tax_constructors_reject_invalid_edges() {
        let customer = Customer::new(CustomerId::new(1), CustomerKind::WholesaleAccount, true);
        assert!(
            CRMAccount::try_new(
                AccountId::new(1),
                customer.clone(),
                AccountTier::Standard,
                CRMAccountStatus::Active,
                100,
                101,
            )
            .is_err()
        );
        let account = CRMAccount::try_new(
            AccountId::new(1),
            customer.clone(),
            AccountTier::Strategic,
            CRMAccountStatus::Active,
            1_000,
            100,
        )
        .unwrap();
        let contact = CRMContact::new(
            ContactId::new(2),
            AccountId::new(1),
            CustomerId::new(1),
            ContactKind::Primary,
            Role::Manager,
            SubscriptionStatus::Subscribed,
            ConsentStatus::Granted,
            DataProcessingPermission::new(
                ConsentPurpose::Marketing,
                ProcessingBasis::Consent,
                true,
            ),
        );
        let account_contact = CRMAccountContact::try_new(account.clone(), contact.clone()).unwrap();
        assert!(contact_can_receive_marketing(&contact));
        assert!(
            CRMAccountContact::try_new(
                account.clone(),
                CRMContact::new(
                    ContactId::new(3),
                    AccountId::new(99),
                    CustomerId::new(1),
                    ContactKind::Buyer,
                    Role::Manager,
                    SubscriptionStatus::Subscribed,
                    ConsentStatus::Granted,
                    DataProcessingPermission::new(
                        ConsentPurpose::Marketing,
                        ProcessingBasis::Consent,
                        true,
                    ),
                )
            )
            .is_err()
        );

        let won_probability = BasisPoints::try_new(10_000).unwrap();
        let bad_probability = BasisPoints::try_new(9_000).unwrap();
        assert!(
            SalesOpportunity::try_new(
                OpportunityId::new(1),
                AccountId::new(1),
                ContactId::new(2),
                Some(LeadId::new(1)),
                OpportunityStage::Won,
                100,
                Currency::USD,
                won_probability,
                epoch(),
                epoch(),
                epoch(),
            )
            .is_ok()
        );
        assert!(
            SalesOpportunity::try_new(
                OpportunityId::new(1),
                AccountId::new(1),
                ContactId::new(2),
                None,
                OpportunityStage::Won,
                100,
                Currency::USD,
                bad_probability,
                epoch(),
                epoch(),
                epoch(),
            )
            .is_err()
        );

        let warehouse = Warehouse::new(1, "main".to_owned());
        let stock = StockState::try_new(sku(), 10, 0).unwrap();
        let allocation =
            Allocation::try_new(InventoryNode::new(warehouse.clone(), stock), 2).unwrap();
        let fulfillment = DistinctFulfillmentPlan::try_new(2, vec![allocation]).unwrap();
        let zone = ShippingZone::new(1, "local".to_owned());
        let package = Package::new(6, 1);
        let quote = CarrierQuote::try_new(
            CarrierService::new(55, zone.clone(), 10, 5, days(2)),
            package.clone(),
            6,
        )
        .unwrap();
        let destination = ShippingDestination::new(1, zone, 12_345);
        assert!(
            validate_logistics_shipment_plan(
                ShipmentId::new(2),
                paid_order(),
                fulfillment.clone(),
                quote.clone(),
                warehouse.clone(),
                1,
                12_345,
                epoch(),
                epoch(),
            )
            .is_ok()
        );
        let plan = LogisticsShipmentPlan::try_new(
            ShipmentId::new(1),
            paid_order(),
            fulfillment,
            package,
            quote,
            warehouse,
            destination,
            epoch(),
            epoch(),
        )
        .unwrap();
        assert!(validate_carrier_handoff(plan, 9_001, epoch(), epoch()).is_ok());
        assert!(
            WarehouseTransfer::try_new(
                TransferId::new(1),
                sku(),
                Warehouse::new(1, "a".to_owned()),
                Warehouse::new(1, "b".to_owned()),
                StockState::try_new(sku(), 10, 0).unwrap(),
                2,
                1,
                1,
            )
            .is_err()
        );

        let tax_rate = TaxRate::new(BasisPoints::try_new(1_000).unwrap());
        let invoice_line = TaxInvoiceLine::try_new(
            sku(),
            2,
            100,
            20,
            TaxTreatment::Taxable,
            tax_rate.clone(),
            RoundingMode::Floor,
            180,
            18,
            198,
        )
        .unwrap();
        assert_eq!(invoice_line_grand_total(&[invoice_line]).unwrap(), 198);
        assert!(TaxExclusivePrice::try_new(100, 10, 111).is_err());
        assert!(
            account_contact.account().open_balance() <= account_contact.account().lifetime_value()
        );
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
