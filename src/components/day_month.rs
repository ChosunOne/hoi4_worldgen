use serde::de::Visitor;
use serde::{de, Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::num::ParseIntError;
use std::str::FromStr;

/// Zero-indexed day of the month (0-30) and month of the year (0-11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[non_exhaustive]
pub struct DayMonth {
    /// The zero-indexed day of the month (0-30).
    pub day: u8,
    /// The zero-indexed month of the year (0-11).
    pub month: u8,
}

/// An error parsing a `DayMonth`.
#[non_exhaustive]
pub struct DayMonthParseError;

impl Debug for DayMonthParseError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("DayMonthParseError")
    }
}

impl Display for DayMonthParseError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("DayMonthParseError")
    }
}

impl Error for DayMonthParseError {}
impl de::Error for DayMonthParseError {
    #[inline]
    fn custom<T: Display>(_msg: T) -> Self {
        DayMonthParseError
    }
}

impl From<ParseIntError> for DayMonthParseError {
    #[inline]
    fn from(_: ParseIntError) -> Self {
        DayMonthParseError
    }
}

impl FromStr for DayMonth {
    type Err = DayMonthParseError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split('.').collect::<Vec<_>>();
        if parts.len() != 2 {
            return Err(DayMonthParseError);
        }
        let day = parts.get(0).ok_or(DayMonthParseError)?.parse::<u8>()?;
        if day > 30 {
            return Err(DayMonthParseError);
        }
        let month = parts.get(1).ok_or(DayMonthParseError)?.parse::<u8>()?;
        if month > 11 {
            return Err(DayMonthParseError);
        }
        Ok(Self { day, month })
    }
}

/// Visitor for a `DayMonth`
#[derive(Debug)]
#[non_exhaustive]
pub struct DayMonthVisitor;

impl<'de> Visitor<'de> for DayMonthVisitor {
    type Value = DayMonth;

    #[inline]
    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str(
            "a string of the form `DD.MM`, where DD and MM are zero indexed day and month",
        )
    }

    #[inline]
    fn visit_str<E: de::Error>(self, s: &str) -> Result<Self::Value, E> {
        let daymonth = match s.parse() {
            Ok(a) => a,
            Err(_) => return Err(de::Error::custom("invalid day month")),
        };
        Ok(daymonth)
    }
}

impl<'de> Deserialize<'de> for DayMonth {
    #[inline]
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(DayMonthVisitor)
    }
}
