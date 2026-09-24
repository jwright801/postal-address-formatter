use crate::address::{Address, Country};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    TooFewLines,
    InvalidCityLine(String),
    InvalidUkLine(String),
    UnrecognizedRegion(String),
    InvalidUsZip(String),
    InvalidCaPostalCode(String),
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
            ParseError::InvalidUkLine(line) => {
                write!(f, "could not read '{line}' as 'Post Town POSTCODE'")
            }
            ParseError::UnrecognizedRegion(region) => write!(
                f,
                "'{region}' is not a recognized US state or Canadian province code"
            ),
            ParseError::InvalidUsZip(zip) => {
                write!(f, "'{zip}' is not a valid US ZIP code")
            }
            ParseError::InvalidCaPostalCode(code) => {
                write!(f, "'{code}' is not a valid Canadian postal code")
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

const CA_PROVINCES: &[&str] = &[
    "AB", "BC", "MB", "NB", "NL", "NS", "NT", "NU", "ON", "PE", "QC", "SK", "YT",
];

/// Letters Canada Post never uses as the first character of a postal code.
const CA_POSTAL_EXCLUDED_FIRST: &[char] = &['D', 'F', 'I', 'O', 'Q', 'U'];

/// Parses a free-form US, Canadian, or UK address: an optional recipient
/// line, one street line, an optional unit line, and a trailing line that
/// is either "City, Region Postal" (US/CA) or "Post Town Postcode" (UK,
/// no comma). Which shape the last line has decides which country's
/// postal code and formatting rules apply.
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

    let (country, city, region, postal_code) = if last.contains(',') {
        let (city, region, postal_raw) = split_city_line(last)?;
        if US_STATES.contains(&region.as_str()) {
            (Country::Us, city, Some(region), validate_us_zip(&postal_raw)?)
        } else if CA_PROVINCES.contains(&region.as_str()) {
            (Country::Ca, city, Some(region), validate_ca_postal_code(&postal_raw)?)
        } else {
            return Err(ParseError::UnrecognizedRegion(region));
        }
    } else {
        let (post_town, postal_code) = parse_uk_line(last)?;
        (Country::Gb, post_town, None, postal_code)
    };

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
        country,
    })
}

/// Splits "City, Region Postal..." into its parts. The postal piece is
/// returned as the raw, space-joined remainder of the line because a US
/// ZIP is one token but a Canadian postal code is written as two.
fn split_city_line(line: &str) -> Result<(String, String, String), ParseError> {
    let (city_part, tail) = line
        .rsplit_once(',')
        .ok_or_else(|| ParseError::InvalidCityLine(line.to_string()))?;

    let mut fields = tail.trim().split_whitespace();
    let region = fields
        .next()
        .ok_or_else(|| ParseError::InvalidCityLine(line.to_string()))?
        .to_uppercase();
    let postal_parts: Vec<&str> = fields.collect();
    if postal_parts.is_empty() {
        return Err(ParseError::InvalidCityLine(line.to_string()));
    }

    Ok((city_part.trim().to_string(), region, postal_parts.join(" ")))
}

fn validate_us_zip(zip: &str) -> Result<String, ParseError> {
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
        Ok(zip.to_string())
    } else {
        Err(ParseError::InvalidUsZip(zip.to_string()))
    }
}

/// Validates and canonicalizes a Canadian postal code (format `A1A 1A1`),
/// accepting it with or without the space and in either letter case.
fn validate_ca_postal_code(raw: &str) -> Result<String, ParseError> {
    let compact: Vec<char> = raw
        .chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_uppercase)
        .collect();

    let invalid = || ParseError::InvalidCaPostalCode(raw.to_string());

    if compact.len() != 6 {
        return Err(invalid());
    }
    let is_letter = |c: char| c.is_ascii_alphabetic();
    let is_digit = |c: char| c.is_ascii_digit();
    let pattern_ok = is_letter(compact[0])
        && is_digit(compact[1])
        && is_letter(compact[2])
        && is_digit(compact[3])
        && is_letter(compact[4])
        && is_digit(compact[5]);
    if !pattern_ok || CA_POSTAL_EXCLUDED_FIRST.contains(&compact[0]) {
        return Err(invalid());
    }

    Ok(format!(
        "{}{}{} {}{}{}",
        compact[0], compact[1], compact[2], compact[3], compact[4], compact[5]
    ))
}

/// Splits a UK last line into post town and postcode. Unlike the US/CA
/// line there's no comma to anchor on, so the postcode is found by
/// testing the last one or two whitespace-separated tokens (people write
/// it both as "SW1A 1AA" and "SW1A1AA") and treating whatever remains as
/// the post town.
fn parse_uk_line(line: &str) -> Result<(String, String), ParseError> {
    let tokens: Vec<&str> = line.split_whitespace().collect();

    if tokens.len() >= 3 {
        let candidate = format!("{}{}", tokens[tokens.len() - 2], tokens[tokens.len() - 1]);
        if let Some(postcode) = validate_uk_postcode(&candidate) {
            return Ok((tokens[..tokens.len() - 2].join(" "), postcode));
        }
    }
    if tokens.len() >= 2 {
        if let Some(postcode) = validate_uk_postcode(tokens[tokens.len() - 1]) {
            return Ok((tokens[..tokens.len() - 1].join(" "), postcode));
        }
    }

    Err(ParseError::InvalidUkLine(line.to_string()))
}

/// Validates a UK postcode and normalizes it to "OUTWARD INWARD" (e.g.
/// `SW1A 1AA`), accepting it with or without the space and in either
/// letter case. This checks the shape Royal Mail documents (outward code
/// of 2-4 characters, inward code always digit+letter+letter), plus the
/// long-standing special case for Girobank's `GIR 0AA`. It does not
/// check the outward code against the real list of assigned postal
/// areas.
fn validate_uk_postcode(raw: &str) -> Option<String> {
    let compact: Vec<char> = raw
        .chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_uppercase)
        .collect();

    if compact == ['G', 'I', 'R', '0', 'A', 'A'] {
        return Some("GIR 0AA".to_string());
    }

    if !(5..=7).contains(&compact.len()) {
        return None;
    }
    let (outward, inward) = compact.split_at(compact.len() - 3);

    let is_letter = |c: char| c.is_ascii_alphabetic();
    let is_digit = |c: char| c.is_ascii_digit();

    let inward_ok = is_digit(inward[0]) && is_letter(inward[1]) && is_letter(inward[2]);
    let outward_ok = match outward.len() {
        2 => is_letter(outward[0]) && is_digit(outward[1]),
        3 => {
            is_letter(outward[0])
                && ((is_digit(outward[1]) && is_digit(outward[2]))
                    || (is_letter(outward[1]) && is_digit(outward[2]))
                    || (is_digit(outward[1]) && is_letter(outward[2])))
        }
        4 => {
            is_letter(outward[0])
                && is_letter(outward[1])
                && is_digit(outward[2])
                && (is_digit(outward[3]) || is_letter(outward[3]))
        }
        _ => false,
    };
    if !inward_ok || !outward_ok {
        return None;
    }

    Some(format!(
        "{} {}",
        outward.iter().collect::<String>(),
        inward.iter().collect::<String>()
    ))
}
