//! Deterministic order-state transitions with protected invariants.

use std::fmt;

/// Lifecycle states for a simplified exchange order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    New,
    Accepted,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

/// Events that may change an order's lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderEvent {
    Accept,
    Fill(u64),
    Cancel,
    Reject,
}

/// An order whose internal state can change only through [`Order::apply`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    id: String,
    quantity: u64,
    filled_quantity: u64,
    status: OrderStatus,
}

/// Errors returned when an order cannot be created or transitioned safely.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderError {
    EmptyIdentifier,
    ZeroQuantity,
    InvalidTransition {
        status: OrderStatus,
        event: OrderEvent,
    },
    FillExceedsRemaining {
        requested: u64,
        remaining: u64,
    },
    ZeroFill,
}

impl fmt::Display for OrderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyIdentifier => write!(f, "order identifier must not be empty"),
            Self::ZeroQuantity => write!(f, "order quantity must be greater than zero"),
            Self::InvalidTransition { status, event } => {
                write!(f, "event {event:?} is invalid for status {status:?}")
            }
            Self::FillExceedsRemaining {
                requested,
                remaining,
            } => write!(
                f,
                "fill quantity {requested} exceeds remaining quantity {remaining}"
            ),
            Self::ZeroFill => write!(f, "fill quantity must be greater than zero"),
        }
    }
}

impl std::error::Error for OrderError {}

impl Order {
    /// Creates a new order while validating its initial invariants.
    pub fn new(id: impl Into<String>, quantity: u64) -> Result<Self, OrderError> {
        let id = id.into();

        if id.trim().is_empty() {
            return Err(OrderError::EmptyIdentifier);
        }

        if quantity == 0 {
            return Err(OrderError::ZeroQuantity);
        }

        Ok(Self {
            id,
            quantity,
            filled_quantity: 0,
            status: OrderStatus::New,
        })
    }

    /// Returns the exchange or client order identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the original order quantity.
    pub const fn quantity(&self) -> u64 {
        self.quantity
    }

    /// Returns the cumulative filled quantity.
    pub const fn filled_quantity(&self) -> u64 {
        self.filled_quantity
    }

    /// Returns the current lifecycle status.
    pub const fn status(&self) -> OrderStatus {
        self.status
    }

    /// Returns the quantity that remains unfilled.
    pub const fn remaining_quantity(&self) -> u64 {
        self.quantity - self.filled_quantity
    }

    /// Applies one validated lifecycle event.
    pub fn apply(&mut self, event: OrderEvent) -> Result<(), OrderError> {
        match event {
            OrderEvent::Accept if self.status == OrderStatus::New => {
                self.status = OrderStatus::Accepted;
                Ok(())
            }
            OrderEvent::Reject if self.status == OrderStatus::New => {
                self.status = OrderStatus::Rejected;
                Ok(())
            }
            OrderEvent::Cancel
                if matches!(
                    self.status,
                    OrderStatus::New | OrderStatus::Accepted | OrderStatus::PartiallyFilled
                ) =>
            {
                self.status = OrderStatus::Cancelled;
                Ok(())
            }
            OrderEvent::Fill(fill_quantity)
                if matches!(
                    self.status,
                    OrderStatus::Accepted | OrderStatus::PartiallyFilled
                ) =>
            {
                if fill_quantity == 0 {
                    return Err(OrderError::ZeroFill);
                }

                let remaining = self.remaining_quantity();
                if fill_quantity > remaining {
                    return Err(OrderError::FillExceedsRemaining {
                        requested: fill_quantity,
                        remaining,
                    });
                }

                self.filled_quantity += fill_quantity;
                self.status = if self.filled_quantity == self.quantity {
                    OrderStatus::Filled
                } else {
                    OrderStatus::PartiallyFilled
                };

                Ok(())
            }
            _ => Err(OrderError::InvalidTransition {
                status: self.status,
                event,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_moves_from_new_to_filled() {
        let mut order = Order::new("order-1", 10).expect("valid order");

        order.apply(OrderEvent::Accept).expect("accept");
        order.apply(OrderEvent::Fill(4)).expect("partial fill");
        order.apply(OrderEvent::Fill(6)).expect("final fill");

        assert_eq!(order.status(), OrderStatus::Filled);
        assert_eq!(order.filled_quantity(), 10);
        assert_eq!(order.remaining_quantity(), 0);
    }

    #[test]
    fn overfill_is_rejected_without_mutating_order() {
        let mut order = Order::new("order-2", 5).expect("valid order");
        order.apply(OrderEvent::Accept).expect("accept");

        let error = order.apply(OrderEvent::Fill(6)).expect_err("must fail");

        assert_eq!(
            error,
            OrderError::FillExceedsRemaining {
                requested: 6,
                remaining: 5,
            }
        );
        assert_eq!(order.filled_quantity(), 0);
        assert_eq!(order.status(), OrderStatus::Accepted);
    }

    #[test]
    fn empty_identifier_is_rejected() {
        assert_eq!(Order::new("   ", 1), Err(OrderError::EmptyIdentifier));
    }

    #[test]
    fn terminal_state_rejects_additional_events() {
        let mut order = Order::new("order-3", 1).expect("valid order");
        order.apply(OrderEvent::Accept).expect("accept");
        order.apply(OrderEvent::Fill(1)).expect("fill");

        assert_eq!(
            order.apply(OrderEvent::Cancel),
            Err(OrderError::InvalidTransition {
                status: OrderStatus::Filled,
                event: OrderEvent::Cancel,
            })
        );
    }
}
