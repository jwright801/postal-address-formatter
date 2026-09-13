use crate::json;
use std::fmt;

/// Only US addresses are understood right now. This is its own type,
/// rather than a bare string, so the day a second country lands the
/// compiler finds every place that assumed there was only one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Country {
    Us,
}

impl Country {
    pub fn code(&self) -> &'static str {
        match self {
            Country::Us => "US",
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
    pub region: String,
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
        lines.push(format!("{}, {} {}", self.city, self.region, self.postal_code));
        lines.join("\n")
    }

    pub fn to_json(&self) -> String {
        format!(
            "{{\"recipient\":{},\"street\":{},\"unit\":{},\"city\":{},\"region\":{},\"postal_code\":{},\"country\":{}}}",
            json::nullable_string(&self.recipient),
            json::string(&self.street),
            json::nullable_string(&self.unit),
            json::string(&self.city),
            json::string(&self.region),
            json::string(&self.postal_code),
            json::string(self.country.code()),
        )
    }
}
