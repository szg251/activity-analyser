use crate::activity::Activity;
use crate::athlete::MeasurementRecords;
use crate::measurements::{Altitude, AltitudeDiff, Average, HeartRate, Power, Speed, Work};
use crate::metrics::TSS;
use crate::peak::Peak;
use chrono::{DateTime, Duration, Local, NaiveDate};
use std::collections::{HashMap, HashSet};

/// Results of a full activity analysis
#[derive(Debug)]
pub struct ActivityAnalysis {
    pub total_work: Work,
    pub normalized_power: Option<Power>,
    pub intensity_factor: Option<f64>,
    pub variability_index: Option<f64>,
    pub tss: Option<TSS>,
    pub hr_tss: Option<TSS>,
    pub average_power: Option<Power>,
    pub maximum_power: Option<Power>,
    pub average_heart_rate: Option<HeartRate>,
    pub maximum_heart_rate: Option<HeartRate>,
    pub average_speed: Option<Speed>,
    pub maximum_speed: Option<Speed>,
    pub elevation_gain: Option<AltitudeDiff>,
    pub elevation_loss: Option<AltitudeDiff>,
    pub peak_performances: PeakPerformances,
}

impl ActivityAnalysis {
    /// Analyse an activity and create an ActivityAnalysis
    pub fn from_activity(
        measurement_records: &MeasurementRecords,
        activity: &Activity,
        peak_durations: &HashSet<Duration>,
    ) -> Self {
        let date: Option<NaiveDate> = activity.start_time.map(|t| t.naive_utc().into());
        let ftp = date.and_then(|d| measurement_records.get_actual_ftp(&d));
        let fthr = date.and_then(|d| measurement_records.get_actual_fthr(&d));

        let power_data_with_timestamps = activity.get_data_with_timestamps("power");
        let power_data = power_data_with_timestamps
            .iter()
            .map(|t| t.0)
            .collect::<Vec<_>>();

        let heart_rate_data_with_timestamps = activity.get_data_with_timestamps("heart_rate");
        let heart_rate_data = heart_rate_data_with_timestamps
            .iter()
            .map(|t| t.0)
            .collect::<Vec<_>>();

        let speed_data_with_timestamps = activity.get_data_with_timestamps("enhanced_speed");
        let speed_data = speed_data_with_timestamps
            .iter()
            .map(|t| t.0)
            .collect::<Vec<_>>();

        let altitude_data = activity.get_data("altitude");

        let average_power = Average::average(&power_data);
        let maximum_power = power_data.iter().max().copied();

        let average_heart_rate = Average::average(&heart_rate_data);
        let maximum_heart_rate = heart_rate_data.iter().max().copied();

        let average_speed = Average::average(&speed_data);
        let maximum_speed = speed_data
            .iter()
            .max_by(|Speed(x), Speed(y)| x.total_cmp(y))
            .copied();

        let total_work = calc_total_work(&power_data);
        let normalized_power = calc_normalized_power(&power_data);
        let intensity_factor = match (ftp, normalized_power) {
            (Some(ftp), Some(normalized_power)) => {
                Some(calc_intensity_factor(&ftp, &normalized_power))
            }
            _ => None,
        };
        let variability_index = match (normalized_power, average_power) {
            (Some(normalized_power), Some(average_power)) => {
                Some(calc_variability_index(&normalized_power, &average_power))
            }
            _ => None,
        };
        let tss = match (ftp, &activity.duration, &normalized_power) {
            (Some(ftp), Some(duration), Some(normalized_power)) => {
                Some(calc_tss(&ftp, &duration, &normalized_power))
            }
            _ => None,
        };
        let hr_tss = fthr.map(|fthr| calc_hr_tss(&fthr, &heart_rate_data));
        let (elevation_gain, elevation_loss) = calc_altitude_changes(&altitude_data);

        let peak_performances = PeakPerformances::from_data(
            &power_data_with_timestamps,
            &heart_rate_data_with_timestamps,
            &speed_data_with_timestamps,
            &peak_durations,
        );

        Self {
            total_work,
            normalized_power,
            intensity_factor,
            variability_index,
            tss,
            hr_tss,
            average_power,
            maximum_power,
            average_heart_rate,
            maximum_heart_rate,
            average_speed,
            maximum_speed,
            elevation_gain,
            elevation_loss,
            peak_performances,
        }
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

/// Calculate Intensity Factor
pub fn calc_intensity_factor(ftp: &Power, normalized_power: &Power) -> f64 {
    let Power(ftp) = *ftp;
    let Power(normalized_power) = *normalized_power;
    normalized_power as f64 / ftp as f64
}

/// Calculate Variablity Index
pub fn calc_variability_index(normalized_power: &Power, average_power: &Power) -> f64 {
    let Power(normalized_power) = *normalized_power;
    let Power(average_power) = *average_power;

    normalized_power as f64 / average_power as f64
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

/// Calculate user specific Training Stress Scores
pub fn calc_tss(ftp: &Power, duration: &Duration, normalized_power: &Power) -> TSS {
    let intensity_factor = calc_intensity_factor(ftp, normalized_power);
    let Power(ftp) = *ftp;
    let Power(normalized_power) = *normalized_power;
    let duration = duration.num_seconds() as f64;

    TSS(
        (((duration * (normalized_power as f64) * intensity_factor) / (ftp as f64 * 3_600.0))
            * 100.0) as i64,
    )
}

/// Calculate user specific Heart Rate Training Stress Score
pub fn calc_hr_tss(fthr: &HeartRate, heart_rate_data: &Vec<HeartRate>) -> TSS {
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

    let zones_count =
        heart_rate_data
            .iter()
            .fold((0, 0, 0, 0, 0, 0, 0, 0, 0, 0), |mut acc, HeartRate(hr)| {
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
            });

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

/// Highest performance values achieved for certain time durations
#[derive(Debug)]
pub struct PeakPerformances {
    pub power: HashMap<Duration, Peak<Power>>,
    pub heart_rate: HashMap<Duration, Peak<HeartRate>>,
    pub speed: HashMap<Duration, Peak<Speed>>,
}

impl PeakPerformances {
    /// Calculate peak performances for multiple measurement types
    pub fn from_data(
        power_data: &Vec<(Power, &DateTime<Local>)>,
        heart_rate_data: &Vec<(HeartRate, &DateTime<Local>)>,
        speed_data: &Vec<(Speed, &DateTime<Local>)>,
        peak_durations: &HashSet<Duration>,
    ) -> Self {
        Self {
            power: Self::get_one(power_data, &peak_durations),
            heart_rate: Self::get_one(heart_rate_data, &peak_durations),
            speed: Self::get_one(speed_data, &peak_durations),
        }
    }

    /// Calculate performances for a specific measurment type
    fn get_one<T>(
        data_with_timestamps: &Vec<(T, &DateTime<Local>)>,
        peak_durations: &HashSet<Duration>,
    ) -> HashMap<Duration, Peak<T>>
    where
        T: Ord + Average + Copy,
    {
        peak_durations
            .iter()
            .filter_map(|duration| {
                Some((
                    duration.clone(),
                    Peak::from_measurement_records(data_with_timestamps, *duration)?,
                ))
            })
            .collect()
    }
}

#[cfg(test)]
mod activity_analysis_tests {
    use super::*;
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
        let tss = calc_tss(&Power(260), &Duration::hours(1), &Power(260));
        assert_eq!(tss, TSS(100))
    }

    #[test]
    fn ninety_minute_effort_tss() {
        let tss = calc_tss(&Power(260), &Duration::minutes(90), &Power(260));
        assert_eq!(tss, TSS(150))
    }

    #[test]
    fn four_hour_effort_tss() {
        let tss = calc_tss(&Power(260), &Duration::hours(4), &Power(130));
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

        let intensity_factor = calc_intensity_factor(&ftp, &np);

        assert_in_delta!(intensity_factor, 0.82, 0.005);
    }

    #[test]
    fn activity_file_variability_index() {
        let mut fp = File::open("./tests/fixtures/Activity.fit").unwrap();
        let activity = Activity::from_reader(&mut fp).unwrap();
        let avg_power = Average::average(&activity.get_data("power")).unwrap();
        let np = calc_normalized_power(&activity.get_data("power")).unwrap();

        let variability_index = calc_variability_index(&np, &avg_power);

        assert_in_delta!(variability_index, 1.075, 0.0005);
    }

    #[test]
    fn activity_file_tss() {
        let mut fp = File::open("./tests/fixtures/Activity.fit").unwrap();
        let activity = Activity::from_reader(&mut fp).unwrap();
        let ftp = Power(260);
        let np = calc_normalized_power(&activity.get_data("power")).unwrap();

        let tss = calc_tss(&ftp, &activity.duration.unwrap(), &np);

        assert_eq!(tss, TSS(67));
    }
}
