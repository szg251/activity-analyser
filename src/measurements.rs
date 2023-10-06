use derive_more::{Add, Sub, Sum};
use fitparser::{Error, Value};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct Power(pub i64);

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

#[derive(Debug, Clone, Copy, PartialEq, Add, Sub, Sum)]
pub struct Work(pub i64);

impl TryFrom<Value> for Work {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Error> {
        Ok(Self(value.try_into()?))
    }
}

impl From<Power> for Work {
    fn from(value: Power) -> Work {
        Work(value.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HeartRate(pub i64);

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cadence(pub i64);

impl TryFrom<Value> for Cadence {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Error> {
        Ok(Self(value.try_into()?))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Speed(pub f64);

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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Altitude(pub f64);

impl TryFrom<Value> for Altitude {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Error> {
        Ok(Self(value.try_into()?))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Sub, Add, Sum)]
pub struct AltitudeDiff(pub f64);

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PositionLat(pub i64);

impl TryFrom<Value> for PositionLat {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Error> {
        Ok(Self(value.try_into()?))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PositionLong(pub i64);

impl TryFrom<Value> for PositionLong {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Error> {
        Ok(Self(value.try_into()?))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position(pub PositionLat, pub PositionLong);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Weight(pub f64);

impl TryFrom<Value> for Weight {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Error> {
        Ok(Self(value.try_into()?))
    }
}
