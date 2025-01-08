use derive_more::{Add, Sub, Sum};
use fitparser::{Error, Value};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

/// A vector-like collection that can be averaged
pub trait Average<A = Self>: Sized {
    fn average<I>(elems: I) -> Option<Self>
    where
        I: AsRef<[A]>;
}

impl Average for i64 {
    fn average<I>(elems: I) -> Option<Self>
    where
        I: AsRef<[i64]>,
    {
        let elems = elems.as_ref();

        if !elems.is_empty() {
            Some(elems.iter().sum::<i64>() / (elems.len() as i64))
        } else {
            None
        }
    }
}

/// Power data in Watts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Power(pub i64);

impl Display for Power {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{} W", self.0)
    }
}

impl TryFrom<Value> for Power {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Error> {
        Ok(Self(value.try_into()?))
    }
}

impl Average for Power {
    fn average<I>(elems: I) -> Option<Self>
    where
        I: AsRef<[Self]>,
    {
        let elems = elems.as_ref();
        if !elems.is_empty() {
            let avg = elems.iter().map(|Self(inner)| inner).sum::<i64>() / (elems.len() as i64);
            Some(Self(avg))
        } else {
            None
        }
    }
}

/// Work data in kJ
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Add, Sub, Sum)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Work(pub f64);

impl Display for Work {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:.2} kJ", self.0)
    }
}

impl TryFrom<Value> for Work {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Error> {
        Ok(Self(value.try_into()?))
    }
}

impl From<Power> for Work {
    fn from(value: Power) -> Work {
        Work(value.0 as f64 / 1000.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HeartRate(pub i64);

/// Heart rate data in bpm
impl Display for HeartRate {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{} BPM", self.0)
    }
}

impl TryFrom<Value> for HeartRate {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Error> {
        Ok(Self(value.try_into()?))
    }
}

impl Average for HeartRate {
    fn average<I>(elems: I) -> Option<Self>
    where
        I: AsRef<[Self]>,
    {
        let elems = elems.as_ref();
        if !elems.is_empty() {
            let avg = elems.iter().map(|Self(inner)| inner).sum::<i64>() / (elems.len() as i64);
            Some(Self(avg))
        } else {
            None
        }
    }
}

/// Cadence data in rpm
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cadence(pub i64);

impl Display for Cadence {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{} RPM", self.0)
    }
}

impl TryFrom<Value> for Cadence {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Error> {
        Ok(Self(value.try_into()?))
    }
}

/// Speed data in m/s
/// Default display will convert it to km/h
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Speed(pub f64);

impl Display for Speed {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:.2} km/h", self.0 * 3.6)
    }
}

impl Eq for Speed {}

#[allow(clippy::derive_ord_xor_partial_ord)]
impl Ord for Speed {
    /// Boldly we claim that floats are always comparable.
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl TryFrom<Value> for Speed {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Error> {
        Ok(Self(value.try_into()?))
    }
}

impl Average for Speed {
    fn average<I>(elems: I) -> Option<Self>
    where
        I: AsRef<[Self]>,
    {
        let elems = elems.as_ref();
        if !elems.is_empty() {
            let avg = elems.iter().map(|Self(inner)| inner).sum::<f64>() / (elems.len() as f64);
            Some(Self(avg))
        } else {
            None
        }
    }
}

/// Altitude in meters
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Altitude(pub f64);

impl Display for Altitude {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{} m", self.0)
    }
}

impl TryFrom<Value> for Altitude {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Error> {
        Ok(Self(value.try_into()?))
    }
}

/// Altitude difference in meters
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Sub, Add, Sum)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AltitudeDiff(pub f64);

impl Display for AltitudeDiff {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{} m", self.0)
    }
}

impl TryFrom<Value> for AltitudeDiff {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Error> {
        Ok(Self(value.try_into()?))
    }
}

impl From<Altitude> for AltitudeDiff {
    fn from(value: Altitude) -> AltitudeDiff {
        AltitudeDiff(value.0)
    }
}

/// Weight data in kg
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Weight(pub f64);

impl Display for Weight {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{} kg", self.0)
    }
}

impl TryFrom<Value> for Weight {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Error> {
        Ok(Self(value.try_into()?))
    }
}
