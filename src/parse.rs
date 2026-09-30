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

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(input: &str) -> Address {
        parse(input).unwrap_or_else(|e| panic!("expected {input:?} to parse, got: {e}"))
    }

    fn err(input: &str) -> ParseError {
        parse(input).expect_err("expected a parse error")
    }

    #[test]
    fn us_address_with_all_lines() {
        let a = ok("Jane Doe\n500 Market St\nSuite 210\nSan Francisco, CA 94105\n");
        assert_eq!(a.recipient.as_deref(), Some("Jane Doe"));
        assert_eq!(a.street, "500 Market St");
        assert_eq!(a.unit.as_deref(), Some("Suite 210"));
        assert_eq!(a.city, "San Francisco");
        assert_eq!(a.region.as_deref(), Some("CA"));
        assert_eq!(a.postal_code, "94105");
        assert_eq!(a.country, Country::Us);
    }

    #[test]
    fn street_and_city_line_only() {
        let a = ok("500 Market St\nSan Francisco, CA 94105");
        assert_eq!(a.recipient, None);
        assert_eq!(a.unit, None);
        assert_eq!(a.street, "500 Market St");
    }

    #[test]
    fn two_lines_before_city_are_street_and_unit() {
        let a = ok("500 Market St\nPO Box 12\nSan Francisco, CA 94105");
        assert_eq!(a.recipient, None);
        assert_eq!(a.street, "500 Market St");
        assert_eq!(a.unit.as_deref(), Some("PO Box 12"));
    }

    #[test]
    fn extra_lines_are_folded_into_unit() {
        let a = ok("Jane Doe\n500 Congress Ave\nFloor 3\nBuilding B\nAustin, TX 78701");
        assert_eq!(a.recipient.as_deref(), Some("Jane Doe"));
        assert_eq!(a.street, "500 Congress Ave");
        assert_eq!(a.unit.as_deref(), Some("Floor 3, Building B"));
    }

    #[test]
    fn blank_lines_and_padding_are_ignored() {
        let a = ok("\n  500 Market St  \n\n  San Francisco,  CA   94105  \n\n");
        assert_eq!(a.street, "500 Market St");
        assert_eq!(a.city, "San Francisco");
        assert_eq!(a.postal_code, "94105");
    }

    #[test]
    fn too_few_lines() {
        assert_eq!(err(""), ParseError::TooFewLines);
        assert_eq!(err("\n  \n"), ParseError::TooFewLines);
        assert_eq!(err("San Francisco, CA 94105"), ParseError::TooFewLines);
    }

    #[test]
    fn region_is_uppercased() {
        assert_eq!(ok("1 Main St\nDenver, co 80202").region.as_deref(), Some("CO"));
    }

    #[test]
    fn dc_is_a_valid_region() {
        let a = ok("1600 Pennsylvania Ave NW\nWashington, DC 20500");
        assert_eq!(a.country, Country::Us);
    }

    #[test]
    fn city_containing_a_comma_keeps_it() {
        let a = ok("1 Main St\nWinston, Salem, NC 27101");
        assert_eq!(a.city, "Winston, Salem");
        assert_eq!(a.region.as_deref(), Some("NC"));
    }

    #[test]
    fn zip_plus_four_is_accepted() {
        assert_eq!(ok("1 Main St\nDenver, CO 80202-1234").postal_code, "80202-1234");
    }

    #[test]
    fn bad_zips_are_rejected() {
        for zip in ["9410", "941055", "9410a", "94105-12", "94105 1234", "94105_1234"] {
            let input = format!("1 Main St\nDenver, CO {zip}");
            assert!(
                matches!(err(&input), ParseError::InvalidUsZip(_)),
                "{zip} should be rejected as a ZIP"
            );
        }
    }

    #[test]
    fn unknown_region_is_reported_uppercased() {
        assert_eq!(
            err("1 Main St\nDenver, zz 80202"),
            ParseError::UnrecognizedRegion("ZZ".to_string())
        );
    }

    #[test]
    fn city_line_missing_pieces() {
        assert!(matches!(
            err("1 Main St\nDenver, CO"),
            ParseError::InvalidCityLine(_)
        ));
        assert!(matches!(
            err("1 Main St\nDenver,"),
            ParseError::InvalidCityLine(_)
        ));
    }

    #[test]
    fn canadian_postal_code_is_normalized() {
        for raw in ["M5H 2N2", "m5h2n2", "M5H  2n2"] {
            let a = ok(&format!("100 Queen St W\nToronto, ON {raw}"));
            assert_eq!(a.country, Country::Ca);
            assert_eq!(a.postal_code, "M5H 2N2");
        }
    }

    #[test]
    fn bad_canadian_postal_codes_are_rejected() {
        // D is never a valid first letter; the rest are wrong shape or length.
        for code in ["D5H 2N2", "M5H 2N", "5MH 2N2", "M5H 2N22", "MMH 2N2"] {
            let input = format!("100 Queen St W\nToronto, ON {code}");
            assert_eq!(
                err(&input),
                ParseError::InvalidCaPostalCode(code.to_string())
            );
        }
    }

    #[test]
    fn us_zip_is_not_accepted_for_a_province() {
        assert!(matches!(
            err("100 Queen St W\nToronto, ON 94105"),
            ParseError::InvalidCaPostalCode(_)
        ));
    }

    #[test]
    fn uk_address_splits_town_and_postcode() {
        let a = ok("221B Baker Street\nLondon NW1 6XE");
        assert_eq!(a.country, Country::Gb);
        assert_eq!(a.city, "London");
        assert_eq!(a.region, None);
        assert_eq!(a.postal_code, "NW1 6XE");
    }

    #[test]
    fn uk_postcode_without_space_or_in_lowercase() {
        assert_eq!(ok("1 High St\nLondon nw16xe").postal_code, "NW1 6XE");
        assert_eq!(ok("1 High St\nLondon SW1A1AA").postal_code, "SW1A 1AA");
    }

    #[test]
    fn uk_multi_word_post_town() {
        let a = ok("1 High St\nStoke on Trent ST4 1AA");
        assert_eq!(a.city, "Stoke on Trent");
        assert_eq!(a.postal_code, "ST4 1AA");
    }

    #[test]
    fn uk_outward_code_shapes() {
        for (raw, expected) in [
            ("M1 1AE", "M1 1AE"),
            ("B33 8TH", "B33 8TH"),
            ("CR2 6XH", "CR2 6XH"),
            ("DN55 1PT", "DN55 1PT"),
            ("W1A 0AX", "W1A 0AX"),
            ("EC1A 1BB", "EC1A 1BB"),
        ] {
            assert_eq!(ok(&format!("1 High St\nTown {raw}")).postal_code, expected);
        }
    }

    #[test]
    fn girobank_postcode_is_special_cased() {
        assert_eq!(ok("1 High St\nBootle GIR 0AA").postal_code, "GIR 0AA");
    }

    #[test]
    fn bad_uk_last_lines_are_rejected() {
        for line in ["London", "London NW1", "London NW1 6X", "London 12345", "NW1 6XE 7"] {
            let input = format!("1 High St\n{line}");
            assert_eq!(err(&input), ParseError::InvalidUkLine(line.to_string()));
        }
    }

    #[test]
    fn uk_postcode_after_a_comma_is_not_a_region() {
        assert_eq!(
            err("1 High St\nLondon, NW1 6XE"),
            ParseError::UnrecognizedRegion("NW1".to_string())
        );
    }

    #[test]
    fn pretty_print_round_trips() {
        for input in [
            "Jane Doe\n500 Market St\nSuite 210\nSan Francisco, CA 94105",
            "100 Queen St W\nToronto, ON M5H 2N2",
            "221B Baker Street\nLondon NW1 6XE",
        ] {
            assert_eq!(ok(input).pretty_print(), input);
        }
    }

    #[test]
    fn error_messages_name_the_offending_input() {
        let msg = err("1 Main St\nDenver, CO 1234").to_string();
        assert!(msg.contains("'1234'"), "unexpected message: {msg}");
    }
}
