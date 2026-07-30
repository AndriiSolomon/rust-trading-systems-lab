//! Pre-trade validation for simplified notional and daily-loss limits.

/// Configured risk limits for new order validation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RiskLimits {
    pub max_order_notional: f64,
    pub max_position_notional: f64,
    pub daily_loss_limit: f64,
}

/// Current account state used by the risk validator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RiskSnapshot {
    pub current_position_notional: f64,
    pub realized_pnl_today: f64,
}

/// Proposed order values used before order creation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrderIntent {
    pub price: f64,
    pub quantity: f64,
}

/// Typed reasons why a proposed order cannot pass risk validation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RiskViolation {
    InvalidMaxOrderNotional,
    InvalidMaxPositionNotional,
    InvalidDailyLossLimit,
    InvalidCurrentPositionNotional,
    InvalidRealizedPnl,
    InvalidPrice,
    InvalidQuantity,
    NonFiniteOrderNotional,
    NonFiniteProjectedPositionNotional,
    OrderNotionalExceeded {
        order_notional: f64,
        limit: f64,
    },
    PositionNotionalExceeded {
        projected_position_notional: f64,
        limit: f64,
    },
    DailyLossLimitReached {
        realized_pnl_today: f64,
        limit: f64,
    },
}

/// Validates a proposed order against input integrity and configured limits.
pub fn validate_order(
    limits: RiskLimits,
    snapshot: RiskSnapshot,
    intent: OrderIntent,
) -> Result<(), RiskViolation> {
    if !limits.max_order_notional.is_finite() || limits.max_order_notional <= 0.0 {
        return Err(RiskViolation::InvalidMaxOrderNotional);
    }

    if !limits.max_position_notional.is_finite() || limits.max_position_notional <= 0.0 {
        return Err(RiskViolation::InvalidMaxPositionNotional);
    }

    if !limits.daily_loss_limit.is_finite() || limits.daily_loss_limit <= 0.0 {
        return Err(RiskViolation::InvalidDailyLossLimit);
    }

    if !snapshot.current_position_notional.is_finite()
        || snapshot.current_position_notional < 0.0
    {
        return Err(RiskViolation::InvalidCurrentPositionNotional);
    }

    if !snapshot.realized_pnl_today.is_finite() {
        return Err(RiskViolation::InvalidRealizedPnl);
    }

    if !intent.price.is_finite() || intent.price <= 0.0 {
        return Err(RiskViolation::InvalidPrice);
    }

    if !intent.quantity.is_finite() || intent.quantity <= 0.0 {
        return Err(RiskViolation::InvalidQuantity);
    }

    if snapshot.realized_pnl_today <= -limits.daily_loss_limit {
        return Err(RiskViolation::DailyLossLimitReached {
            realized_pnl_today: snapshot.realized_pnl_today,
            limit: limits.daily_loss_limit,
        });
    }

    let order_notional = intent.price * intent.quantity;
    if !order_notional.is_finite() {
        return Err(RiskViolation::NonFiniteOrderNotional);
    }

    if order_notional > limits.max_order_notional {
        return Err(RiskViolation::OrderNotionalExceeded {
            order_notional,
            limit: limits.max_order_notional,
        });
    }

    let projected_position_notional = snapshot.current_position_notional + order_notional;
    if !projected_position_notional.is_finite() {
        return Err(RiskViolation::NonFiniteProjectedPositionNotional);
    }

    if projected_position_notional > limits.max_position_notional {
        return Err(RiskViolation::PositionNotionalExceeded {
            projected_position_notional,
            limit: limits.max_position_notional,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> RiskLimits {
        RiskLimits {
            max_order_notional: 1_000.0,
            max_position_notional: 5_000.0,
            daily_loss_limit: 250.0,
        }
    }

    #[test]
    fn valid_order_passes() {
        let result = validate_order(
            limits(),
            RiskSnapshot {
                current_position_notional: 2_000.0,
                realized_pnl_today: -50.0,
            },
            OrderIntent {
                price: 100.0,
                quantity: 5.0,
            },
        );

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn daily_loss_limit_blocks_new_order() {
        let result = validate_order(
            limits(),
            RiskSnapshot {
                current_position_notional: 0.0,
                realized_pnl_today: -250.0,
            },
            OrderIntent {
                price: 100.0,
                quantity: 1.0,
            },
        );

        assert_eq!(
            result,
            Err(RiskViolation::DailyLossLimitReached {
                realized_pnl_today: -250.0,
                limit: 250.0,
            })
        );
    }

    #[test]
    fn non_finite_snapshot_is_rejected() {
        let result = validate_order(
            limits(),
            RiskSnapshot {
                current_position_notional: f64::NAN,
                realized_pnl_today: 0.0,
            },
            OrderIntent {
                price: 100.0,
                quantity: 1.0,
            },
        );

        assert_eq!(result, Err(RiskViolation::InvalidCurrentPositionNotional));
    }

    #[test]
    fn overflowing_order_notional_is_rejected() {
        let result = validate_order(
            RiskLimits {
                max_order_notional: f64::MAX,
                max_position_notional: f64::MAX,
                daily_loss_limit: 250.0,
            },
            RiskSnapshot {
                current_position_notional: 0.0,
                realized_pnl_today: 0.0,
            },
            OrderIntent {
                price: f64::MAX,
                quantity: 2.0,
            },
        );

        assert_eq!(result, Err(RiskViolation::NonFiniteOrderNotional));
    }
}
