use derive_more::{Add, AddAssign, Display};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Add, AddAssign, Debug, Display)]
pub struct TSS(pub i64);

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Display)]
pub struct CTL(pub f64);

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Display)]
pub struct ATL(pub f64);

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Display)]
pub struct TSB(pub f64);
