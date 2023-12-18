mod loan;

use loan::Loan;

use chrono::NaiveDate;
use clap::Parser;
use loan::Schedule;
use prettytable::{Row, Table};
use rust_decimal::prelude::*;
use rust_decimal::Decimal;

#[macro_use]
extern crate prettytable;

/// Custom validator for currency format (e.g., USD, EUR, etc.)
fn validate_currency_format(value: &str) -> Result<String, String> {
    if value.chars().all(|c| c.is_ascii_uppercase()) && value.len() >= 3 && value.len() <= 5 {
        Ok(value.into())
    } else {
        Err("Invalid currency format. Please use uppercase letters (e.g., USD, EUR).".to_string())
    }
}

/// Custom validator for date format (YYYY-MM-DD)
fn validate_date_format(value: &str) -> Result<NaiveDate, String> {
    if let Ok(date) = NaiveDate::parse_from_str(&value, "%Y-%m-%d") {
        Ok(date)
    } else {
        Err("Invalid date format. Please use the format YYYY-MM-DD.".to_string())
    }
}

// Round a Decimal using Banker's Rounding
// SEE: https://en.wikipedia.org/wiki/Rounding#Rounding_half_to_even
fn bankers_round(value: Decimal) -> Decimal {
    let scale = 2;
    value.round_dp_with_strategy(scale, rust_decimal::RoundingStrategy::MidpointNearestEven)
}

#[derive(Parser, Debug)]
struct Args {
    /// Start Date (format: YYYY-MM-DD)
    #[arg(long, value_parser = validate_date_format)]
    start_date: NaiveDate,

    /// End Date (format: YYYY-MM-DD)
    #[arg(long, value_parser = validate_date_format)]
    end_date: NaiveDate,

    /// Loan Amount
    #[arg(long)]
    loan_amount: Decimal,

    /// Loan Currency
    #[arg(long, value_parser = validate_currency_format)]
    loan_currency: String,

    /// Base Interest Rate
    #[arg(long)]
    base_interest_rate: Decimal,

    /// Margin Interest Rate
    #[arg(long)]
    margin: Decimal,
}

fn main() {
    let args: Args = Args::parse();

    let start_date = args.start_date;
    let end_date = args.end_date;
    let loan_amount = args.loan_amount;
    let currency = args.loan_currency;
    let base_rate = args.base_interest_rate;
    let margin = args.margin;

    let loan = Loan::new(
        start_date,
        end_date,
        loan_amount,
        base_rate,
        margin,
        currency,
    );

    let schedule = Schedule::new(&loan);

    let total_interest = schedule.calculate_interest();

    // Create a table
    let mut table = Table::new();

    table.add_row(row![
        "Accrual Date",
        "Days Elapsed",
        "Interest Without Margin",
        "Interest With Margin",
        "Currency",
    ]);

    schedule.entries.iter().for_each(|entry| {
        let formatted_interest_without_margin =
            format!("{:.2}", bankers_round(entry.daily_interest_without_margin));
        let formatted_interest_with_margin =
            format!("{:.2}", bankers_round(entry.daily_interest_with_margin));
        table.add_row(row![
            entry.accrual_date,
            entry.days_elapsed,
            formatted_interest_without_margin,
            formatted_interest_with_margin,
            loan.currency,
        ]);
    });

    table.add_row(row![
        "Total",
        "",
        total_interest.without_margin,
        total_interest.with_margin,
        loan.currency
    ]);

    table.printstd();
}
