use std::fmt::Display;

use chrono::{Duration, NaiveDate};
use rust_decimal::prelude::{Decimal, Zero};

use crate::bankers_round;

const DAYS_IN_YEAR: u64 = 365;

#[derive(Debug)]
pub struct Entry {
    pub daily_interest_without_margin: Decimal,
    pub daily_interest_with_margin: Decimal,
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
pub struct Interest(Decimal);

impl From<Decimal> for Interest {
    fn from(value: Decimal) -> Self {
        Self(value)
    }
}

impl Display for Interest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:.2}", self.0))
    }
}

#[derive(Debug)]
pub struct TotalInterest {
    pub with_margin: Interest,
    pub without_margin: Interest,
}

impl TotalInterest {
    fn new(with_margin: Interest, without_margin: Interest) -> Self {
        Self {
            with_margin,
            without_margin,
        }
    }
}

#[derive(Debug)]
pub struct Loan {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub loan_amount: Decimal,
    pub base_rate: Decimal,
    pub margin: Decimal,
    pub currency: String,
}

impl Loan {
    pub fn new(
        start_date: NaiveDate,
        end_date: NaiveDate,
        loan_amount: Decimal,
        base_rate: Decimal,
        margin: Decimal,
        currency: String,
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
fn daily_interest_without_margin(loan: &Loan) -> Decimal {
    let daily_rate = loan.base_rate / Decimal::from(DAYS_IN_YEAR) / Decimal::from(100);
    loan.loan_amount * daily_rate
}

// calculates the daily interest with margin
fn daily_interest_with_margin(loan: &Loan) -> Decimal {
    let daily_rate =
        (loan.base_rate + loan.margin) / Decimal::from(DAYS_IN_YEAR) / Decimal::from(100);
    loan.loan_amount * daily_rate
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

    pub fn calculate_interest(&self) -> TotalInterest {
        let (interest_with_margin, interest_without_margin) = self
            .entries
            .iter()
            .fold(
                (Decimal::zero(), Decimal::zero()),
                |(interest_with_margin, interest_without_margin), entry| {
                    (
                        interest_with_margin + bankers_round(entry.daily_interest_with_margin),
                        interest_without_margin
                            + bankers_round(entry.daily_interest_without_margin),
                    )
                },
            )
            .into();
        TotalInterest::new(interest_with_margin.into(), interest_without_margin.into())
    }
}
