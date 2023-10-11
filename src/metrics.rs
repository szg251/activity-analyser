use crate::measurements::{Altitude, AltitudeDiff, Average, HeartRate, Power, Work};
use chrono::{Duration, NaiveDate};
use derive_more::{Add, AddAssign, Display};
// use crate::activity::Activity;

/// Accumulated Training Stress Scores for a day
#[derive(Clone, Debug)]
pub struct DailyTSS(pub NaiveDate, pub TSS);

/// Training Stress Score
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Add, AddAssign, Debug, Display)]
pub struct TSS(pub i64);

impl TSS {
    /// Calculate user specific Training Stress Scores
    pub fn calculate(ftp: &Power, duration: &Duration, normalized_power: &Power) -> TSS {
        let IF(intensity_factor) = IF::calculate(ftp, normalized_power);
        let Power(ftp) = *ftp;
        let Power(normalized_power) = *normalized_power;
        let duration = duration.num_seconds() as f64;

        TSS(
            (((duration * (normalized_power as f64) * intensity_factor) / (ftp as f64 * 3_600.0))
                * 100.0) as i64,
        )
    }

    /// Calculate user specific Heart Rate Training Stress Score
    pub fn calculate_hr_tss(fthr: &HeartRate, heart_rate_data: &Vec<HeartRate>) -> TSS {
        let HeartRate(fthr) = fthr;
        let zones = (
            fthr * 73 / 100,
            fthr * 77 / 100,
            fthr * 81 / 100,
            fthr * 85 / 100,
            fthr * 89 / 100,
            fthr * 93 / 100,
            fthr,
            fthr * 103 / 100,
            fthr * 106 / 100,
        );

        let zones_count = heart_rate_data.iter().fold(
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            |mut acc, HeartRate(hr)| {
                if hr < &zones.0 {
                    acc.0 += 1;
                } else if hr < &zones.1 {
                    acc.1 += 1;
                } else if hr < &zones.2 {
                    acc.2 += 1;
                } else if hr < &zones.3 {
                    acc.3 += 1;
                } else if hr < &zones.4 {
                    acc.4 += 1;
                } else if hr < &zones.5 {
                    acc.5 += 1;
                } else if hr < &zones.6 {
                    acc.6 += 1;
                } else if hr < &zones.7 {
                    acc.7 += 1;
                } else if hr < &zones.8 {
                    acc.8 += 1;
                } else {
                    acc.9 += 1;
                };
                acc
            },
        );

        TSS((zones_count.0 * 20
            + zones_count.1 * 30
            + zones_count.2 * 40
            + zones_count.3 * 50
            + zones_count.4 * 60
            + zones_count.5 * 75
            + zones_count.6 * 100
            + zones_count.7 * 105
            + zones_count.8 * 110
            + zones_count.9 * 120)
            / 3600)
    }
}

/// Calculate training load with a given decay and impact constant
fn calc_training_load(
    decay_const: i64,
    impact_const: i64,
    yesterdays_tl: f64,
    daily_tss: &DailyTSS,
) -> f64 {
    let TSS(tss) = daily_tss.1;
    let decay_factor = (-1.0 / decay_const as f64).exp();
    let impact_factor = 1.0 - (-1.0 / impact_const as f64).exp();

    yesterdays_tl * decay_factor + tss as f64 * impact_factor
}

/// Chronic Training Load
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Display)]
pub struct CTL(pub f64);

impl CTL {
    /// Calculating Chronic Training Load (CTL), a 42 day average of daily TSS values
    pub fn calculate(Self(yesterdays_tl): &Self, daily_tss: &DailyTSS) -> Self {
        Self(calc_training_load(42, 42, *yesterdays_tl, daily_tss))
    }
}

/// Acute Training Load
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Display)]
pub struct ATL(pub f64);

impl ATL {
    /// Calculating Acute Training Load (ATL), a 7 day average of daily TSS values
    pub fn calculate(Self(yesterdays_tl): &Self, daily_tss: &DailyTSS) -> Self {
        Self(calc_training_load(7, 7, *yesterdays_tl, daily_tss))
    }
}

/// Training Stress Balance
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Display)]
pub struct TSB(pub f64);

impl TSB {
    pub fn calculate(CTL(ctl): &CTL, ATL(atl): &ATL) -> Self {
        Self(ctl - atl)
    }
}

/// Intensity Factor
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Display)]
pub struct IF(pub f64);

impl IF {
    /// Calculate Intensity Factor
    pub fn calculate(ftp: &Power, normalized_power: &Power) -> Self {
        let Power(ftp) = *ftp;
        let Power(normalized_power) = *normalized_power;

        Self(normalized_power as f64 / ftp as f64)
    }
}

/// Variability Index
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Display)]
pub struct VI(pub f64);

impl VI {
    /// Calculate Variablity Index
    pub fn calculate(normalized_power: &Power, average_power: &Power) -> Self {
        let Power(normalized_power) = *normalized_power;
        let Power(average_power) = *average_power;

        Self(normalized_power as f64 / average_power as f64)
    }
}

/// Calculate total work
pub fn calc_total_work(power_data: &Vec<Power>) -> Work {
    power_data.into_iter().map(|power| Work::from(*power)).sum()
}

/// Calculate Normalized Power
pub fn calc_normalized_power(power_data: &Vec<Power>) -> Option<Power> {
    // Returning simple average, if data size doesn't hit threshold
    if power_data.len() < 30 {
        return Average::average(power_data);
    }

    let avg: i64 = Average::average(
        rolling_averages(power_data, 30)
            .iter()
            .map(|Power(x)| x.pow(4))
            .collect::<Vec<i64>>(),
    )?;

    let result = (avg as f64).powf(0.25) as i64;
    Some(Power(result))
}

/// Calculate rolling averages of a set window size
pub fn rolling_averages<I, T>(data: T, size: usize) -> Vec<I>
where
    T: AsRef<[I]>,
    I: Average,
{
    data.as_ref()
        .windows(size)
        .map(|window| Average::average(window).unwrap())
        .collect()
}

/// Calculate altitude gain and altitude loss of an activity
pub fn calc_altitude_changes(
    altitude_data: &Vec<Altitude>,
) -> (Option<AltitudeDiff>, Option<AltitudeDiff>) {
    let init: (
        Option<AltitudeDiff>,
        Option<AltitudeDiff>,
        Option<&Altitude>,
    ) = (None, None, None);
    let (gain, loss, _) = altitude_data.iter().fold(
        init,
        |(acc_gain, acc_loss, prev_alt), next_alt| match prev_alt {
            None => (acc_gain, acc_loss, Some(next_alt)),
            Some(prev_alt) => {
                if prev_alt < next_alt {
                    let cur_gain =
                        <Altitude as Into<AltitudeDiff>>::into(*next_alt) - (*prev_alt).into();
                    match acc_gain {
                        None => (Some(cur_gain), acc_loss, Some(next_alt)),
                        Some(acc_gain) => (Some(acc_gain + cur_gain), acc_loss, Some(next_alt)),
                    }
                } else {
                    let cur_loss =
                        <Altitude as Into<AltitudeDiff>>::into(*prev_alt) - (*next_alt).into();
                    match acc_loss {
                        None => (acc_gain, Some(cur_loss), Some(next_alt)),
                        Some(acc_loss) => (acc_gain, Some(acc_loss + cur_loss), Some(next_alt)),
                    }
                }
            }
        },
    );

    (gain, loss)
}

#[cfg(test)]
mod activity_analysis_tests {
    use super::*;
    use crate::activity::Activity;
    use assertables::{assert_in_delta, assert_in_delta_as_result};
    use std::fs::File;

    #[test]
    /// Don't panic on small data (less than 30 seconds)
    fn small_data() {
        let power_data: Vec<Power> = vec![Power(200), Power(200), Power(200), Power(200)];

        assert_eq!(calc_normalized_power(&power_data), Some(Power(200)));
    }

    #[test]
    /// Constant effort NP should be equal to average power
    fn constant_effort_np() {
        // TODO: implement and test intermittent data
        // let power_data: Vec<(Power, DateTime<Local>)> = (0..3600)
        //     .map(|s| {
        //         (
        //             Power(200),
        //             "2012-12-12 12:12:12Z".parse::<DateTime<Local>>().unwrap()
        //                 + Duration::seconds(s),
        //         )
        //     })
        let power_data: Vec<Power> = (0..3600).map(|_| Power(200)).collect();

        assert_eq!(calc_normalized_power(&power_data), Some(Power(200)));
    }

    #[test]
    fn one_hour_effort_tss() {
        let tss = TSS::calculate(&Power(260), &Duration::hours(1), &Power(260));
        assert_eq!(tss, TSS(100))
    }

    #[test]
    fn ninety_minute_effort_tss() {
        let tss = TSS::calculate(&Power(260), &Duration::minutes(90), &Power(260));
        assert_eq!(tss, TSS(150))
    }

    #[test]
    fn four_hour_effort_tss() {
        let tss = TSS::calculate(&Power(260), &Duration::hours(4), &Power(130));
        assert_eq!(tss, TSS(100))
    }

    #[test]
    fn constant_effort_total_work() {
        let Work(work) = calc_total_work(&vec![Power(260); 100]);
        assert_in_delta!(work, 26.0, 0.001);
    }

    // Golden tests

    #[test]
    fn activity_file_work() {
        let mut fp = File::open("./tests/fixtures/Activity.fit").unwrap();
        let activity = Activity::from_reader(&mut fp).unwrap();

        let Work(work) = calc_total_work(&activity.get_data("power"));
        assert_in_delta!(work, 719.35, 0.001);
    }

    #[test]
    fn activity_file_average_power() {
        let mut fp = File::open("./tests/fixtures/Activity.fit").unwrap();
        let activity = Activity::from_reader(&mut fp).unwrap();

        let Power(power) = Average::average(&activity.get_data("power")).unwrap();
        assert_eq!(power, 199);
    }

    #[test]
    fn activity_file_normalized_power() {
        let mut fp = File::open("./tests/fixtures/Activity.fit").unwrap();
        let activity = Activity::from_reader(&mut fp).unwrap();

        let Power(power) = calc_normalized_power(&activity.get_data("power")).unwrap();
        assert_eq!(power, 214);
    }

    #[test]
    fn activity_file_intensity_factor() {
        let mut fp = File::open("./tests/fixtures/Activity.fit").unwrap();
        let activity = Activity::from_reader(&mut fp).unwrap();
        let ftp = Power(260);
        let np = calc_normalized_power(&activity.get_data("power")).unwrap();

        let IF(intensity_factor) = IF::calculate(&ftp, &np);

        assert_in_delta!(intensity_factor, 0.82, 0.005);
    }

    #[test]
    fn activity_file_variability_index() {
        let mut fp = File::open("./tests/fixtures/Activity.fit").unwrap();
        let activity = Activity::from_reader(&mut fp).unwrap();
        let avg_power = Average::average(&activity.get_data("power")).unwrap();
        let np = calc_normalized_power(&activity.get_data("power")).unwrap();

        let VI(variability_index) = VI::calculate(&np, &avg_power);

        assert_in_delta!(variability_index, 1.075, 0.0005);
    }

    #[test]
    fn activity_file_tss() {
        let mut fp = File::open("./tests/fixtures/Activity.fit").unwrap();
        let activity = Activity::from_reader(&mut fp).unwrap();
        let ftp = Power(260);
        let np = calc_normalized_power(&activity.get_data("power")).unwrap();

        let tss = TSS::calculate(&ftp, &activity.duration.unwrap(), &np);

        assert_eq!(tss, TSS(67));
    }
}
