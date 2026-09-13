use crate::address::{Address, Country};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    TooFewLines,
    InvalidCityLine(String),
    InvalidState(String),
    InvalidPostalCode(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::TooFewLines => write!(
                f,
                "address needs at least a street line and a 'City, ST ZIP' line"
            ),
            ParseError::InvalidCityLine(line) => {
                write!(f, "could not read '{line}' as 'City, ST ZIP'")
            }
            ParseError::InvalidState(state) => {
                write!(f, "'{state}' is not a recognized two-letter state code")
            }
            ParseError::InvalidPostalCode(zip) => {
                write!(f, "'{zip}' is not a valid US ZIP code")
            }
        }
    }
}

impl Error for ParseError {}

const US_STATES: &[&str] = &[
    "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "FL", "GA", "HI", "ID", "IL", "IN", "IA",
    "KS", "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ",
    "NM", "NY", "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN", "TX", "UT", "VT",
    "VA", "WA", "WV", "WI", "WY", "DC",
];

/// Parses a free-form US address: an optional recipient line, one street
/// line, an optional unit line, and a trailing "City, ST ZIP" line.
pub fn parse(input: &str) -> Result<Address, ParseError> {
    let lines: Vec<&str> = input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    if lines.len() < 2 {
        return Err(ParseError::TooFewLines);
    }

    let (last, rest) = lines.split_last().expect("checked len >= 2 above");
    let (city, region, postal_code) = split_city_line(last)?;

    if !US_STATES.contains(&region.as_str()) {
        return Err(ParseError::InvalidState(region));
    }
    validate_zip(&postal_code)?;

    let (recipient, street, unit) = match rest.len() {
        1 => (None, rest[0].to_string(), None),
        2 => (None, rest[0].to_string(), Some(rest[1].to_string())),
        _ => (
            Some(rest[0].to_string()),
            rest[1].to_string(),
            Some(rest[2..].join(", ")),
        ),
    };

    Ok(Address {
        recipient,
        street,
        unit,
        city,
        region,
        postal_code,
        country: Country::Us,
    })
}

fn split_city_line(line: &str) -> Result<(String, String, String), ParseError> {
    let (city_part, tail) = line
        .rsplit_once(',')
        .ok_or_else(|| ParseError::InvalidCityLine(line.to_string()))?;

    let mut fields = tail.trim().split_whitespace();
    let region = fields
        .next()
        .ok_or_else(|| ParseError::InvalidCityLine(line.to_string()))?
        .to_uppercase();
    let postal_code = fields
        .next()
        .ok_or_else(|| ParseError::InvalidCityLine(line.to_string()))?
        .to_string();
    if fields.next().is_some() {
        return Err(ParseError::InvalidCityLine(line.to_string()));
    }

    Ok((city_part.trim().to_string(), region, postal_code))
}

fn validate_zip(zip: &str) -> Result<(), ParseError> {
    let all_digits = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit());

    let valid = match zip.len() {
        5 => all_digits(zip),
        10 => {
            let (first, second) = zip.split_at(5);
            second.starts_with('-') && all_digits(first) && all_digits(&second[1..])
        }
        _ => false,
    };

    if valid {
        Ok(())
    } else {
        Err(ParseError::InvalidPostalCode(zip.to_string()))
    }
}
