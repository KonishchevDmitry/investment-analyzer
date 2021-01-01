use std::ops::{Neg, Add};
use std::str::FromStr;

use chrono::{self, Duration, Local, TimeZone};
use chrono_tz::Tz;
use num_traits::Zero;
use regex::Regex;
use rust_decimal::RoundingStrategy;

use crate::core::GenericResult;
use crate::currency::Cash;
use crate::formatting;
use crate::types::{Date, Time, DateTime, Decimal};

#[derive(Clone, Copy)]
pub enum DecimalRestrictions {
    No,
    Zero,
    NonZero,
    NegativeOrZero,
    PositiveOrZero,
    StrictlyPositive,
    StrictlyNegative,
}

pub fn parse_decimal(string: &str, restrictions: DecimalRestrictions) -> GenericResult<Decimal> {
    let value = Decimal::from_str(string).map_err(|_| "Invalid decimal value")?;
    validate_decimal(value, restrictions)
}

pub fn validate_decimal(value: Decimal, restrictions: DecimalRestrictions) -> GenericResult<Decimal> {
    if !match restrictions {
        DecimalRestrictions::No => true,
        DecimalRestrictions::Zero => value.is_zero(),
        DecimalRestrictions::NonZero => !value.is_zero(),
        DecimalRestrictions::NegativeOrZero => value.is_sign_negative() || value.is_zero(),
        DecimalRestrictions::PositiveOrZero => value.is_sign_positive() || value.is_zero(),
        DecimalRestrictions::StrictlyPositive => value.is_sign_positive() && !value.is_zero(),
        DecimalRestrictions::StrictlyNegative => value.is_sign_negative() && !value.is_zero(),
    } {
        return Err!("The value doesn't comply to the specified restrictions");
    }

    Ok(value)
}

pub fn validate_named_decimal(name: &str, value: Decimal, restrictions: DecimalRestrictions) -> GenericResult<Decimal> {
    Ok(validate_decimal(value, restrictions).map_err(|e| format!(
        "Invalid {} ({}): {}", name, value, e))?)
}

pub fn validate_named_cash(name: &str, currency: &str, value: Decimal, restrictions: DecimalRestrictions) -> GenericResult<Cash> {
    Ok(Cash::new(currency, validate_named_decimal(name, value, restrictions)?))
}

pub fn decimal_precision(value: Decimal) -> u32 {
    value.fract().scale()
}

pub fn round(value: Decimal, points: u32) -> Decimal {
    round_with(value, points, RoundingMethod::Round)
}

#[derive(Clone, Copy, Debug)]
pub enum RoundingMethod {
    Round,
    Truncate,
}

pub fn round_with(value: Decimal, points: u32, method: RoundingMethod) -> Decimal {
    let mut round_value = match method {
        RoundingMethod::Round => value.round_dp_with_strategy(points, RoundingStrategy::RoundHalfUp),
        RoundingMethod::Truncate => {
            let mut value = value;
            let scale = value.scale();

            if scale > points {
                value.set_scale(scale - points).unwrap();
                value = value.trunc();
                value.set_scale(points).unwrap();
            }

            value
        },
    };

    if round_value.is_zero() && round_value.is_sign_negative() {
        round_value = round_value.neg();
    }

    round_value.normalize()
}

pub fn parse_period(start: Date, end: Date) -> GenericResult<(Date, Date)> {
    let period = (start, end.succ());

    if period.0 >= period.1 {
        return Err!("Invalid period: {}", formatting::format_period(period));
    }

    Ok(period)
}

pub fn parse_date(date: &str, format: &str) -> GenericResult<Date> {
    Ok(Date::parse_from_str(date, format).map_err(|_| format!(
        "Invalid date: {:?}", date))?)
}

pub fn parse_time(time: &str, format: &str) -> GenericResult<Time> {
    Ok(Time::parse_from_str(time, format).map_err(|_| format!(
        "Invalid time: {:?}", time))?)
}

pub fn parse_date_time(date_time: &str, format: &str) -> GenericResult<DateTime> {
    Ok(DateTime::parse_from_str(date_time, format).map_err(|_| format!(
        "Invalid time: {:?}", date_time))?)
}

pub fn parse_tz_date_time<T: TimeZone>(
    string: &str, format: &str, tz: T, future_check: bool,
) -> GenericResult<chrono::DateTime<T>> {
    let date_time = tz.datetime_from_str(string, format).map_err(|_| format!(
        "Invalid time: {:?}", string))?;

    if future_check && (date_time.naive_utc() - utc_now()).num_hours() > 0 {
        return Err!("Invalid time: {:?}. It's from future", date_time);
    }

    Ok(date_time)
}

pub fn parse_timezone(timezone: &str) -> GenericResult<Tz> {
    Ok(timezone.parse().map_err(|_| format!("Invalid time zone: {:?}", timezone))?)
}

pub fn parse_duration(string: &str) -> GenericResult<Duration> {
    let re = Regex::new(r"^(?P<number>[1-9]\d*)(?P<unit>[mhd])$").unwrap();

    let seconds = re.captures(string).and_then(|captures| {
        let mut duration = match captures.name("number").unwrap().as_str().parse::<i64>().ok() {
            Some(duration) if duration > 0 => duration,
            _ => return None,
        };

        duration *= match captures.name("unit").unwrap().as_str() {
            "m" => 60,
            "h" => 60 * 60,
            "d" => 60 * 60 * 24,
            _ => unreachable!(),
        };

        Some(duration)
    }).ok_or_else(|| format!("Invalid duration: {}", string))?;

    Ok(Duration::seconds(seconds))
}

pub fn today() -> Date {
    tz_now().date().naive_local()
}

pub fn today_trade_conclusion_date() -> Date {
    today()
}

pub fn today_trade_execution_date() -> Date {
    today().add(Duration::days(2))
}

pub fn now() -> DateTime {
    tz_now().naive_local()
}

pub fn utc_now() -> DateTime {
    tz_now().naive_utc()
}

fn tz_now() -> chrono::DateTime<Local> {
    #[cfg(debug_assertions)]
    {
        use std::process;
        use lazy_static::lazy_static;

        lazy_static! {
            static ref FAKE_NOW: Option<chrono::DateTime<Local>> = parse_fake_now().unwrap_or_else(|e| {
                eprintln!("{}.", e);
                process::exit(1);
            });
        }

        if let Some(&now) = FAKE_NOW.as_ref() {
            return now;
        }
    }

    chrono::Local::now()
}

#[cfg(debug_assertions)]
fn parse_fake_now() -> GenericResult<Option<chrono::DateTime<Local>>> {
    use std::env::{self, VarError};

    let name = "INVESTMENTS_NOW";

    match env::var(name) {
        Ok(value) => {
            let timezone = chrono::Local::now().timezone();
            if let Ok(now) = timezone.datetime_from_str(&value, "%Y.%m.%d %H:%M:%S") {
                return Ok(Some(now));
            }
        },
        Err(e) => match e {
            VarError::NotPresent => return Ok(None),
            VarError::NotUnicode(_) => {},
        },
    };

    Err!("Invalid {} environment variable value", name)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use super::*;

    #[rstest(num, scale, precision,
        case(321, 0, 0),
        case(321, 1, 1),
        case(321, 2, 2),
        case(321, 3, 3),
        case(321, 4, 4),

        case(3210, 0, 0),
        case(3210, 1, 1),
        case(3210, 2, 2),
        case(3210, 3, 3),
        case(3210, 4, 4),
        case(3210, 5, 5),
    )]
    fn decimal_precision(num: i64, scale: u32, precision: u32) {
        let value = Decimal::new(num, scale);
        assert_eq!(super::decimal_precision(value), precision)
    }

    #[rstest(value, expected,
        case(dec!(-1.5), dec!(-2)),
        case(dec!(-1.4), dec!(-1)),
        case(dec!(-1),   dec!(-1)),
        case(dec!(-0.5), dec!(-1)),
        case(dec!(-0.4), dec!(0)),
        case(dec!( 0), dec!(0)),
        case(dec!(-0), dec!(0)),
        case(dec!(0.4), dec!(0)),
        case(dec!(0.5), dec!(1)),
        case(dec!(1),   dec!(1)),
        case(dec!(1.4), dec!(1)),
        case(dec!(1.5), dec!(2)),
    )]
    fn rounding(value: Decimal, expected: Decimal) {
        assert_eq!(round(value, 0), expected);
    }

    #[rstest(value, expected,
        case(dec!(-1.6), dec!(-1)),
        case(dec!(-1.4), dec!(-1)),
        case(dec!(-1),   dec!(-1)),
        case(dec!(-0.6), dec!(0)),
        case(dec!(-0.4), dec!(0)),
        case(dec!( 0), dec!(0)),
        case(dec!(-0), dec!(0)),
        case(dec!(0.4), dec!(0)),
        case(dec!(0.6), dec!(0)),
        case(dec!(1),   dec!(1)),
        case(dec!(1.4), dec!(1)),
        case(dec!(1.6), dec!(1)),
    )]
    fn truncate_rounding(value: Decimal, expected: Decimal) {
        assert_eq!(round_with(value, 0, RoundingMethod::Truncate), expected);
    }
}