use chrono::{Duration, NaiveDate};
use rust_decimal::{prelude::Zero, Decimal};

pub struct Entry {
    pub daily_interest_without_margin: Decimal,
    pub daily_interest_with_margin: Decimal,
    pub accrual_date: NaiveDate,
    pub days_elapsed: u64,
}

pub struct Schedule {
    pub entries: std::vec::Vec<Entry>,
}

impl From<Vec<Entry>> for Schedule {
    fn from(entries: Vec<Entry>) -> Self {
        Schedule { entries }
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

impl Loan {
    // Calculates daily interest without margin
    fn daily_interest_without_margin(&self, to: NaiveDate) -> Decimal {
        if to < self.start_date || to > self.end_date {
            Decimal::zero()
        } else {
            let days_in_year = Decimal::from(365);
            let daily_rate = self.base_rate / days_in_year / Decimal::from(100);
            self.loan_amount * daily_rate
        }
    }

    // Calculates the daily interest with margin
    fn daily_interest_with_margin(&self, to: NaiveDate) -> Decimal {
        if to < self.start_date || to > self.end_date {
            Decimal::zero()
        } else {
            let days_in_year = Decimal::from(365);
            let daily_rate = (self.base_rate + self.margin) / days_in_year / Decimal::from(100);
            self.loan_amount * daily_rate
        }
    }

    // Creates a schedule of daily interest accruals
    pub fn calculate_schedule(&self) -> Schedule {
        let duration = self.end_date.signed_duration_since(self.start_date);
        println!("{:?}", self);
        println!("duration = {}", duration.num_days());
        let mut schedule = Vec::with_capacity(duration.num_days() as usize + 1);

        for days_elapsed in 0..=duration.num_days() as u64 {
            let accrual_date = self.start_date + Duration::days(days_elapsed as i64);
            let daily_interest_without_margin = self.daily_interest_without_margin(accrual_date);
            let daily_interest_with_margin = self.daily_interest_with_margin(accrual_date);

            schedule.push(Entry {
                daily_interest_without_margin,
                daily_interest_with_margin,
                accrual_date,
                days_elapsed,
            });
        }
        schedule.into()
    }
}
