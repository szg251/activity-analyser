use derive_more::{Add, AddAssign, Display};

/// Training Stress Score
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Add, AddAssign, Debug, Display)]
pub struct TSS(pub i64);

/// Chronic Training Load
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Display)]
pub struct CTL(pub f64);

/// Acute Training Load
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Display)]
pub struct ATL(pub f64);

/// Training Stress Balance
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Display)]
pub struct TSB(pub f64);
