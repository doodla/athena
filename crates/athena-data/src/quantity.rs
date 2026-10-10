//! Quantities that always carry their provenance.
//!
//! Every number Athena records for tokens, time, or money is a [`Quantity`]. A bare number cannot be
//! recorded: the type has no constructor without a provenance and a basis, and deserialization refuses a
//! quantity whose fields contradict its provenance (design §5.1, D16).

use serde::{Deserialize, Serialize};
use std::fmt;

/// Where a number came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    /// Observed directly: counted by an API, or timed by Athena's own clock.
    Measured,
    /// Reported by a tool that computed it, such as a cost at list price.
    Reported,
    /// A judgment or a bound, never an observation.
    Estimated,
    /// Not known yet; a later resolution record supplies it.
    Pending,
}

/// The unit of a quantity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    Usd,
    Tokens,
    Seconds,
    Minutes,
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Unit::Usd => "usd",
            Unit::Tokens => "tokens",
            Unit::Seconds => "s",
            Unit::Minutes => "min",
        };
        f.write_str(name)
    }
}

/// A number with its unit, provenance, and the basis for it, plus optional bounds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RawQuantity", into = "RawQuantity")]
pub struct Quantity {
    value: Option<f64>,
    unit: Unit,
    provenance: Provenance,
    basis: String,
    lower: Option<f64>,
    upper: Option<f64>,
}

/// Why a quantity's fields are inconsistent.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum QuantityError {
    #[error("a quantity needs a basis that says where it came from")]
    MissingBasis,
    #[error("a {0:?} quantity needs a value")]
    MissingValue(Provenance),
    #[error("a pending quantity has no value yet")]
    PendingWithValue,
    #[error("an estimated quantity needs a value or at least one bound")]
    EstimateWithoutAnything,
    #[error("quantity values must be finite and not negative")]
    NotFiniteOrNegative,
    #[error("bounds are out of order")]
    BoundsOutOfOrder,
}

impl Quantity {
    fn build(
        value: Option<f64>,
        unit: Unit,
        provenance: Provenance,
        basis: impl Into<String>,
        lower: Option<f64>,
        upper: Option<f64>,
    ) -> Result<Self, QuantityError> {
        let quantity = Quantity {
            value,
            unit,
            provenance,
            basis: basis.into(),
            lower,
            upper,
        };
        quantity.validate()?;
        Ok(quantity)
    }

    /// A value observed directly. Panics only on a programming error (a negative or non-finite value).
    pub fn measured(value: f64, unit: Unit, basis: impl Into<String>) -> Self {
        Self::build(Some(value), unit, Provenance::Measured, basis, None, None)
            .expect("measured quantities are finite and not negative")
    }

    /// A value a tool reported, with the tool and method named in `basis`.
    pub fn reported(value: f64, unit: Unit, basis: impl Into<String>) -> Self {
        Self::build(Some(value), unit, Provenance::Reported, basis, None, None)
            .expect("reported quantities are finite and not negative")
    }

    /// An estimate with optional bounds.
    pub fn estimated(
        value: Option<f64>,
        unit: Unit,
        basis: impl Into<String>,
        lower: Option<f64>,
        upper: Option<f64>,
    ) -> Result<Self, QuantityError> {
        Self::build(value, unit, Provenance::Estimated, basis, lower, upper)
    }

    /// A value not known yet.
    pub fn pending(unit: Unit, basis: impl Into<String>) -> Self {
        Self::build(None, unit, Provenance::Pending, basis, None, None)
            .expect("a pending quantity with a basis is always valid")
    }

    fn validate(&self) -> Result<(), QuantityError> {
        if self.basis.trim().is_empty() {
            return Err(QuantityError::MissingBasis);
        }
        for number in [self.value, self.lower, self.upper].into_iter().flatten() {
            if !number.is_finite() || number < 0.0 {
                return Err(QuantityError::NotFiniteOrNegative);
            }
        }
        match self.provenance {
            Provenance::Measured | Provenance::Reported if self.value.is_none() => {
                return Err(QuantityError::MissingValue(self.provenance));
            }
            Provenance::Pending if self.value.is_some() => {
                return Err(QuantityError::PendingWithValue);
            }
            Provenance::Estimated
                if self.value.is_none() && self.lower.is_none() && self.upper.is_none() =>
            {
                return Err(QuantityError::EstimateWithoutAnything);
            }
            _ => {}
        }
        let ordered = |a: Option<f64>, b: Option<f64>| match (a, b) {
            (Some(a), Some(b)) => a <= b,
            _ => true,
        };
        if !(ordered(self.lower, self.value)
            && ordered(self.value, self.upper)
            && ordered(self.lower, self.upper))
        {
            return Err(QuantityError::BoundsOutOfOrder);
        }
        Ok(())
    }

    pub fn value(&self) -> Option<f64> {
        self.value
    }
    pub fn unit(&self) -> Unit {
        self.unit
    }
    pub fn provenance(&self) -> Provenance {
        self.provenance
    }
    pub fn basis(&self) -> &str {
        &self.basis
    }
    pub fn lower(&self) -> Option<f64> {
        self.lower
    }
    pub fn upper(&self) -> Option<f64> {
        self.upper
    }

    /// The best single number for sums: the value, else the lower bound, else zero.
    pub fn best_lower(&self) -> f64 {
        self.value.or(self.lower).unwrap_or(0.0)
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.value, self.lower, self.upper) {
            (Some(v), _, _) => write!(f, "{v} {} ({:?})", self.unit, self.provenance),
            (None, lower, upper) => write!(
                f,
                "[{}, {}] {} ({:?})",
                lower.map_or("?".to_string(), |v| v.to_string()),
                upper.map_or("?".to_string(), |v| v.to_string()),
                self.unit,
                self.provenance
            ),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawQuantity {
    value: Option<f64>,
    unit: Unit,
    provenance: Provenance,
    basis: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    lower: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    upper: Option<f64>,
}

impl TryFrom<RawQuantity> for Quantity {
    type Error = QuantityError;
    fn try_from(raw: RawQuantity) -> Result<Self, Self::Error> {
        Quantity::build(
            raw.value,
            raw.unit,
            raw.provenance,
            raw.basis,
            raw.lower,
            raw.upper,
        )
    }
}

impl From<Quantity> for RawQuantity {
    fn from(q: Quantity) -> Self {
        RawQuantity {
            value: q.value,
            unit: q.unit,
            provenance: q.provenance,
            basis: q.basis,
            lower: q.lower,
            upper: q.upper,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measured_quantity_round_trips() {
        let q = Quantity::measured(41.0, Unit::Tokens, "API usage");
        let json = serde_json::to_string(&q).unwrap();
        let back: Quantity = serde_json::from_str(&json).unwrap();
        assert_eq!(q, back);
    }

    #[test]
    fn bare_number_is_refused() {
        // M14 (blocks): a record field that is a bare number does not parse as a quantity.
        assert!(serde_json::from_str::<Quantity>("0.41").is_err());
    }

    #[test]
    fn measured_without_value_is_refused() {
        let raw = r#"{"value":null,"unit":"usd","provenance":"measured","basis":"x"}"#;
        assert!(serde_json::from_str::<Quantity>(raw).is_err());
    }

    #[test]
    fn missing_basis_is_refused() {
        let raw = r#"{"value":1.0,"unit":"usd","provenance":"reported","basis":"  "}"#;
        assert!(serde_json::from_str::<Quantity>(raw).is_err());
    }

    #[test]
    fn pending_has_no_value() {
        let raw = r#"{"value":3.0,"unit":"usd","provenance":"pending","basis":"later"}"#;
        assert!(serde_json::from_str::<Quantity>(raw).is_err());
        let ok = r#"{"value":null,"unit":"usd","provenance":"pending","basis":"later"}"#;
        assert!(serde_json::from_str::<Quantity>(ok).is_ok());
    }

    #[test]
    fn estimate_may_be_bounds_only() {
        let q = Quantity::estimated(None, Unit::Usd, "killed; cap", Some(0.0), Some(3.0)).unwrap();
        assert_eq!(q.best_lower(), 0.0);
        assert!(Quantity::estimated(None, Unit::Usd, "nothing", None, None).is_err());
    }

    #[test]
    fn bounds_must_be_ordered() {
        assert!(Quantity::estimated(Some(5.0), Unit::Usd, "x", Some(6.0), None).is_err());
        assert!(Quantity::estimated(Some(5.0), Unit::Usd, "x", Some(1.0), Some(9.0)).is_ok());
    }

    #[test]
    fn unknown_fields_are_refused() {
        let raw = r#"{"value":1.0,"unit":"usd","provenance":"reported","basis":"x","extra":1}"#;
        assert!(serde_json::from_str::<Quantity>(raw).is_err());
    }
}
