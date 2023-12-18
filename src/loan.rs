use std::{error::Error, fmt::Display, ops::Add};

use chrono::{Duration, NaiveDate};
use rust_decimal::prelude::{Decimal, Zero};

const DAYS_IN_YEAR: u64 = 365;

#[derive(Debug, Clone, Copy)]
pub enum CurrencyCode {
    GBP,
    EUR,
    USD,
}

impl Display for CurrencyCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CurrencyCode::GBP => f.write_str("GBP"),
            CurrencyCode::EUR => f.write_str("EUR"),
            CurrencyCode::USD => f.write_str("USD"),
        }
    }
}

impl CurrencyCode {
    fn symbol(&self) -> &str {
        match self {
            CurrencyCode::GBP => "£",
            CurrencyCode::EUR => "€",
            CurrencyCode::USD => "$",
        }
    }
}

#[derive(Debug)]
pub struct UnknownCurrencyError {
    currency_code: String,
}

impl UnknownCurrencyError {
    fn new(currency_code: String) -> Self {
        Self { currency_code }
    }
}

impl Display for UnknownCurrencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Error unknown currency code: {}",
            self.currency_code
        ))
    }
}

impl Error for UnknownCurrencyError {}

impl TryFrom<&str> for CurrencyCode {
    type Error = UnknownCurrencyError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_uppercase().as_str() {
            "GBP" => Ok(CurrencyCode::GBP),
            "EUR" => Ok(CurrencyCode::EUR),
            "USD" => Ok(CurrencyCode::USD),
            _ => Err(UnknownCurrencyError::new(value.into())),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Money {
    pub value: Decimal,
    pub code: CurrencyCode,
}

impl Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}{:.2}", self.code.symbol(), self.value))
    }
}

impl Add<Decimal> for Money {
    type Output = Self;
    fn add(self, rhs: Decimal) -> Self::Output {
        Self {
            value: self.value + rhs,
            code: self.code,
        }
    }
}

impl Add<Money> for Decimal {
    type Output = Money;
    fn add(self, rhs: Money) -> Self::Output {
        Self::Output {
            value: self + rhs.value,
            code: rhs.code,
        }
    }
}

// Round a Decimal using Banker's Rounding
// SEE: https://en.wikipedia.org/wiki/Rounding#Rounding_half_to_even
fn bankers_round(money: Money) -> Money {
    let scale = 2;
    let new_value = money
        .value
        .round_dp_with_strategy(scale, rust_decimal::RoundingStrategy::MidpointNearestEven);
    Money {
        value: new_value,
        code: money.code,
    }
}

#[derive(Debug)]
pub struct Entry {
    pub daily_interest_without_margin: Money,
    pub daily_interest_with_margin: Money,
    pub accrual_date: NaiveDate,
    pub days_elapsed: u64,
}

#[derive(Debug)]
pub struct Schedule {
    pub entries: std::vec::Vec<Entry>,
}

impl Iterator for Schedule {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        self.entries.pop()
    }
}

impl From<Vec<Entry>> for Schedule {
    fn from(entries: Vec<Entry>) -> Self {
        Schedule { entries }
    }
}

#[derive(Debug)]
pub struct TotalInterest {
    pub with_margin: Money,
    pub without_margin: Money,
}

#[derive(Debug)]
pub struct Loan {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub loan_amount: Decimal,
    pub base_rate: Decimal,
    pub margin: Decimal,
    pub currency: CurrencyCode,
}

impl Loan {
    pub fn new(
        start_date: NaiveDate,
        end_date: NaiveDate,
        loan_amount: Decimal,
        base_rate: Decimal,
        margin: Decimal,
        currency: CurrencyCode,
    ) -> Self {
        Self {
            start_date,
            end_date,
            loan_amount,
            base_rate,
            margin,
            currency,
        }
    }
}

// Calculates daily interest without margin
fn daily_interest_without_margin(loan: &Loan) -> Money {
    let daily_rate = loan.base_rate / Decimal::from(DAYS_IN_YEAR) / Decimal::from(100);
    Money {
        value: (loan.loan_amount * daily_rate).into(),
        code: loan.currency,
    }
}

// calculates the daily interest with margin
fn daily_interest_with_margin(loan: &Loan) -> Money {
    let daily_rate =
        (loan.base_rate + loan.margin) / Decimal::from(DAYS_IN_YEAR) / Decimal::from(100);
    Money {
        value: (loan.loan_amount * daily_rate).into(),
        code: loan.currency,
    }
}

impl Schedule {
    pub fn new(loan: &Loan) -> Self {
        let duration = loan.end_date.signed_duration_since(loan.start_date);
        println!("{:?}", loan);
        println!("duration = {}", duration.num_days());
        (0..=duration.num_days() as u64)
            .map(|days_elapsed| {
                let accrual_date = loan.start_date + Duration::days(days_elapsed as i64);
                let daily_interest_without_margin = daily_interest_without_margin(loan);
                let daily_interest_with_margin = daily_interest_with_margin(loan);
                Entry {
                    daily_interest_without_margin,
                    daily_interest_with_margin,
                    accrual_date,
                    days_elapsed,
                }
            })
            .collect::<Vec<_>>()
            .into()
    }

    pub fn calculate_interest(&self) -> Option<TotalInterest> {
        if self.entries.is_empty() {
            return None;
        }

        let (interest_with_margin, interest_without_margin) = self.entries.iter().fold(
            (Decimal::zero(), Decimal::zero()),
            |(interest_with_margin, interest_without_margin), entry| {
                (
                    (interest_with_margin + bankers_round(entry.daily_interest_with_margin)).value,
                    (interest_without_margin + bankers_round(entry.daily_interest_without_margin))
                        .value,
                )
            },
        );

        let currency_code = self.entries[0].daily_interest_with_margin.code;

        Some(TotalInterest {
            with_margin: Money {
                value: interest_with_margin,
                code: currency_code,
            },
            without_margin: Money {
                value: interest_without_margin,
                code: currency_code,
            },
        })
    }
}
