use rust_trading_systems_lab::order_state::{Order, OrderEvent, OrderStatus};
use rust_trading_systems_lab::risk::{
    validate_order, OrderIntent, RiskLimits, RiskSnapshot,
};

#[test]
fn risk_validation_precedes_order_lifecycle() {
    let limits = RiskLimits {
        max_order_notional: 2_000.0,
        max_position_notional: 10_000.0,
        daily_loss_limit: 500.0,
    };

    validate_order(
        limits,
        RiskSnapshot {
            current_position_notional: 1_000.0,
            realized_pnl_today: -25.0,
        },
        OrderIntent {
            price: 250.0,
            quantity: 4.0,
        },
    )
    .expect("risk validation should pass");

    let mut order = Order::new("integration-order", 4).expect("valid order");
    order.apply(OrderEvent::Accept).expect("accepted");
    order.apply(OrderEvent::Fill(4)).expect("filled");

    assert_eq!(order.status(), OrderStatus::Filled);
    assert_eq!(order.quantity(), 4);
    assert_eq!(order.filled_quantity(), 4);
}
