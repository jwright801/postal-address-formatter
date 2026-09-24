use crate::json;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Country {
    Us,
    Ca,
    Gb,
}

impl Country {
    pub fn code(&self) -> &'static str {
        match self {
            Country::Us => "US",
            Country::Ca => "CA",
            Country::Gb => "GB",
        }
    }
}

impl fmt::Display for Country {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Address {
    pub recipient: Option<String>,
    pub street: String,
    pub unit: Option<String>,
    pub city: String,
    /// The state or province code. UK addresses have no equivalent field -
    /// the post town on the last line stands alone next to the postcode.
    pub region: Option<String>,
    pub postal_code: String,
    pub country: Country,
}

impl Address {
    /// Reconstructs the canonical on-envelope layout from the parsed
    /// fields, rather than echoing back whatever whitespace the input had.
    pub fn pretty_print(&self) -> String {
        let mut lines = Vec::new();
        if let Some(recipient) = &self.recipient {
            lines.push(recipient.clone());
        }
        lines.push(self.street.clone());
        if let Some(unit) = &self.unit {
            lines.push(unit.clone());
        }
        lines.push(match &self.region {
            Some(region) => format!("{}, {} {}", self.city, region, self.postal_code),
            None => format!("{} {}", self.city, self.postal_code),
        });
        lines.join("\n")
    }

    pub fn to_json(&self) -> String {
        format!(
            "{{\"recipient\":{},\"street\":{},\"unit\":{},\"city\":{},\"region\":{},\"postal_code\":{},\"country\":{}}}",
            json::nullable_string(&self.recipient),
            json::string(&self.street),
            json::nullable_string(&self.unit),
            json::string(&self.city),
            json::nullable_string(&self.region),
            json::string(&self.postal_code),
            json::string(self.country.code()),
        )
    }
}
