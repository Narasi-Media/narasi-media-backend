use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Money {
    amount: i64,
}

impl Money {
    pub const ZERO: Money = Money { amount: 0 };

    pub fn from_idr(amount: i64) -> Self {
        Self { amount }
    }

    pub fn amount(&self) -> i64 {
        self.amount
    }

    pub fn calculate_order_price(quantity: u32, price_per_thousand: Money) -> Self {
        if quantity == 0 {
            return Self::ZERO;
        }
        let total = (quantity as f64 / 1000.0) * (price_per_thousand.amount() as f64);
        Self::from_idr(total.ceil() as i64)
    }

    pub fn calculate_partial_refund(
        remains: u32,
        total_quantity: u32,
        total_price: Money,
    ) -> Self {
        if remains == 0 || total_quantity == 0 {
            return Self::ZERO;
        }
        let refund = (remains as f64 / total_quantity as f64) * (total_price.amount() as f64);
        Self::from_idr(refund.floor() as i64)
    }

    pub fn format_idr(&self) -> String {
        let is_negative = self.amount < 0;
        let abs_val = self.amount.abs();
        let s = abs_val.to_string();
        let mut result = String::new();
        let mut count = 0;

        for c in s.chars().rev() {
            if count > 0 && count % 3 == 0 {
                result.push('.');
            }
            result.push(c);
            count += 1;
        }

        let formatted: String = result.chars().rev().collect();
        if is_negative {
            format!("-Rp {}", formatted)
        } else {
            format!("Rp {}", formatted)
        }
    }
}

impl Add for Money {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_idr(self.amount + rhs.amount)
    }
}

impl Sub for Money {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_idr(self.amount - rhs.amount)
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_idr())
    }
}
