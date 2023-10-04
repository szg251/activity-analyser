use crate::metrics::{HeartRate, Power, Weight};
use chrono::NaiveDate;

pub struct Measurements(Vec<(NaiveDate, Measurement)>);

impl Measurements {
    // Create a new sorted list of measurements
    pub fn new<T>(mut measurements: T) -> Self
    where
        T: AsMut<[(NaiveDate, Measurement)]>,
    {
        let measurements = measurements.as_mut();
        measurements.sort_by(|(a, _), (b, _)| a.cmp(b));
        Self(measurements.to_vec())
    }

    pub fn get_actual_ftp(self: &Self, date: &NaiveDate) -> Option<Power> {
        self.get_actual(date, Measurement::get_ftp)
    }

    pub fn get_actual_fthr(self: &Self, date: &NaiveDate) -> Option<HeartRate> {
        self.get_actual(date, Measurement::get_fthr)
    }

    fn get_actual<T, F>(self: &Self, date: &NaiveDate, getter: F) -> Option<T>
    where
        F: Fn(&Measurement) -> Option<T>,
    {
        let Measurements(measurements) = self;
        let m = measurements
            .iter()
            .filter_map(|(d, m)| Some((*d, getter(m)?)))
            .take_while(|(d, _)| d <= date)
            .last()?;
        Some(m.1)
    }
}

#[derive(Clone)]
pub enum Measurement {
    FTP(Power),
    FTHr(HeartRate),
    Weight(Weight),
}

impl Measurement {
    pub fn get_ftp(self: &Self) -> Option<Power> {
        match self {
            Self::FTP(power) => Some(*power),
            _ => None,
        }
    }

    pub fn get_fthr(self: &Self) -> Option<HeartRate> {
        match self {
            Self::FTHr(hr) => Some(*hr),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn find_ftp() {
        let measurements = Measurements::new([
            (
                NaiveDate::from_ymd_opt(2022, 7, 8).unwrap(),
                Measurement::FTP(Power(200)),
            ),
            (
                NaiveDate::from_ymd_opt(2022, 8, 8).unwrap(),
                Measurement::FTP(Power(210)),
            ),
            (
                NaiveDate::from_ymd_opt(2022, 9, 8).unwrap(),
                Measurement::FTP(Power(220)),
            ),
        ]);
        assert_eq!(
            measurements.get_actual_ftp(&NaiveDate::from_ymd_opt(2022, 9, 1).unwrap()),
            Some(Power(210))
        );
    }
}
