//! Deterministic comparison of local and exchange order views.

/// Simplified execution statuses reported by an order source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

/// Locally stored order state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalOrderView {
    pub id: String,
    pub status: ExecutionStatus,
    pub filled_quantity: u64,
}

/// Exchange-reported order state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExchangeOrderView {
    pub id: String,
    pub status: ExecutionStatus,
    pub filled_quantity: u64,
}

/// Explicit action produced by comparing local and exchange state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconciliationAction {
    NoChange,
    UpdateLocalStatus {
        from: ExecutionStatus,
        to: ExecutionStatus,
    },
    UpdateLocalFill {
        from: u64,
        to: u64,
    },
    UpdateStatusAndFill {
        status_from: ExecutionStatus,
        status_to: ExecutionStatus,
        fill_from: u64,
        fill_to: u64,
    },
    ExchangeFillRegressed {
        local_fill: u64,
        exchange_fill: u64,
    },
    IdentifierMismatch,
}

/// Compares two views without mutating either source.
pub fn reconcile(
    local: &LocalOrderView,
    exchange: &ExchangeOrderView,
) -> ReconciliationAction {
    if local.id != exchange.id {
        return ReconciliationAction::IdentifierMismatch;
    }

    if exchange.filled_quantity < local.filled_quantity {
        return ReconciliationAction::ExchangeFillRegressed {
            local_fill: local.filled_quantity,
            exchange_fill: exchange.filled_quantity,
        };
    }

    let status_changed = local.status != exchange.status;
    let fill_changed = local.filled_quantity != exchange.filled_quantity;

    match (status_changed, fill_changed) {
        (false, false) => ReconciliationAction::NoChange,
        (true, false) => ReconciliationAction::UpdateLocalStatus {
            from: local.status,
            to: exchange.status,
        },
        (false, true) => ReconciliationAction::UpdateLocalFill {
            from: local.filled_quantity,
            to: exchange.filled_quantity,
        },
        (true, true) => ReconciliationAction::UpdateStatusAndFill {
            status_from: local.status,
            status_to: exchange.status,
            fill_from: local.filled_quantity,
            fill_to: exchange.filled_quantity,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_status_and_fill_drift() {
        let local = LocalOrderView {
            id: "order-7".into(),
            status: ExecutionStatus::Open,
            filled_quantity: 0,
        };

        let exchange = ExchangeOrderView {
            id: "order-7".into(),
            status: ExecutionStatus::PartiallyFilled,
            filled_quantity: 3,
        };

        assert_eq!(
            reconcile(&local, &exchange),
            ReconciliationAction::UpdateStatusAndFill {
                status_from: ExecutionStatus::Open,
                status_to: ExecutionStatus::PartiallyFilled,
                fill_from: 0,
                fill_to: 3,
            }
        );
    }

    #[test]
    fn fill_regression_is_flagged_instead_of_applied() {
        let local = LocalOrderView {
            id: "order-8".into(),
            status: ExecutionStatus::PartiallyFilled,
            filled_quantity: 5,
        };

        let exchange = ExchangeOrderView {
            id: "order-8".into(),
            status: ExecutionStatus::PartiallyFilled,
            filled_quantity: 3,
        };

        assert_eq!(
            reconcile(&local, &exchange),
            ReconciliationAction::ExchangeFillRegressed {
                local_fill: 5,
                exchange_fill: 3,
            }
        );
    }

    #[test]
    fn identifier_mismatch_is_reported() {
        let local = LocalOrderView {
            id: "local-order".into(),
            status: ExecutionStatus::Open,
            filled_quantity: 0,
        };

        let exchange = ExchangeOrderView {
            id: "exchange-order".into(),
            status: ExecutionStatus::Open,
            filled_quantity: 0,
        };

        assert_eq!(reconcile(&local, &exchange), ReconciliationAction::IdentifierMismatch);
    }
}
