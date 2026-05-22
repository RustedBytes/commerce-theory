use commerce_theory::*;

fn epoch() -> Timestamp {
    unix_epoch_timestamp()
}

fn later(n: Nat) -> Timestamp {
    epoch() + days(n)
}

fn sku(n: Nat) -> Sku {
    Sku::new(n)
}

fn ledger_account(id: Id, name: &str) -> LedgerAccount {
    LedgerAccount::new(id, name.to_owned())
}

fn accounts() -> AccountingAccounts {
    AccountingAccounts::new(
        ledger_account(1, "cash"),
        ledger_account(2, "deferred"),
        ledger_account(3, "revenue"),
        ledger_account(4, "refunds"),
        ledger_account(5, "inventory"),
        ledger_account(6, "cogs"),
    )
}

fn advanced_accounts() -> AdvancedAccountingAccounts {
    AdvancedAccountingAccounts::new(
        accounts(),
        ledger_account(7, "receivable"),
        ledger_account(8, "payable"),
        ledger_account(9, "tax"),
        ledger_account(10, "marketplace clearing"),
        ledger_account(11, "marketplace fees"),
        ledger_account(12, "chargeback reserve"),
        ledger_account(13, "chargeback expense"),
        ledger_account(14, "realized fx gain"),
        ledger_account(15, "realized fx loss"),
        ledger_account(16, "unrealized fx gain"),
        ledger_account(17, "unrealized fx loss"),
    )
}

fn cart_line_for(line_sku: Sku, price: Money, quantity: Quantity) -> CartLine {
    CartLine::try_new(line_sku, price, price / 2, quantity, 0, 1).unwrap()
}

fn free_shipping() -> ShippingMethod {
    ShippingMethod::new(0, 0, 100)
}

fn order_with(
    id: Nat,
    status: OrderStatus,
    line_sku: Sku,
    price: Money,
    quantity: Quantity,
) -> Order {
    let items = vec![cart_line_for(line_sku, price, quantity)];
    let total = order_total(&free_shipping(), 0, 0, &items).unwrap();
    Order::try_new(
        OrderId::new(id),
        items,
        0,
        free_shipping(),
        0,
        Currency::USD,
        status,
        total,
    )
    .unwrap()
}

fn customer() -> Customer {
    Customer::new(CustomerId::new(1), CustomerKind::WholesaleAccount, true)
}

fn crm_account() -> CRMAccount {
    CRMAccount::try_new(
        AccountId::new(1),
        customer(),
        AccountTier::Strategic,
        CRMAccountStatus::Active,
        2_000,
        100,
    )
    .unwrap()
}

fn data_permission() -> DataProcessingPermission {
    DataProcessingPermission::new(ConsentPurpose::Marketing, ProcessingBasis::Consent, true)
}

fn crm_contact() -> CRMContact {
    CRMContact::new(
        ContactId::new(2),
        AccountId::new(1),
        CustomerId::new(1),
        ContactKind::Primary,
        Role::Manager,
        SubscriptionStatus::Subscribed,
        ConsentStatus::Granted,
        data_permission(),
    )
}

fn support_case(status: SupportCaseStatus, order_id: Option<OrderId>) -> SupportCase {
    SupportCase::try_new(
        SupportCaseId::new(1),
        AccountId::new(1),
        ContactId::new(2),
        order_id,
        status,
        SupportPriority::High,
        epoch(),
        later(1),
        later(5),
    )
    .unwrap()
}

fn warehouse(id: Id) -> Warehouse {
    Warehouse::new(id, format!("warehouse-{id}"))
}

fn allocation_for(line_sku: Sku, quantity: Quantity, warehouse: &Warehouse) -> Allocation {
    let stock = StockState::try_new(line_sku, 20, 0).unwrap();
    Allocation::try_new(InventoryNode::new(warehouse.clone(), stock), quantity).unwrap()
}

fn fulfillment_for(
    line_sku: Sku,
    quantity: Quantity,
    warehouse: &Warehouse,
) -> DistinctFulfillmentPlan {
    DistinctFulfillmentPlan::try_new(
        quantity,
        vec![allocation_for(line_sku, quantity, warehouse)],
    )
    .unwrap()
}

fn carrier_service(zone: ShippingZone) -> CarrierService {
    CarrierService::new(77, zone, 100, 5, days(2))
}

fn logistics_plan_for(order: Order) -> LogisticsShipmentPlan {
    let line = &order.items()[0];
    let line_sku = line.sku();
    let quantity = line.quantity();
    let wh = warehouse(1);
    let fulfillment = fulfillment_for(line_sku, quantity, &wh);
    let zone = ShippingZone::new(1, "local".to_owned());
    let package = Package::new(10, 1);
    let quote = CarrierQuote::try_new(carrier_service(zone.clone()), package.clone(), 7).unwrap();
    let destination = ShippingDestination::new(1, zone, 12_345);
    LogisticsShipmentPlan::try_new(
        ShipmentId::new(1),
        order,
        fulfillment,
        package,
        quote,
        wh,
        destination,
        epoch(),
        later(3),
    )
    .unwrap()
}

fn logistics_plan() -> LogisticsShipmentPlan {
    logistics_plan_for(order_with(10, OrderStatus::Paid, sku(10), 500, 2))
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

fn dropship_offer() -> DropshipOffer {
    DropshipOffer::try_new(sku(10), supplier(), 40, 100, 2, 20, Currency::USD, true).unwrap()
}

fn dropship_costs() -> DropshipProfitCosts {
    DropshipProfitCosts::new(40, 5, 3, 2, 1, 1, 1, 1)
}

fn tax_rate() -> TaxRate {
    TaxRate::new(BasisPoints::try_new(1_000).unwrap())
}

fn tax_jurisdiction() -> TaxJurisdiction {
    TaxJurisdiction::new(1, "local".to_owned(), TaxRegime::SalesTax, Currency::USD)
}

fn taxable_invoice_line() -> TaxInvoiceLine {
    TaxInvoiceLine::try_new(
        sku(10),
        2,
        100,
        20,
        TaxTreatment::Taxable,
        tax_rate(),
        RoundingMode::Floor,
        180,
        18,
        198,
    )
    .unwrap()
}

fn competitor_offer(price: Money) -> CompetitorOffer {
    CompetitorOffer::new(
        CompetitorId::new(price),
        sku(10),
        price,
        Currency::USD,
        true,
        true,
        epoch(),
    )
}

fn opportunity_candidate(expected_profit: Money) -> DropshipOpportunityCandidate {
    DropshipOpportunityCandidate::try_new(
        sku(expected_profit),
        2,
        100,
        50,
        expected_profit,
        20,
        120,
        dropship_costs(),
    )
    .unwrap()
}

#[test]
fn foundation_accounting_and_finance_paths_are_covered() {
    let decimal = DecimalMoney::new(-123, 2);
    assert_eq!(decimal.coefficient(), -123);
    assert_eq!(decimal.scale(), 2);
    assert_eq!(
        format!("{}", ValidationError::RefundExceedsRemaining),
        "refund exceeds remaining amount"
    );
    assert!(checked_add(Nat::MAX, 1, "overflow").is_err());
    assert!(checked_mul(Nat::MAX, 2, "overflow").is_err());
    assert!(checked_div(1, 0, "zero").is_err());
    assert_eq!(checked_sum([1, 2, 3], "sum").unwrap(), 6);
    assert_eq!(nat_sub(1, 2), 0);
    assert!(timestamp_from_ymdhms(2024, 2, 29, 12, 0, 0).is_some());
    assert!(timestamp_from_ymdhms(2024, 13, 1, 0, 0, 0).is_none());
    assert_eq!(timestamp_age(later(2), later(1)), days(1));
    assert_eq!(round_div(RoundingMode::Floor, 5, 2).unwrap(), 2);
    assert_eq!(round_div(RoundingMode::Ceiling, 5, 2).unwrap(), 3);
    assert_eq!(round_div(RoundingMode::HalfUp, 5, 2).unwrap(), 3);
    assert!(round_money(RoundingMode::Floor, 1, 0).is_err());
    assert_eq!(floor_rounding_remainder(7, 3).unwrap(), 1);
    assert!(floor_rounding_remainder(7, 0).is_err());
    assert_eq!(
        floor_rounded_lines_remainder_total(10, &[11, 12, 13]).unwrap(),
        6
    );

    let usd = MoneyIn::<Usd>::new(100);
    assert_eq!(
        usd.checked_add(MoneyIn::<Usd>::new(25)).unwrap().amount(),
        125
    );
    assert_eq!(usd.saturating_sub(MoneyIn::<Usd>::new(150)).amount(), 0);
    assert_eq!(MoneyIn::<Eur>::zero().currency(), Currency::EUR);
    assert!(same_currency(
        &MoneyAmount::new(10, Currency::USD),
        &MoneyAmount::new(20, Currency::USD)
    ));
    assert_eq!(
        apply_bps(BasisPoints::try_new(2_500).unwrap(), 200).unwrap(),
        50
    );
    assert_eq!(
        round_bps_amount(
            RoundingMode::HalfUp,
            999,
            BasisPoints::try_new(1_000).unwrap()
        )
        .unwrap(),
        100
    );
    assert_eq!(profit_amount(100, 125), 0);
    assert_eq!(profit_loss_amount(100, 125).unwrap(), -25);
    assert!(BasisPoints::try_new(10_001).is_err());

    let accts = accounts();
    let advanced = advanced_accounts();
    let balanced = BalancedJournalEntry::try_new(vec![
        debit(ledger_account(50, "debit"), 10),
        credit(ledger_account(51, "credit"), 10),
    ])
    .unwrap();
    assert_eq!(debit_total(balanced.postings()).unwrap(), 10);
    assert!(BalancedJournalEntry::try_new(vec![debit(ledger_account(52, "only"), 5)]).is_err());
    assert!(payment_captured_journal(&accts, 100).is_ok());
    assert!(refund_issued_journal(&accts, 25).is_ok());
    assert!(invoice_accrual_journal(&advanced, 90, 10, 100).is_ok());
    assert!(invoice_accrual_journal(&advanced, 90, 10, 99).is_err());
    assert!(cash_sale_journal(&advanced, 90, 10, 100).is_ok());
    assert!(receivable_collection_journal(&advanced, 100).is_ok());
    assert!(supplier_bill_journal(&advanced, 40).is_ok());
    assert!(supplier_payment_journal(&advanced, 40).is_ok());
    assert!(marketplace_sale_clearing_journal(&advanced, 200).is_ok());
    assert!(marketplace_settlement_journal(&advanced, 200, 20, 180).is_ok());
    assert!(marketplace_settlement_journal(&advanced, 200, 20, 181).is_err());
    assert!(marketplace_payout_reconciliation_journal(&advanced, 200, 20, 10, 5, 15, 150).is_ok());
    assert!(chargeback_reserve_journal(&advanced, 10).is_ok());
    assert!(chargeback_settlement_journal(&advanced, 10).is_ok());
    assert!(unrealized_fx_gain_journal(&advanced, 10).is_ok());
    assert!(unrealized_fx_loss_journal(&advanced, 10).is_ok());
    assert!(realized_fx_gain_journal(&advanced, 10).is_ok());
    assert!(realized_fx_loss_journal(&advanced, 10).is_ok());

    let rate = ExchangeRate::try_new(Currency::USD, Currency::EUR, 9, 10, epoch()).unwrap();
    assert!(fx_quote_fresh(later(1), days(2), &rate));
    assert_eq!(
        convert_money_rounded(RoundingMode::Ceiling, 101, &rate).unwrap(),
        91
    );
    assert_eq!(convert_money_floor(101, &rate).unwrap(), 90);
    assert!(ExchangeRate::try_new(Currency::USD, Currency::EUR, 1, 0, epoch()).is_err());
    assert!(TaxCalculation::try_new(100, tax_rate(), RoundingMode::Floor, 10, 110).is_ok());
    assert!(TaxCalculation::try_new(100, tax_rate(), RoundingMode::Floor, 9, 109).is_err());
    assert!(
        CarrierQuote::try_new(
            carrier_service(ShippingZone::new(2, "zone".to_owned())),
            Package::new(10, 1),
            5
        )
        .is_ok()
    );
    assert!(
        CarrierQuote::try_new(
            carrier_service(ShippingZone::new(3, "zone".to_owned())),
            Package::new(101, 1),
            5
        )
        .is_err()
    );
    assert_eq!(abs_diff_nat(4, 10), 6);
    assert!(ReconciliationWithinTolerance::try_new(100, 103, 3).is_ok());
    assert!(ReconciliationWithinTolerance::try_new(100, 104, 3).is_err());
}

#[test]
fn catalog_pricing_orders_marketplace_and_marketing_paths_are_covered() {
    let product = Product::new(
        ProductId::new(1),
        Brand::new(1, "brand".to_owned()),
        Category::new(2, "category".to_owned()),
        ProductStatus::Active,
    );
    let variant = ProductVariant::new(VariantId::new(1), ProductId::new(1), sku(10), true);
    let entry = ProductCatalogEntry::try_new(product.clone(), variant.clone()).unwrap();
    assert_eq!(entry.product(), &product);
    assert!(
        ProductCatalogEntry::try_new(
            product,
            ProductVariant::new(VariantId::new(2), ProductId::new(99), sku(11), true)
        )
        .is_err()
    );
    assert!(
        ValidListingContent::try_new(
            ListingContent::new(20, 3, true),
            MarketplaceContentPolicy::new(80, 2)
        )
        .is_ok()
    );
    assert!(
        ValidListingContent::try_new(
            ListingContent::new(100, 1, false),
            MarketplaceContentPolicy::new(80, 2)
        )
        .is_err()
    );

    let line = CartLine::try_new(sku(10), 100, 40, 3, 25, 2).unwrap();
    assert_eq!(line_gross_total(&line).unwrap(), 300);
    assert_eq!(line_cost_total(&line).unwrap(), 120);
    assert_eq!(line_net_total(&line).unwrap(), 275);
    assert_eq!(line_weight_total(&line).unwrap(), 6);
    assert_eq!(
        cart_discount_total(std::slice::from_ref(&line)).unwrap(),
        25
    );
    assert_eq!(cart_quantity_total(std::slice::from_ref(&line)).unwrap(), 3);
    let coupon = Coupon::new(25, 100, 2);
    assert!(coupon_can_be_applied(&coupon, 200, 1));
    assert!(!coupon_can_be_applied(&coupon, 200, 2));
    assert_eq!(subtotal_after_coupon_amount(10, 20), 0);
    let method = ShippingMethod::new(50, 1_000, 10);
    assert!(shipping_available(&method, 6));
    assert_eq!(shipping_charge(&method, 900), 50);
    assert_eq!(shipping_charge(&method, 1_000), 0);

    let order = order_with(1, OrderStatus::New, sku(10), 500, 2);
    assert_eq!(order.items().len(), 1);
    assert!(can_order_transition(OrderStatus::New, OrderStatus::Paid));
    assert!(!can_order_transition(
        OrderStatus::Cancelled,
        OrderStatus::Paid
    ));
    for transition in [
        CanOrderTransition::NewPaid,
        CanOrderTransition::NewCancelled,
        CanOrderTransition::NewBackordered,
        CanOrderTransition::PaidPacked,
        CanOrderTransition::PaidRefunded,
        CanOrderTransition::PackedShipped,
        CanOrderTransition::ShippedDelivered,
        CanOrderTransition::DeliveredRefunded,
        CanOrderTransition::BackorderedPaid,
        CanOrderTransition::BackorderedCancelled,
    ] {
        assert_eq!(
            CanOrderTransition::from_statuses(transition.source(), transition.target()),
            Some(transition)
        );
    }
    let created = TypedPayment::<CreatedPayment>::try_new(
        PaymentId::new(1),
        OrderId::new(1),
        1_000,
        Currency::USD,
    )
    .unwrap();
    let (_, captured) = capture_payment(authorize_payment(created));
    assert!(
        mark_paid(
            TypedOrder::<NewOrder>::try_new(OrderId::new(1), 1_000, Currency::USD).unwrap(),
            &captured
        )
        .is_ok()
    );
    assert!(
        mark_paid(
            TypedOrder::<NewOrder>::try_new(OrderId::new(2), 1_000, Currency::USD).unwrap(),
            &captured
        )
        .is_err()
    );

    let listing = MarketplaceListing::new(
        sku(10),
        Marketplace::AmazonLike,
        55,
        1_000,
        Currency::USD,
        5,
        ListingStatus::Active,
    );
    assert!(listing_active(&listing));
    assert!(listing_in_stock(&listing));
    assert!(listing_can_be_advertised(&listing));
    let stock = StockState::try_new(sku(10), 10, 0).unwrap();
    let synced = SyncedMarketplaceListing::try_new(listing.clone(), stock).unwrap();
    assert_eq!(synced.listing(), &listing);
    assert!(
        SyncedMarketplaceListing::try_new(
            listing.clone(),
            StockState::try_new(sku(99), 10, 0).unwrap()
        )
        .is_err()
    );
    let policy = ChannelPricePolicy::try_new(500, 2_000).unwrap();
    assert!(valid_channel_price(&policy, 1_000));
    assert!(
        SafeProductFeedLine::try_new(
            sku(10),
            SalesChannel::MarketplaceChannel(Marketplace::AmazonLike),
            1_000,
            Currency::USD,
            5,
            stock,
            policy
        )
        .is_ok()
    );
    let fee_rate = BasisPoints::try_new(1_000).unwrap();
    assert_eq!(
        marketplace_fee_rounded(RoundingMode::Floor, 1_000, fee_rate).unwrap(),
        100
    );
    assert_eq!(
        marketplace_payout_rounded(RoundingMode::Floor, 1_000, fee_rate).unwrap(),
        100
    );
    let fee =
        MarketplaceFeeLedger::try_new(1_000, fee_rate, RoundingMode::Floor, 100, 900).unwrap();
    assert!(MarketplaceFeeLedger::try_new(1_000, fee_rate, RoundingMode::Floor, 99, 901).is_err());
    assert!(
        MarketplacePayoutCalculation::try_new(1_000, fee_rate, RoundingMode::Floor, 100).is_ok()
    );
    assert!(
        MarketplaceOrder::try_new(
            Marketplace::AmazonLike,
            MarketplaceOrderId::new(10),
            order.clone(),
            order.total(),
            fee
        )
        .is_ok()
    );

    assert!(destination_matches_marketplace(
        AdDestination::MarketplaceListing(Marketplace::AmazonLike, 55),
        Marketplace::AmazonLike
    ));
    assert!(!destination_matches_marketplace(
        AdDestination::Website,
        Marketplace::AmazonLike
    ));
    let campaign = MarketingCampaign::try_new(
        CampaignId::new(1),
        AdPlatform::GoogleLike,
        AdType::Search,
        AdDestination::Website,
        CampaignStatus::Active,
        1_000,
        250,
        10_000,
        500,
        25,
        2_000,
    )
    .unwrap();
    assert!(
        MarketingCampaign::try_new(
            CampaignId::new(2),
            AdPlatform::MetaLike,
            AdType::Social,
            AdDestination::Website,
            CampaignStatus::Active,
            100,
            101,
            10,
            1,
            1,
            1
        )
        .is_err()
    );
    assert_eq!(
        campaigns_spend_total(std::slice::from_ref(&campaign)).unwrap(),
        250
    );
    assert_eq!(
        campaigns_budget_total(std::slice::from_ref(&campaign)).unwrap(),
        1_000
    );
    assert!(ClickAttributedCampaign::try_new(campaign.clone()).is_ok());
    assert!(meets_roas_target(&campaign, 2, 1).unwrap());
    assert!(meets_roi_target(500, 100, 2, 1).unwrap());
    assert!(Funnel::try_new(100, 50, 25, 10).is_ok());
    assert!(Funnel::try_new(100, 50, 60, 10).is_err());
    assert!(can_retarget(ConsentStatus::Granted));
    assert!(can_send_marketing_message(SubscriptionStatus::Subscribed));
    let credit = AttributionCredit::new(CampaignId::new(1), order.id(), 100);
    assert_eq!(
        attribution_credit_total(std::slice::from_ref(&credit)).unwrap(),
        100
    );
    assert!(attribution_credits_match_order(
        &order,
        std::slice::from_ref(&credit)
    ));
    let attribution = OrderAttributionLedger::try_new(order, vec![credit]).unwrap();
    assert!(MatchedOrderAttributionLedger::try_new(attribution).is_ok());
    let variant_a = ExperimentVariant::try_new(1, 40, 100, 10).unwrap();
    let variant_b = ExperimentVariant::try_new(2, 60, 100, 20).unwrap();
    assert_eq!(
        experiment_traffic_total(&[variant_a.clone(), variant_b.clone()]).unwrap(),
        100
    );
    assert!(Experiment::try_new(1, vec![variant_a, variant_b]).is_ok());
}

#[test]
fn inventory_b2b_dropship_competitor_and_merchandising_paths_are_covered() {
    let stock = StockState::try_new(sku(10), 10, 3).unwrap();
    assert_eq!(available_stock(&stock), 7);
    assert!(can_reserve(&stock, 7));
    assert!(reserve_stock(&stock, 8).is_err());
    let versioned = VersionedStock::try_new(sku(10), 10, 3, 4).unwrap();
    assert!(reserve_versioned_stock(&versioned, 2, 3).is_err());
    assert_eq!(
        reserve_versioned_stock(&versioned, 2, 4).unwrap().version(),
        5
    );
    let wh = warehouse(1);
    let bin = BinStock::new(sku(10), BinLocation::new(wh.clone(), 10), 5);
    assert!(PickTask::try_new(sku(10), 5, bin.clone()).is_ok());
    assert!(PickTask::try_new(sku(10), 6, bin).is_err());
    assert!(PackTask::try_new(5, 4).is_ok());
    assert!(PackTask::try_new(5, 6).is_err());
    assert!(WarehouseShipment::try_new(5, 5).is_ok());
    assert!(WarehouseShipment::try_new(5, 6).is_err());
    let allocation = allocation_for(sku(10), 2, &wh);
    assert_eq!(
        allocations_total(std::slice::from_ref(&allocation)).unwrap(),
        2
    );
    assert_eq!(
        allocations_available_total(std::slice::from_ref(&allocation)).unwrap(),
        20
    );
    assert!(allocation_keys_distinct(std::slice::from_ref(&allocation)));
    assert!(FulfillmentPlan::try_new(2, vec![allocation.clone()]).is_ok());
    assert!(FulfillmentPlan::try_new(3, vec![allocation.clone()]).is_err());
    assert!(DistinctFulfillmentPlan::try_new(2, vec![allocation.clone()]).is_ok());
    assert!(
        DistinctFulfillmentPlan::try_new(4, vec![allocation.clone(), allocation.clone()]).is_err()
    );
    assert!(release_reserved_stock(&stock, 2).is_ok());
    assert!(confirm_reserved_shipment(&stock, 2).is_ok());
    let attempt = ReservationAttempt::new(versioned, 1, 4);
    assert!(commit_reservation_attempt(&attempt).is_some());
    assert!(
        ConcurrentReservationConflict::try_new(
            attempt.clone(),
            ReservationAttempt::new(versioned, 2, 4)
        )
        .is_ok()
    );
    let reservation =
        TimedReservation::try_new(stock, 2, epoch(), later(1), ReservationStatus::Active).unwrap();
    assert!(reservation_active_at(epoch(), &reservation));
    assert!(reservation_expired_at(later(2), &reservation));
    assert!(release_expired_reservation(&reservation, later(2)).is_ok());
    assert!(BackorderRequest::try_new(sku(10), 10, 4, 6).is_ok());
    let window = PreorderWindow::try_new(sku(10), epoch(), later(3), 5).unwrap();
    assert!(PreorderReservation::try_new(window.clone(), 5, later(1)).is_ok());
    assert!(PreorderReservation::try_new(window, 6, later(1)).is_err());
    let unit_a = SerializedInventoryUnit::new(sku(10), SerialNumber::new(1), wh.clone(), false);
    let unit_b = SerializedInventoryUnit::new(sku(10), SerialNumber::new(2), wh.clone(), false);
    assert!(serial_numbers_distinct(&[unit_a.clone(), unit_b.clone()]));
    assert!(SerializedInventorySet::try_new(vec![unit_a.clone(), unit_b]).is_ok());
    assert!(SerializedInventorySet::try_new(vec![unit_a.clone(), unit_a]).is_err());
    let lot = InventoryLot::new(sku(10), 1, wh.clone(), later(3), 10);
    assert!(lot_usable_at(epoch(), &lot));
    assert!(SkuSubstitution::try_new(sku(9), sku(10), stock, 5).is_ok());
    let second = warehouse(2);
    let split_plan = DistinctFulfillmentPlan::try_new(
        4,
        vec![
            allocation_for(sku(10), 2, &wh),
            allocation_for(sku(11), 2, &second),
        ],
    )
    .unwrap();
    assert!(SplitFulfillmentPlan::try_new(split_plan, wh.clone(), second).is_ok());
    assert_eq!(
        timed_allocations_total(std::slice::from_ref(&allocation))
            .unwrap()
            .ret,
        2
    );
    assert_eq!(
        allocation_key_support(std::slice::from_ref(&allocation)).len(),
        1
    );
    assert_eq!(
        allocation_quantity_for_key(
            std::slice::from_ref(&allocation),
            allocation_key(&allocation)
        )
        .unwrap(),
        2
    );
    assert_eq!(
        allocation_quantity_by_key(&[allocation])
            .unwrap()
            .values()
            .copied()
            .sum::<Nat>(),
        2
    );

    assert!(customer_can_buy_wholesale(&customer()));
    assert!(!payment_terms_allowed(
        TradeMode::Retail,
        PaymentTerms::NetDays(30)
    ));
    let entry =
        TradePriceBookEntry::try_new(sku(10), Currency::USD, 40, 100, 80, 20, 10, 2).unwrap();
    assert_eq!(unit_price_for_trade_mode(TradeMode::Retail, &entry), 100);
    assert_eq!(unit_price_for_trade_mode(TradeMode::Wholesale, &entry), 80);
    let retail = RetailLine::try_new(entry.clone(), 2, 10).unwrap();
    assert_eq!(retail_line_net_total(&retail).unwrap(), 190);
    let wholesale = WholesaleLine::try_new(entry, 2, 5).unwrap();
    assert_eq!(wholesale_line_net_total(&wholesale).unwrap(), 155);
    assert_eq!(
        wholesale_retail_equivalent_total(std::slice::from_ref(&wholesale)).unwrap(),
        200
    );
    let credit_account = WholesaleCreditAccount::try_new(customer(), 1_000, 100).unwrap();
    assert!(can_place_wholesale_credit_order(&credit_account, 155));

    let capacity = SupplierDailyCapacity::try_new(supplier(), 10, 4).unwrap();
    assert!(supplier_can_receive_orders(capacity.supplier()));
    assert!(can_add_supplier_orders(&capacity, 6));
    let offer = dropship_offer();
    assert!(dropship_offer_can_be_sold(&offer));
    let reservation = SupplierReservation::try_new(
        offer.clone(),
        supplier(),
        2,
        SupplierReservationStatus::Confirmed,
    )
    .unwrap();
    assert!(reservation_confirmed(&reservation));
    let line = DropshipLine::try_new(offer.clone(), 2, 5).unwrap();
    assert!(ReservedDropshipLine::try_new(line.clone(), reservation).is_ok());
    assert_eq!(dropship_line_sale_gross(&line).unwrap(), 200);
    assert_eq!(dropship_line_customer_net(&line).unwrap(), 195);
    assert_eq!(dropship_line_supplier_cost(&line).unwrap(), 80);
    assert_eq!(dropship_line_weight(&line).unwrap(), 4);
    assert_eq!(
        dropship_sale_net_total(std::slice::from_ref(&line)).unwrap(),
        195
    );
    let quote = DropshipShippingQuote::new(SupplierId::new(1), 10, 10, days(2));
    assert!(dropship_shipping_quote_can_ship(&quote, 4));
    let po = DropshipPurchaseOrder::try_new(
        supplier(),
        vec![line.clone()],
        quote.clone(),
        DropshipPOStatus::Created,
        90,
    )
    .unwrap();
    assert!(can_dropship_po_transition(
        DropshipPOStatus::Created,
        DropshipPOStatus::Submitted
    ));
    assert!(dropship_sla_safe(&supplier(), &quote, days(5)));
    assert!(
        DropshipFulfillment::try_new(order_with(2, OrderStatus::Paid, sku(10), 100, 2), po, 195)
            .is_ok()
    );
    assert!(DropshipReturnRequest::try_new(line, 1, 50, 20).is_ok());

    let costs = dropship_costs();
    assert_eq!(dropship_profit_costs_total(&costs).unwrap(), 54);
    assert_eq!(revenue_after_discount(100, 20), 80);
    assert_eq!(required_revenue_for_profit(54, 20).unwrap(), 74);
    assert_eq!(required_gross_for_profit(54, 20, 5).unwrap(), 79);
    assert!(GuaranteedDropshipProfitQuote::try_new(100, costs.clone(), 20, 46, 46).is_ok());
    assert!(
        DropshipCostUpperBounds::try_new(
            costs.clone(),
            DropshipProfitCosts::new(50, 5, 3, 2, 1, 1, 1, 1)
        )
        .is_ok()
    );
    assert!(ad_spend_safe_for_min_profit(100, 50, 10, 20));
    assert_eq!(profit_after_ad_spend(100, 50, 10).unwrap(), 40);
    assert_eq!(profit_loss_int(50, 60).unwrap(), -10);

    let best = competitor_offer(95);
    let benchmark = CompetitorPriceBenchmark::try_new(
        sku(10),
        Currency::USD,
        vec![best.clone(), competitor_offer(120)],
        best.clone(),
    )
    .unwrap();
    assert!(competitor_offer_relevant(&best, sku(10), Currency::USD));
    assert!(price_snapshot_fresh(later(1), days(2), epoch()));
    assert!(trust_allows_auto_repricing(TrustLevel::High));
    assert_eq!(customer_net_at_offer_price(100, 5), 95);
    assert_eq!(profit_at_offer_price(100, 5, &costs).unwrap(), 41);
    assert_eq!(profitable_price_floor(&costs, 20, 5).unwrap(), 79);
    assert!(price_profitable_for_min_profit(100, 5, &costs, 20).unwrap());
    assert!(price_at_or_below_competitor(95, 95));
    assert_eq!(undercut_price(95, 10), 85);
    assert_eq!(
        target_price_from_strategy(CompetitivePricingStrategy::Match, 95).unwrap(),
        95
    );
    assert_eq!(
        target_price_from_strategy(CompetitivePricingStrategy::Undercut(5), 95).unwrap(),
        90
    );
    assert_eq!(
        target_price_from_strategy(CompetitivePricingStrategy::Premium(5), 95).unwrap(),
        100
    );
    assert!(CompetitorAwareDropshipOffer::try_new(offer, benchmark, 5, costs, 20).is_err());

    let brand_policy = BrandPricingPolicy::try_new(80, 120).unwrap();
    assert!(advertised_price_allowed(&brand_policy, 90));
    let component = BundleComponent::try_new(sku(10), 2, 10).unwrap();
    assert_eq!(component_required_for_bundles(3, &component).unwrap(), 6);
    assert!(component_can_fulfill_bundles(3, &component).unwrap());
    assert!(BundleReservation::try_new(3, vec![component]).is_ok());
    let promo = AcceptedPromotionSet::try_new(90, 10, 20, 50).unwrap();
    assert!(promotion_set_allowed_by_policy(
        PromotionStackingPolicy::Exclusive,
        1,
        &promo
    ));
    assert!(promotion_set_allowed_by_policy(
        PromotionStackingPolicy::Stackable,
        10,
        &promo
    ));
    assert!(promotion_set_allowed_by_policy(
        PromotionStackingPolicy::StackableWithCap,
        10,
        &promo
    ));
    assert!(
        ValidSearchResultItem::try_new(SearchResultItem::new(sku(10), false, true, true)).is_ok()
    );
}

#[test]
fn tax_post_purchase_risk_crm_and_logistics_paths_are_covered() {
    assert!(seller_collects_tax_for_treatment(TaxTreatment::Taxable));
    assert_eq!(
        tax_for_treatment(
            TaxTreatment::ZeroRated,
            RoundingMode::Floor,
            &tax_rate(),
            100
        )
        .unwrap(),
        0
    );
    assert!(TaxInclusivePrice::try_new(110, 100, 10).is_ok());
    assert!(TaxInclusivePrice::try_new(111, 100, 10).is_err());
    assert!(TaxExclusivePrice::try_new(100, 10, 110).is_ok());
    let line = taxable_invoice_line();
    assert_eq!(
        invoice_line_subtotal_total(std::slice::from_ref(&line)).unwrap(),
        180
    );
    assert_eq!(
        invoice_line_tax_total(std::slice::from_ref(&line)).unwrap(),
        18
    );
    assert_eq!(
        invoice_line_grand_total(std::slice::from_ref(&line)).unwrap(),
        198
    );
    let invoice = TaxInvoice::try_new(
        1,
        epoch(),
        1,
        CustomerId::new(1),
        tax_jurisdiction(),
        Currency::USD,
        vec![line.clone()],
        180,
        18,
        2,
        0,
        200,
    )
    .unwrap();
    assert!(
        OrderTaxInvoiceLink::try_new(
            order_with(3, OrderStatus::Paid, sku(10), 100, 2),
            invoice.clone()
        )
        .is_err()
    );
    let cert = TaxExemptionCertificate::try_new(CustomerId::new(1), 1, epoch(), later(10)).unwrap();
    assert!(certificate_valid_at(&cert, later(1)));
    assert!(B2BTaxExemption::try_new(customer(), tax_jurisdiction(), cert, later(1)).is_ok());
    assert_eq!(seller_tax_due_for_facilitator(true, 18), 0);
    assert!(
        MarketplaceFacilitatorTax::try_new(
            Marketplace::AmazonLike,
            tax_jurisdiction(),
            180,
            tax_rate(),
            RoundingMode::Floor,
            18,
            false,
            18
        )
        .is_ok()
    );
    assert_eq!(invoice_line_floor_tax_rounding_remainder(&line).unwrap(), 0);
    assert_eq!(
        invoice_lines_floor_tax_rounding_remainder_total(&[line]).unwrap(),
        0
    );

    let plan = SubscriptionPlan::try_new(100, days(30)).unwrap();
    assert!(SubscriptionPlan::try_new(100, Duration::ZERO).is_err());
    assert!(
        RecurringSubscription::try_new(
            CustomerId::new(1),
            plan,
            SubscriptionLifecycleStatus::Active,
            epoch(),
            later(30)
        )
        .is_ok()
    );
    let card = GiftCard::new(100, later(10));
    let redemption = GiftCardRedemption::try_new(card.clone(), 25).unwrap();
    assert_eq!(gift_card_balance_after_redeem(&redemption), 75);
    assert!(gift_card_valid_at(epoch(), &card));
    assert!(Chargeback::try_new(100, 25).is_ok());
    let cash_events = vec![CashflowEvent::new(100, 10), CashflowEvent::new(20, 5)];
    assert_eq!(cashflow_inflows_total(&cash_events).unwrap(), 120);
    assert_eq!(cashflow_outflows_total(&cash_events).unwrap(), 15);
    assert!(CashflowPlan::try_new(100, 20, 50, 30).is_ok());
    assert!(EventBackedCashflowPlan::try_new(100, 20, cash_events).is_ok());

    let fraud = FraudPolicy::new(3, 10, 1);
    assert!(coupon_uses_allowed(&fraud, 3));
    assert!(orders_per_hour_allowed(&fraud, 10));
    assert!(can_perform(Role::Admin, Action::DeleteOrder));
    let audit = AuditEvent::new(Role::Finance, Action::IssueRefund, OrderId::new(1));
    assert!(
        AuditedCommand::try_new(Role::Finance, Action::IssueRefund, OrderId::new(1), audit).is_ok()
    );
    let entity_event = EntityAuditEvent::new(Role::Support, Action::ViewOrder, 10);
    assert!(
        AuditedEntityCommand::try_new(Role::Support, Action::ViewOrder, 10, entity_event.clone())
            .is_ok()
    );
    assert!(data_processing_allowed(&data_permission()));
    assert!(role_can_access_data(
        Role::Support,
        AccessPurpose::CustomerSupport,
        DataCategory::SupportNotes
    ));
    assert!(processing_allowed_for(
        &data_permission(),
        ConsentPurpose::Marketing,
        ProcessingBasis::Consent
    ));
    let consent_state = MarketingConsentState::new(
        SubscriptionStatus::Subscribed,
        ConsentStatus::Granted,
        data_permission(),
    );
    assert!(marketing_allowed(&consent_state));
    assert!(!marketing_allowed(&withdraw_marketing_consent(
        &consent_state
    )));
    let retention = DataRetentionPolicy::new(DataCategory::OrderData, days(30));
    assert!(within_retention_window(&retention, later(10), epoch()));
    assert!(!retention_expired(&retention, later(10), epoch()));
    assert!(
        RetainedPersonalData::try_new(
            CustomerId::new(1),
            DataCategory::OrderData,
            epoch(),
            later(10),
            retention
        )
        .is_ok()
    );
    assert!(personal_data_usable(ErasureStatus::Active));
    assert!(can_process_personal_data(
        ErasureStatus::Active,
        &data_permission(),
        ConsentPurpose::Marketing,
        ProcessingBasis::Consent
    ));
    assert!(can_complete_erasure(ErasureStatus::Requested, false));
    assert!(audit_log_appended(
        std::slice::from_ref(&entity_event),
        &[entity_event.clone(), entity_event.clone()],
        std::slice::from_ref(&entity_event)
    ));
    assert!(
        AuditedDataAccess::try_new(
            Role::Support,
            Action::ViewOrder,
            AccessPurpose::CustomerSupport,
            DataCategory::OrderData,
            10,
            entity_event
        )
        .is_ok()
    );

    let account = crm_account();
    assert!(can_crm_account_transition(
        CRMAccountStatus::Prospect,
        CRMAccountStatus::Active
    ));
    assert!(crm_account_active(&account));
    assert!(ActiveCRMAccount::try_new(account.clone()).is_ok());
    assert!(transition_crm_account(account.clone(), CRMAccountStatus::Paused).is_ok());
    let contact = crm_contact();
    assert!(contact_can_receive_marketing(&contact));
    let account_contact = CRMAccountContact::try_new(account.clone(), contact.clone()).unwrap();
    let message =
        PermittedCustomerMessage::try_new(InteractionId::new(1), contact.clone(), epoch()).unwrap();
    assert!(PermittedAccountMessage::try_new(account_contact.clone(), message).is_ok());
    let interaction = CRMInteraction::try_new(
        InteractionId::new(2),
        AccountId::new(1),
        ContactId::new(2),
        InteractionKind::Email,
        epoch(),
        later(1),
    )
    .unwrap();
    assert!(CRMInteractionForContact::try_new(account_contact.clone(), interaction).is_ok());
    let lead = Lead::try_new(
        LeadId::new(1),
        AccountId::new(1),
        ContactId::new(2),
        Some(CampaignId::new(1)),
        LeadStatus::New,
        500,
        Currency::USD,
        epoch(),
        epoch(),
    )
    .unwrap();
    assert!(can_lead_transition(LeadStatus::New, LeadStatus::Working));
    let converted_lead = transition_lead(
        transition_lead(lead, LeadStatus::Working, later(1)).unwrap(),
        LeadStatus::Qualified,
        later(2),
    )
    .unwrap();
    let converted_lead = transition_lead(converted_lead, LeadStatus::Converted, later(3)).unwrap();
    assert!(LeadForContact::try_new(account_contact.clone(), converted_lead).is_ok());
    let won = BasisPoints::try_new(10_000).unwrap();
    assert!(opportunity_stage_probability_allowed(
        OpportunityStage::Won,
        won
    ));
    let opportunity = SalesOpportunity::try_new(
        OpportunityId::new(1),
        AccountId::new(1),
        ContactId::new(2),
        Some(LeadId::new(1)),
        OpportunityStage::Prospecting,
        400,
        Currency::USD,
        BasisPoints::try_new(5_000).unwrap(),
        epoch(),
        epoch(),
        later(30),
    )
    .unwrap();
    assert_eq!(opportunity_weighted_value(&opportunity).unwrap(), 200);
    let opportunity = transition_opportunity(
        opportunity,
        OpportunityStage::Qualified,
        BasisPoints::try_new(5_000).unwrap(),
        later(1),
        later(30),
    )
    .unwrap();
    assert!(OpportunityForContact::try_new(account_contact.clone(), opportunity).is_ok());
    assert_eq!(opportunity_gross_value(&[opportunity]).unwrap(), 400);
    assert_eq!(
        opportunity_weighted_value_total(&[opportunity]).unwrap(),
        200
    );
    assert!(SalesPipeline::try_new(Currency::USD, vec![opportunity]).is_ok());
    let segment =
        CustomerSegment::try_new(SegmentId::new(1), "vip".to_owned(), 1, 100, 50).unwrap();
    assert!(SegmentMembership::try_new(account.clone(), segment.clone()).is_ok());
    let case = support_case(SupportCaseStatus::Opened, Some(OrderId::new(10)));
    let escalated = transition_support_case(case, SupportCaseStatus::Escalated, later(1)).unwrap();
    assert!(SupportCaseForContact::try_new(account_contact, escalated).is_ok());
    let resolved =
        transition_support_case(escalated, SupportCaseStatus::Resolved, later(2)).unwrap();
    assert!(ResolvedSupportCase::try_new(resolved, later(3)).is_ok());
    assert!(RetentionOffer::try_new(account, segment, Coupon::new(50, 100, 10), 1, 25).is_ok());

    let plan = logistics_plan();
    assert!(can_shipment_transition(
        ShipmentStatus::Planned,
        ShipmentStatus::Allocated
    ));
    assert!(order_eligible_for_logistics(plan.order()));
    assert!(cart_contains_sku(sku(10), plan.order().items()));
    assert!(allocations_match_cart_skus(
        plan.order().items(),
        plan.fulfillment().allocations()
    ));
    assert_eq!(
        cart_sku_quantity_total(sku(10), plan.order().items()).unwrap(),
        2
    );
    assert_eq!(
        allocation_sku_quantity_total(sku(10), plan.fulfillment().allocations()).unwrap(),
        2
    );
    assert!(
        allocation_quantities_match_cart_skus(
            plan.order().items(),
            plan.fulfillment().allocations()
        )
        .unwrap()
    );
    assert!(allocations_use_warehouse(
        plan.warehouse(),
        plan.fulfillment().allocations()
    ));
    let shipment = LogisticsShipment::try_new(
        ShipmentId::new(1),
        plan.clone(),
        ShipmentStatus::Planned,
        epoch(),
        epoch(),
    )
    .unwrap();
    assert!(transition_shipment(shipment, ShipmentStatus::Allocated, later(1)).is_ok());
    assert!(
        CarrierHandoff::try_new(
            plan.clone(),
            plan.quote().service().clone(),
            9_999,
            epoch(),
            later(1)
        )
        .is_ok()
    );
    let events = vec![
        TrackingEvent::new(
            TrackingEventId::new(1),
            ShipmentId::new(1),
            77,
            9_999,
            TrackingEventKind::LabelCreated,
            epoch(),
        ),
        TrackingEvent::new(
            TrackingEventId::new(2),
            ShipmentId::new(1),
            77,
            9_999,
            TrackingEventKind::PickupScan,
            later(1),
        ),
        TrackingEvent::new(
            TrackingEventId::new(3),
            ShipmentId::new(1),
            77,
            9_999,
            TrackingEventKind::InTransitScan,
            later(2),
        ),
        TrackingEvent::new(
            TrackingEventId::new(4),
            ShipmentId::new(1),
            77,
            9_999,
            TrackingEventKind::OutForDeliveryScan,
            later(3),
        ),
        TrackingEvent::new(
            TrackingEventId::new(5),
            ShipmentId::new(1),
            77,
            9_999,
            TrackingEventKind::DeliveredScan,
            later(4),
        ),
    ];
    assert!(tracking_events_monotone_from(epoch(), &events));
    assert!(tracking_events_for_shipment(ShipmentId::new(1), &events));
    assert!(tracking_events_for_carrier(77, 9_999, &events));
    assert_eq!(tracking_last_observed_from(epoch(), &events), later(4));
    assert!(tracking_event_ids_distinct(&events));
    assert!(tracking_events_progress_from(
        TrackingEventKind::LabelCreated,
        &events
    ));
    let history =
        TrackingHistory::try_new(ShipmentId::new(1), 77, 9_999, events.clone(), later(4)).unwrap();
    let promise = DeliveryPromise::try_new(plan.clone(), later(3)).unwrap();
    assert!(!delivered_by_promise(&promise, later(4)));
    assert!(DeliveredShipment::try_new(promise, history, events[4].clone(), later(4)).is_err());
    let transfer = WarehouseTransfer::try_new(
        TransferId::new(1),
        sku(10),
        warehouse(1),
        warehouse(2),
        StockState::try_new(sku(10), 10, 0).unwrap(),
        5,
        3,
        2,
    )
    .unwrap();
    assert_eq!(transfer.received(), 2);
    let return_line = ReturnLine::new(sku(10), 1, 100);
    let return_order = order_with(10, OrderStatus::Paid, sku(10), 500, 2);
    let authorization = ReturnAuthorization::try_new(
        ReturnAuthorizationId::new(1),
        support_case(SupportCaseStatus::Opened, Some(return_order.id())),
        return_order.clone(),
        PaymentLedger::try_new(return_order.total(), 0).unwrap(),
        ReturnAuthorizationStatus::Approved,
        vec![return_line],
        1,
        100,
        epoch(),
        later(1),
    )
    .unwrap();
    assert!(return_authorization_approved(&authorization));
    let authorization = transition_return_authorization(
        authorization,
        ReturnAuthorizationStatus::Received,
        later(2),
    )
    .unwrap();
    assert!(ReturnReceipt::try_new(authorization, 1, 50, later(3)).is_err());
}

#[test]
fn event_replay_validation_and_implicit_invariant_paths_are_covered() {
    let envelopes = vec![
        EventEnvelope::new(1, DomainEvent::OrderPlaced(OrderId::new(1), 100)),
        EventEnvelope::new(2, DomainEvent::PaymentCaptured(OrderId::new(1), 100)),
        EventEnvelope::new(3, DomainEvent::OrderShipped(OrderId::new(1))),
    ];
    assert!(stream_sequences_strictly_increase_from(0, &envelopes));
    assert!(stream_sequences_strictly_increase(&EventStream::new(
        envelopes.clone(),
        3
    )));
    assert!(apply_webhook(&WebhookOrderingState::new(0), 1).is_ok());
    assert!(apply_webhook(&WebhookOrderingState::new(1), 1).is_err());
    assert_eq!(
        replay_webhook_stream(WebhookOrderingState::new(0), &envelopes)
            .unwrap()
            .last_sequence(),
        3
    );
    assert!(already_processed(
        IdempotencyKey::new(1),
        &IdempotencyState::new(vec![IdempotencyKey::new(1)])
    ));
    assert!(
        mark_processed(IdempotencyKey::new(2), &IdempotencyState::new(vec![]))
            .processed()
            .contains(&IdempotencyKey::new(2))
    );

    let state = ValidSystemState::new(
        StockState::try_new(sku(10), 10, 4).unwrap(),
        PaymentLedger::try_new(100, 10).unwrap(),
        5,
        0,
        0,
    );
    assert!(apply_stock_reserved_event(&state, sku(10), 1).is_ok());
    assert!(apply_stock_reserved_event(&state, sku(99), 1).is_err());
    assert!(apply_refund_issued_event(&state, 10).is_ok());
    assert!(apply_reservation_released_event(&state, sku(10), 1).is_ok());
    assert!(apply_reserved_shipment_confirmed_event(&state, sku(10), 1).is_ok());
    assert!(apply_tax_liability_recorded_event(&state, 3).is_ok());
    assert!(apply_crm_projected_event(&state).is_ok());
    assert!(apply_logistics_projected_event(&state).is_ok());
    assert_eq!(
        record_captured_payment(state.ledger(), 5)
            .unwrap()
            .captured(),
        105
    );
    let domain_events = vec![
        DomainEvent::PaymentCaptured(OrderId::new(1), 5),
        DomainEvent::RefundIssued(OrderId::new(1), 5),
        DomainEvent::ReservationReleased(sku(10), 1),
        DomainEvent::ReservedShipmentConfirmed(sku(10), 1),
        DomainEvent::TaxLiabilityRecorded(1, 2),
        DomainEvent::SupportCaseOpened(SupportCaseId::new(1), Some(OrderId::new(1))),
        DomainEvent::ShipmentPlanned(ShipmentId::new(1), OrderId::new(1)),
    ];
    assert!(replay_domain_events(state.clone(), &domain_events).is_ok());
    let idempotency = IdempotencyState::new(vec![]);
    let (next, idempotency) = apply_idempotent_domain_event(
        IdempotencyKey::new(1),
        &DomainEvent::PaymentCaptured(OrderId::new(1), 1),
        state.clone(),
        idempotency,
    )
    .unwrap();
    assert_eq!(
        apply_idempotent_domain_event(
            IdempotencyKey::new(1),
            &DomainEvent::PaymentCaptured(OrderId::new(1), 1),
            next.clone(),
            idempotency
        )
        .unwrap()
        .0,
        next
    );
    assert!(replay_from_snapshot(&EventSnapshot::new(state.clone(), 0), &domain_events).is_ok());
    assert_eq!(ledger_captured_fold(0, &domain_events).unwrap(), 5);
    assert_eq!(ledger_refunded_fold(0, &domain_events).unwrap(), 5);
    assert_eq!(project_tax_liability(0, &domain_events).unwrap(), 2);
    assert!(project_ledger(PaymentLedger::try_new(100, 0).unwrap(), &domain_events[..2]).is_ok());

    let symbols = domain_event_symbols(&domain_events);
    assert!(symbols.contains(&OrderEventSymbol::PaymentCaptured));
    assert_eq!(
        order_event_validation_step(
            OrderEventValidationState::Start,
            OrderEventSymbol::OrderPlaced
        ),
        OrderEventValidationState::Placed
    );
    assert!(order_event_validator().accepts(&[
        OrderEventSymbol::OrderPlaced,
        OrderEventSymbol::PaymentCaptured,
        OrderEventSymbol::OrderShipped,
    ]));
    assert_eq!(
        order_event_validator().start(),
        OrderEventValidationState::Start
    );
    assert_eq!(
        order_event_validator().run(&symbols),
        validate_order_event_symbols(&symbols)
    );
    assert_eq!(
        order_transition_target(OrderStatus::New, OrderTransitionLabel::CapturePayment),
        Some(OrderStatus::Paid)
    );
    assert!(
        execute_order_trace(OrderStatus::New, &unpaid_cancellation_trace())
            .unwrap()
            .contains(&OrderStatus::Cancelled)
    );
    assert!(terminal_order_status(OrderStatus::Delivered));
    assert_eq!(
        order_status_lts().transition(OrderStatus::Paid, OrderTransitionLabel::PackPaidOrder),
        Some(OrderStatus::Packed)
    );
    assert_eq!(
        dropship_po_transition_target(DropshipPOStatus::Created, DropshipPOTransitionLabel::Submit),
        Some(DropshipPOStatus::Submitted)
    );
    assert!(
        execute_dropship_po_trace(DropshipPOStatus::Created, &dropship_po_delivery_trace())
            .unwrap()
            .contains(&DropshipPOStatus::Delivered)
    );
    assert!(terminal_dropship_po_status(DropshipPOStatus::Rejected));
    assert_eq!(
        dropship_po_lts().transition(
            DropshipPOStatus::Submitted,
            DropshipPOTransitionLabel::Reject
        ),
        Some(DropshipPOStatus::Rejected)
    );

    let step = WebhookOrderingStep::accept(WebhookOrderingState::new(0), 1).unwrap();
    assert_eq!(step.before().last_sequence(), 0);
    assert_eq!(step.after().last_sequence(), 1);
    assert!(webhook_replay_in_steps(WebhookOrderingState::new(0), &envelopes).is_ok());
    assert!(
        webhook_replay_within_steps(WebhookOrderingState::new(0), &envelopes, 2)
            .unwrap()
            .is_none()
    );
    for event in [
        ValidSystemEvent::StockReserved(sku(10), 1),
        ValidSystemEvent::RefundIssued(1),
        ValidSystemEvent::ReservationReleased(sku(10), 1),
        ValidSystemEvent::ReservedShipmentConfirmed(sku(10), 1),
        ValidSystemEvent::TaxLiabilityRecorded(1),
        ValidSystemEvent::CrmProjected,
        ValidSystemEvent::LogisticsProjected,
    ] {
        assert!(valid_system_replay_in_steps(state.clone(), &[event]).is_ok());
    }
    assert!(ValidSystemEventStep::stock_reserved(state.clone(), sku(10), 1).is_ok());
    assert!(ValidSystemEventStep::refund_issued(state.clone(), 1).is_ok());
    assert!(ValidSystemEventStep::reservation_released(state.clone(), sku(10), 1).is_ok());
    assert!(ValidSystemEventStep::reserved_shipment_confirmed(state.clone(), sku(10), 1).is_ok());
    assert!(ValidSystemEventStep::tax_liability_recorded(state.clone(), 1).is_ok());
    assert!(ValidSystemEventStep::crm_projected(state.clone()).is_ok());
    assert!(ValidSystemEventStep::logistics_projected(state.clone()).is_ok());
    for event in [
        DomainEvent::StockReserved(sku(10), 1),
        DomainEvent::RefundIssued(OrderId::new(1), 1),
        DomainEvent::ReservationReleased(sku(10), 1),
        DomainEvent::ReservedShipmentConfirmed(sku(10), 1),
        DomainEvent::TaxLiabilityRecorded(1, 1),
        DomainEvent::SupportCaseOpened(SupportCaseId::new(1), Some(OrderId::new(1))),
        DomainEvent::ShipmentPlanned(ShipmentId::new(1), OrderId::new(1)),
    ] {
        let step = ValidDomainEventStep::from_event(state.clone(), event).unwrap();
        assert_eq!(step.before(), &state);
        assert_ne!(step.event(), &DomainEvent::OrderShipped(OrderId::new(99)));
    }

    assert!(
        validate_cart_line(RawCartLine {
            sku: sku(10),
            price: 100,
            cost: 40,
            quantity: 1,
            discount: 0,
            weight: 1
        })
        .is_ok()
    );
    assert!(
        validate_stock_state(RawStockState {
            sku: sku(10),
            total: 10,
            reserved: 11
        })
        .is_err()
    );
    assert!(
        validate_channel_price_policy(RawChannelPricePolicy {
            min_price: 10,
            max_price: 5
        })
        .is_err()
    );
    assert!(
        validate_payment_ledger(RawPaymentLedger {
            captured: 10,
            refunded: 11
        })
        .is_err()
    );
    assert!(validate_basis_points(10_001).is_err());
    assert!(validate_pack_task(1, 2).is_err());
    assert!(
        validate_raw_compare_and_swap_reservation(RawReservationAttempt {
            stock: RawStockState {
                sku: sku(10),
                total: 10,
                reserved: 0
            },
            version: 1,
            quantity: 1,
            expected_version: 1
        })
        .is_ok()
    );
    assert!(
        validate_release_reserved_stock(StockState::try_new(sku(10), 10, 2).unwrap(), 1).is_ok()
    );
    assert!(
        validate_confirm_reserved_shipment(StockState::try_new(sku(10), 10, 2).unwrap(), 1).is_ok()
    );
    assert!(
        validate_timed_reservation(
            StockState::try_new(sku(10), 10, 2).unwrap(),
            1,
            epoch(),
            later(1),
            ReservationStatus::Active
        )
        .is_ok()
    );

    let raw_line = RawTaxInvoiceLine {
        sku: sku(10),
        quantity: 2,
        unit_price: 100,
        discount: 20,
        treatment: TaxTreatment::Taxable,
        rate: tax_rate(),
        rounding_mode: RoundingMode::Floor,
        taxable_amount: 180,
        tax: 18,
        total: 198,
    };
    assert!(validate_tax_invoice_line(raw_line.clone()).is_ok());
    assert!(
        validate_tax_invoice(RawTaxInvoice {
            id: 1,
            issued_at: epoch(),
            seller_id: 1,
            buyer_id: CustomerId::new(1),
            jurisdiction: tax_jurisdiction(),
            currency: Currency::USD,
            lines: vec![raw_line],
            subtotal: 180,
            tax: 18,
            shipping: 2,
            discount: 0,
            total: 200,
        })
        .is_ok()
    );

    let product = Product::new(
        ProductId::new(1),
        Brand::new(1, "b".to_owned()),
        Category::new(1, "c".to_owned()),
        ProductStatus::Active,
    );
    let catalog = ProductCatalogEntry::try_new(
        product,
        ProductVariant::new(VariantId::new(1), ProductId::new(1), sku(10), true),
    )
    .unwrap();
    assert!(validate_sellable_catalog_entry(catalog).is_ok());
    let feed = SafeProductFeedLine::try_new(
        sku(10),
        SalesChannel::OwnWebsite,
        100,
        Currency::USD,
        1,
        StockState::try_new(sku(10), 10, 0).unwrap(),
        ChannelPricePolicy::try_new(50, 150).unwrap(),
    )
    .unwrap();
    assert!(validate_publishable_feed_line(feed).is_ok());
    let bounded = validate_bounded_coupon_application(Coupon::new(10, 50, 5), 100, 1).unwrap();
    assert!(validate_fraud_checked_coupon_application(bounded, FraudPolicy::new(5, 10, 0)).is_ok());
    let payment = CapturedPayment::new(OrderId::new(1), 1_000, Currency::USD);
    assert!(validate_captured_payment_journal_projection(accounts(), payment).is_ok());
    assert!(
        validate_refund_journal_projection(
            accounts(),
            PaymentLedger::try_new(1_000, 0).unwrap(),
            100
        )
        .is_ok()
    );

    let forecast = DemandForecast::new(sku(10), 10, Confidence::High, days(7));
    assert!(demand_forecast_actionable(&forecast));
    assert!(validate_actionable_demand_forecast(forecast).is_ok());
    let metrics = SupplierQualityMetrics::new(1, 2, 3);
    let quality =
        validate_approved_supplier_quality(supplier(), metrics, SupplierRiskPolicy::new(2, 3, 4))
            .unwrap();
    assert!(validate_approved_orderable_supplier_quality(quality).is_ok());
    let candidate =
        validate_dropship_opportunity_candidate(sku(10), 2, 100, 50, 30, 20, 110, dropship_costs())
            .unwrap();
    assert_eq!(
        candidates_capital_total(std::slice::from_ref(&candidate)).unwrap(),
        50
    );
    assert_eq!(
        candidates_profit_total(std::slice::from_ref(&candidate)).unwrap(),
        30
    );
    assert_eq!(
        candidates_min_profit_total(std::slice::from_ref(&candidate)).unwrap(),
        20
    );
    assert!(validate_dropship_opportunity_portfolio(vec![candidate.clone()], 100).is_ok());
    assert_eq!(opportunity_rank_key(&candidate), 30);
    assert_eq!(
        opportunity_rank_keys(std::slice::from_ref(&candidate)),
        vec![30]
    );
    assert_eq!(rank_opportunity_keys(&[candidate]).ret, vec![30]);
    assert_eq!(
        rank_opportunity_keys(&[
            opportunity_candidate(40),
            opportunity_candidate(25),
            opportunity_candidate(35),
            opportunity_candidate(20),
        ])
        .ret,
        vec![20, 25, 35, 40]
    );
}

#[test]
fn validator_facade_and_implicit_wrappers_are_covered() {
    let raw_line = RawCartLine {
        sku: sku(10),
        price: 500,
        cost: 200,
        quantity: 2,
        discount: 0,
        weight: 1,
    };
    let cart_line = validate_cart_line(raw_line).unwrap();
    assert!(cart_line_matches_raw(&raw_line, &cart_line));
    assert!(cart_lines_match_raw(
        &[raw_line],
        &validate_cart_lines(vec![raw_line]).unwrap()
    ));
    let raw_order = RawOrder {
        id: OrderId::new(1),
        items: vec![raw_line],
        coupon_amount: 0,
        shipping_method: free_shipping(),
        tax: 0,
        currency: Currency::USD,
        status: OrderStatus::Paid,
        total: 1_000,
    };
    let order = validate_order(raw_order.clone()).unwrap();
    assert!(order_matches_raw(&raw_order, &order));

    let product = Product::new(
        ProductId::new(1),
        Brand::new(1, "brand".to_owned()),
        Category::new(1, "category".to_owned()),
        ProductStatus::Active,
    );
    let variant = ProductVariant::new(VariantId::new(1), ProductId::new(1), sku(10), true);
    let catalog_entry = validate_product_catalog_entry(product, variant).unwrap();
    let listing_content = validate_listing_content(
        ListingContent::new(20, 2, true),
        MarketplaceContentPolicy::new(80, 2),
    )
    .unwrap();
    assert_eq!(listing_content.content().title_length(), 20);

    let stock = validate_stock_state(RawStockState {
        sku: sku(10),
        total: 10,
        reserved: 2,
    })
    .unwrap();
    let versioned = validate_versioned_stock(
        RawStockState {
            sku: sku(10),
            total: 10,
            reserved: 2,
        },
        1,
    )
    .unwrap();
    let wh = warehouse(1);
    let bin = BinStock::new(sku(10), BinLocation::new(wh.clone(), 1), 3);
    assert!(validate_pick_task(sku(10), 2, bin).is_ok());
    assert!(validate_warehouse_shipment(2, 1).is_ok());
    let allocation = validate_allocation(
        InventoryNode::new(wh.clone(), StockState::try_new(sku(10), 10, 0).unwrap()),
        2,
    )
    .unwrap();
    assert!(validate_fulfillment_plan(2, vec![allocation.clone()]).is_ok());
    let distinct = validate_distinct_fulfillment_plan(2, vec![allocation.clone()]).unwrap();
    assert_eq!(distinct.requested(), 2);
    let reservation =
        validate_timed_reservation(stock, 1, epoch(), later(1), ReservationStatus::Active).unwrap();
    assert!(validate_release_expired_reservation(reservation, later(2)).is_ok());
    assert!(validate_backorder_request(sku(10), 5, 2, 3).is_ok());
    let window = validate_preorder_window(sku(10), epoch(), later(2), 5).unwrap();
    assert!(validate_preorder_reservation(window, 2, later(1)).is_ok());
    assert!(
        validate_serialized_inventory_set(vec![
            SerializedInventoryUnit::new(sku(10), SerialNumber::new(1), wh.clone(), false),
            SerializedInventoryUnit::new(sku(10), SerialNumber::new(2), wh.clone(), false),
        ])
        .is_ok()
    );
    assert!(
        validate_usable_inventory_lot(
            InventoryLot::new(sku(10), 1, wh.clone(), later(2), 3),
            epoch()
        )
        .is_ok()
    );
    assert!(validate_sku_substitution(sku(9), sku(10), stock, 2).is_ok());
    let split = validate_split_fulfillment_plan(
        DistinctFulfillmentPlan::try_new(
            4,
            vec![
                allocation_for(sku(10), 2, &wh),
                allocation_for(sku(11), 2, &warehouse(2)),
            ],
        )
        .unwrap(),
        wh.clone(),
        warehouse(2),
    );
    assert!(split.is_ok());
    assert!(validate_typed_order::<NewOrder>(OrderId::new(1), 1_000, Currency::USD).is_ok());
    assert!(
        validate_typed_payment::<CreatedPayment>(
            PaymentId::new(1),
            OrderId::new(1),
            1_000,
            Currency::USD
        )
        .is_ok()
    );
    assert!(
        validate_balanced_journal_entry(vec![
            debit(ledger_account(1, "d"), 1),
            credit(ledger_account(2, "c"), 1),
        ])
        .is_ok()
    );

    let policy = validate_channel_price_policy(RawChannelPricePolicy {
        min_price: 100,
        max_price: 2_000,
    })
    .unwrap();
    let feed = validate_feed_line(RawProductFeedLine {
        sku: sku(10),
        channel: SalesChannel::MarketplaceChannel(Marketplace::AmazonLike),
        price: 1_000,
        currency: Currency::USD,
        stock: 3,
        stock_state: RawStockState {
            sku: sku(10),
            total: 10,
            reserved: 0,
        },
        price_policy: RawChannelPricePolicy {
            min_price: 100,
            max_price: 2_000,
        },
    })
    .unwrap();
    let listing = MarketplaceListing::new(
        sku(10),
        Marketplace::AmazonLike,
        100,
        1_000,
        Currency::USD,
        3,
        ListingStatus::Active,
    );
    let synced =
        validate_synced_marketplace_listing(listing, StockState::try_new(sku(10), 10, 0).unwrap())
            .unwrap();
    let fee_ledger = validate_marketplace_fee_ledger(
        1_000,
        BasisPoints::try_new(1_000).unwrap(),
        RoundingMode::Floor,
        100,
        900,
    )
    .unwrap();
    assert!(
        validate_marketplace_payout_calculation(
            1_000,
            BasisPoints::try_new(1_000).unwrap(),
            RoundingMode::Floor,
            100
        )
        .is_ok()
    );
    assert!(
        validate_marketplace_order(
            Marketplace::AmazonLike,
            MarketplaceOrderId::new(1),
            order.clone(),
            order.total(),
            MarketplaceFeeLedger::try_new(
                order.total(),
                BasisPoints::try_new(1_000).unwrap(),
                RoundingMode::Floor,
                100,
                900
            )
            .unwrap(),
        )
        .is_ok()
    );
    assert_eq!(fee_ledger.payout(), 900);

    let campaign = validate_marketing_campaign(
        CampaignId::new(1),
        AdPlatform::GoogleLike,
        AdType::Shopping,
        AdDestination::MarketplaceStore(Marketplace::AmazonLike),
        CampaignStatus::Active,
        1_000,
        100,
        1_000,
        100,
        10,
        500,
    )
    .unwrap();
    assert!(validate_click_attributed_campaign(campaign.clone()).is_ok());
    assert!(validate_funnel(1_000, 500, 100, 25).is_ok());
    assert!(
        validate_order_attribution_ledger(
            order.clone(),
            vec![AttributionCredit::new(CampaignId::new(1), order.id(), 100)]
        )
        .is_ok()
    );
    let variant_a = validate_experiment_variant(1, 50, 100, 10).unwrap();
    let variant_b = validate_experiment_variant(2, 50, 100, 20).unwrap();
    assert!(validate_experiment(1, vec![variant_a, variant_b]).is_ok());

    let book =
        validate_trade_price_book_entry(sku(10), Currency::USD, 40, 100, 80, 20, 10, 2).unwrap();
    let retail = validate_retail_line(book.clone(), 2, 10).unwrap();
    let wholesale = validate_wholesale_line(book, 2, 5).unwrap();
    let wholesale_account = validate_wholesale_credit_account(customer(), 1_000, 100).unwrap();
    assert_eq!(retail.quantity(), 2);
    assert_eq!(wholesale.quantity(), 2);
    assert!(
        validate_wholesale_credit_checkout(
            wholesale_account.clone(),
            vec![wholesale.clone()],
            PaymentTerms::NetDays(30),
            wholesale_order_net_total(std::slice::from_ref(&wholesale)).unwrap()
        )
        .is_ok()
    );

    let capacity = validate_supplier_daily_capacity(supplier(), 10, 2).unwrap();
    assert_eq!(capacity.orders_accepted_today(), 2);
    let offer =
        validate_dropship_offer(sku(10), supplier(), 40, 100, 2, 10, Currency::USD, true).unwrap();
    let supplier_reservation = validate_supplier_reservation(
        offer.clone(),
        supplier(),
        2,
        SupplierReservationStatus::Confirmed,
    )
    .unwrap();
    let dropship_line = validate_dropship_line(offer.clone(), 2, 0).unwrap();
    assert!(validate_reserved_dropship_line(dropship_line.clone(), supplier_reservation).is_ok());
    let po = validate_dropship_purchase_order(
        supplier(),
        vec![dropship_line.clone()],
        DropshipShippingQuote::new(SupplierId::new(1), 10, 10, days(2)),
        DropshipPOStatus::Created,
        90,
    )
    .unwrap();
    assert!(
        validate_dropship_fulfillment(order_with(2, OrderStatus::Paid, sku(10), 100, 2), po, 200)
            .is_ok()
    );
    assert!(validate_dropship_return_request(dropship_line, 1, 50, 20).is_ok());
    assert!(validate_guaranteed_dropship_profit_quote(100, dropship_costs(), 20, 46, 46).is_ok());
    assert!(
        validate_dropship_cost_upper_bounds(
            dropship_costs(),
            DropshipProfitCosts::new(50, 5, 3, 2, 1, 1, 1, 1)
        )
        .is_ok()
    );
    let best = competitor_offer(100);
    let benchmark =
        validate_singleton_competitor_price_benchmark(sku(10), Currency::USD, best).unwrap();
    let aware =
        validate_competitor_aware_dropship_offer(offer, benchmark.clone(), 0, dropship_costs(), 20)
            .unwrap();
    assert!(validate_brand_pricing_policy(80, 120).is_ok());
    let brand_policy = validate_brand_pricing_policy(80, 120).unwrap();
    assert!(validate_map_compliant_competitor_aware_offer(aware, brand_policy).is_ok());
    let component = validate_bundle_component(sku(10), 2, 10).unwrap();
    assert!(bundle_components_can_fulfill_all(3, std::slice::from_ref(&component)).unwrap());
    assert!(validate_bundle_reservation(3, vec![component]).is_ok());
    assert!(validate_accepted_promotion_set(90, 10, 20, 50).is_ok());
    assert!(validate_search_result_item(SearchResultItem::new(sku(10), false, true, true)).is_ok());
    assert!(
        validate_trusted_fresh_competitor_benchmark(benchmark, later(1), days(2), TrustLevel::High)
            .is_ok()
    );

    assert!(validate_exchange_rate(Currency::USD, Currency::EUR, 9, 10, epoch()).is_ok());
    assert!(validate_tax_calculation(100, tax_rate(), RoundingMode::Floor, 10, 110).is_ok());
    assert!(validate_tax_inclusive_price(110, 100, 10).is_ok());
    assert!(validate_tax_exclusive_price(100, 10, 110).is_ok());
    let tax_line = RawTaxInvoiceLine {
        sku: sku(10),
        quantity: 2,
        unit_price: 100,
        discount: 20,
        treatment: TaxTreatment::Taxable,
        rate: tax_rate(),
        rounding_mode: RoundingMode::Floor,
        taxable_amount: 180,
        tax: 18,
        total: 198,
    };
    let invoice = validate_tax_invoice(RawTaxInvoice {
        id: 1,
        issued_at: epoch(),
        seller_id: 1,
        buyer_id: CustomerId::new(1),
        jurisdiction: tax_jurisdiction(),
        currency: Currency::USD,
        lines: vec![tax_line],
        subtotal: 180,
        tax: 18,
        shipping: 2,
        discount: 0,
        total: 200,
    })
    .unwrap();
    assert!(
        validate_order_tax_invoice_link(order_with(3, OrderStatus::Paid, sku(10), 100, 2), invoice)
            .is_err()
    );
    let cert =
        validate_tax_exemption_certificate(CustomerId::new(1), 1, epoch(), later(3)).unwrap();
    assert!(validate_b2b_tax_exemption(customer(), tax_jurisdiction(), cert, later(1)).is_ok());
    assert!(
        validate_marketplace_facilitator_tax(
            Marketplace::AmazonLike,
            tax_jurisdiction(),
            180,
            tax_rate(),
            RoundingMode::Floor,
            18,
            true,
            0
        )
        .is_ok()
    );
    assert!(
        validate_carrier_quote(
            carrier_service(ShippingZone::new(9, "zone".to_owned())),
            Package::new(5, 1),
            7
        )
        .is_ok()
    );
    assert!(validate_reconciliation_within_tolerance(100, 101, 1).is_ok());

    let subscription_plan = validate_subscription_plan(100, days(30)).unwrap();
    assert!(
        validate_recurring_subscription(
            CustomerId::new(1),
            subscription_plan,
            SubscriptionLifecycleStatus::Active,
            epoch(),
            later(30)
        )
        .is_ok()
    );
    let redemption = validate_gift_card_redemption(GiftCard::new(100, later(3)), 20).unwrap();
    assert!(validate_valid_gift_card_redemption_at(epoch(), redemption).is_ok());
    let chargeback = validate_chargeback(100, 20).unwrap();
    assert!(
        validate_chargeback_for_captured_payment(
            CapturedPayment::new(OrderId::new(1), 100, Currency::USD),
            chargeback
        )
        .is_ok()
    );
    assert!(validate_cashflow_plan(100, 20, 30, 10).is_ok());
    assert!(validate_event_backed_cashflow_plan(100, 20, vec![CashflowEvent::new(30, 10)]).is_ok());

    assert!(
        validate_audited_command(
            Role::Finance,
            Action::IssueRefund,
            OrderId::new(1),
            AuditEvent::new(Role::Finance, Action::IssueRefund, OrderId::new(1)),
        )
        .is_ok()
    );
    assert!(
        validate_audited_entity_command(
            Role::Support,
            Action::ViewOrder,
            42,
            EntityAuditEvent::new(Role::Support, Action::ViewOrder, 42),
        )
        .is_ok()
    );
    assert!(
        validate_event_stream(EventStream::new(
            vec![EventEnvelope::new(
                1,
                DomainEvent::OrderPlaced(OrderId::new(1), 100)
            )],
            1
        ))
        .is_ok()
    );

    let account = validate_crm_account(
        AccountId::new(1),
        customer(),
        AccountTier::Strategic,
        CRMAccountStatus::Active,
        2_000,
        100,
    )
    .unwrap();
    assert!(validate_active_crm_account(account.clone()).is_ok());
    let contact = crm_contact();
    let account_contact = validate_crm_account_contact(account.clone(), contact.clone()).unwrap();
    let permitted =
        validate_permitted_customer_message(InteractionId::new(1), contact.clone(), epoch())
            .unwrap();
    assert!(validate_permitted_account_message(account_contact.clone(), permitted).is_ok());
    let interaction = validate_crm_interaction(
        InteractionId::new(2),
        AccountId::new(1),
        ContactId::new(2),
        InteractionKind::Call,
        epoch(),
        later(1),
    )
    .unwrap();
    assert!(validate_crm_interaction_for_contact(account_contact.clone(), interaction).is_ok());
    let lead = validate_lead(
        LeadId::new(1),
        AccountId::new(1),
        ContactId::new(2),
        None,
        LeadStatus::Converted,
        500,
        Currency::USD,
        epoch(),
        later(1),
    )
    .unwrap();
    assert!(validate_lead_for_contact(account_contact.clone(), lead).is_ok());
    let opportunity = validate_sales_opportunity(
        OpportunityId::new(1),
        AccountId::new(1),
        ContactId::new(2),
        Some(LeadId::new(1)),
        OpportunityStage::Qualified,
        400,
        Currency::USD,
        BasisPoints::try_new(5_000).unwrap(),
        epoch(),
        later(1),
        later(30),
    )
    .unwrap();
    assert!(validate_opportunity_for_contact(account_contact.clone(), opportunity).is_ok());
    assert!(validate_sales_pipeline(Currency::USD, vec![opportunity]).is_ok());
    let segment =
        validate_customer_segment(SegmentId::new(1), "vip".to_owned(), 1, 100, 50).unwrap();
    assert!(validate_segment_membership(account.clone(), segment.clone()).is_ok());
    let case = validate_support_case(
        SupportCaseId::new(1),
        AccountId::new(1),
        ContactId::new(2),
        Some(order.id()),
        SupportCaseStatus::Escalated,
        SupportPriority::Urgent,
        epoch(),
        later(1),
        later(5),
    )
    .unwrap();
    assert!(validate_support_case_for_contact(account_contact.clone(), case).is_ok());
    let resolved_case = validate_support_case(
        SupportCaseId::new(2),
        AccountId::new(1),
        ContactId::new(2),
        Some(order.id()),
        SupportCaseStatus::Resolved,
        SupportPriority::High,
        epoch(),
        later(1),
        later(5),
    )
    .unwrap();
    assert!(validate_resolved_support_case(resolved_case, later(2)).is_ok());
    assert!(
        validate_retention_offer(account.clone(), segment, Coupon::new(50, 100, 10), 1, 25).is_ok()
    );

    let plan = logistics_plan_for(order.clone());
    assert!(
        validate_logistics_shipment(
            ShipmentId::new(1),
            plan.clone(),
            ShipmentStatus::Planned,
            epoch(),
            later(1)
        )
        .is_ok()
    );
    assert!(validate_carrier_handoff(plan.clone(), 999, epoch(), later(1)).is_ok());
    assert!(
        validate_warehouse_transfer(
            TransferId::new(1),
            sku(10),
            warehouse(1),
            warehouse(2),
            StockState::try_new(sku(10), 10, 0).unwrap(),
            5,
            3,
            2
        )
        .is_ok()
    );
    let authorization = validate_return_authorization(
        ReturnAuthorizationId::new(1),
        validate_support_case(
            SupportCaseId::new(3),
            AccountId::new(1),
            ContactId::new(2),
            Some(order.id()),
            SupportCaseStatus::Opened,
            SupportPriority::Normal,
            epoch(),
            later(1),
            later(5),
        )
        .unwrap(),
        order.clone(),
        PaymentLedger::try_new(order.total(), 0).unwrap(),
        ReturnAuthorizationStatus::Approved,
        vec![ReturnLine::new(sku(10), 1, 100)],
        1,
        100,
        epoch(),
        later(1),
    )
    .unwrap();
    let receipt = validate_return_receipt(authorization.clone(), 1, 50, later(2)).unwrap();

    assert!(
        validate_captured_payment_matches_order(
            order.clone(),
            CapturedPayment::new(order.id(), order.total(), Currency::USD)
        )
        .is_ok()
    );
    let sourceable = validate_sourceable_distributor_product(
        DistributorProduct::new(
            SupplierId::new(1),
            sku(10),
            40,
            5,
            10,
            2,
            Currency::USD,
            true,
        ),
        2,
    );
    assert!(sourceable.is_ok());
    assert!(
        validate_wholesale_credit_checkout(
            wholesale_account,
            vec![wholesale],
            PaymentTerms::NetDays(30),
            155
        )
        .is_ok()
    );
    assert!(
        validate_fresh_currency_conversion(
            MoneyAmount::new(100, Currency::USD),
            ExchangeRate::try_new(Currency::USD, Currency::EUR, 9, 10, epoch()).unwrap(),
            MoneyAmount::new(90, Currency::EUR),
            later(1),
            days(2),
        )
        .is_ok()
    );
    let converted = validate_converted_lead_opportunity(lead, opportunity);
    assert!(converted.is_ok());
    let crm_order =
        validate_crm_order_contact(account.clone(), contact.clone(), order.clone()).unwrap();
    assert!(validate_shipment_for_crm_order(crm_order, plan.clone()).is_ok());
    assert!(
        validate_logistics_exception_support_case(
            LogisticsException::new(
                ShipmentId::new(1),
                LogisticsExceptionKind::CarrierDelay,
                epoch(),
                true
            ),
            plan,
            validate_support_case(
                SupportCaseId::new(4),
                AccountId::new(1),
                ContactId::new(2),
                Some(order.id()),
                SupportCaseStatus::Escalated,
                SupportPriority::Urgent,
                epoch(),
                later(1),
                later(5)
            )
            .unwrap(),
        )
        .is_ok()
    );
    assert!(validate_crm_approved_return_handling(authorization, receipt).is_ok());
    assert_eq!(versioned.version(), 1);
    assert_eq!(feed.price_policy(), &policy);
    assert_eq!(synced.stock().sku(), sku(10));
    assert_eq!(catalog_entry.variant().sku(), sku(10));
}
