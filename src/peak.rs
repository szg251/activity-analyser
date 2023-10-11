use crate::measurements::Average;
use chrono::{DateTime, Duration, Local};
use std::cmp::Ordering;
use std::convert::identity;

/// Peak of a given metric for a given amount of seconds
#[derive(Debug)]
pub struct Peak<T> {
    pub value: T,
    pub timestamps: TimeInterval,
    pub duration: Duration,
}

impl<T> Ord for Peak<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl<T> PartialOrd for Peak<T>
where
    T: PartialOrd + Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> PartialEq for Peak<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> Eq for Peak<T> where T: Eq {}

type TimeInterval = (DateTime<Local>, DateTime<Local>);

impl<T> Peak<T>
where
    T: Ord + Average + Copy,
{
    /// Find a peak performance of a given measurement of n seconds
    pub fn from_measurement_records(
        measurements: &Vec<(T, &DateTime<Local>)>,
        duration: Duration,
    ) -> Option<Self> {
        let windows = measurements.windows(duration.num_seconds() as usize);
        windows
            .map(|window| get_peak(window, duration))
            .filter_map(identity)
            .max()
    }
}

fn get_peak<T>(measurements: &[(T, &DateTime<Local>)], duration: Duration) -> Option<Peak<T>>
where
    T: Average + Copy,
{
    let avg = Average::average(measurements.iter().map(|(t, _)| *t).collect::<Vec<T>>())?;
    let start_time = measurements[0].1;
    let end_time = measurements[measurements.len() - 1].1;

    Some(Peak {
        value: avg,
        timestamps: (*start_time, *end_time),
        duration,
    })
}
