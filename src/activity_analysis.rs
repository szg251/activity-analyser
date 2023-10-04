use crate::activity::Activity;
use crate::athlete::Measurements;
use crate::metrics::{Altitude, AltitudeDiff, Average, HeartRate, Power, Speed, Work};
use chrono::{DateTime, Duration, Local, NaiveDate};

#[derive(Debug)]
pub struct ActivityAnalysis {
    pub total_work: Work,
    pub normalized_power: Option<Power>,
    pub intensity_factor: Option<f64>,
    pub variability_index: Option<f64>,
    pub tss: Option<i64>,
    pub hr_tss: Option<i64>,
    pub average_power: Option<Power>,
    pub maximum_power: Option<Power>,
    pub average_heart_rate: Option<HeartRate>,
    pub maximum_heart_rate: Option<HeartRate>,
    pub average_speed: Option<Speed>,
    pub maximum_speed: Option<Speed>,
    pub elevation_gain: Option<AltitudeDiff>,
    pub elevation_loss: Option<AltitudeDiff>,
}

impl ActivityAnalysis {
    pub fn from_activity(measurements: &Measurements, activity: &Activity) -> Self {
        let date: Option<NaiveDate> = activity.start_time.map(|t| t.naive_utc().into());
        let ftp = date.and_then(|d| measurements.get_actual_ftp(&d));
        let fthr = date.and_then(|d| measurements.get_actual_fthr(&d));

        let power_data = activity.get_data_with_timestamps("power");
        let power_only: Vec<Power> = power_data.iter().map(|(p, _)| *p).collect();

        let heart_rate_data = activity.get_data_with_timestamps("heart_rate");
        let heart_rate_only: Vec<HeartRate> = heart_rate_data.iter().map(|(hr, _)| *hr).collect();

        let speed_data = activity.get_data_with_timestamps("speed");
        let speed_only: Vec<Speed> = speed_data.iter().map(|(sp, _)| *sp).collect();

        let altitude_data = activity.get_data_with_timestamps("altitude");
        let altitude_only: Vec<Altitude> = altitude_data.iter().map(|(alt, _)| *alt).collect();

        let average_power = Average::average(&power_only);
        let maximum_power = power_only.iter().max().copied();

        let average_heart_rate = Average::average(&heart_rate_only);
        let maximum_heart_rate = heart_rate_only.iter().max().copied();

        let average_speed = Average::average(&speed_only);
        let maximum_speed = speed_only
            .iter()
            .max_by(|Speed(x), Speed(y)| x.total_cmp(y))
            .copied();

        let total_work = calc_total_work(&power_data);
        let normalized_power = calc_normalized_power(&power_only);
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
        let tss = match (ftp, activity.duration, normalized_power) {
            (Some(ftp), Some(duration), Some(normalized_power)) => {
                Some(calc_tss(&ftp, &duration, &normalized_power))
            }
            _ => None,
        };
        let hr_tss = fthr.map(|fthr| calc_hr_tss(&fthr, &heart_rate_only));
        let (elevation_gain, elevation_loss) = calc_altitude_changes(altitude_only);

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
        }
    }
}

pub fn calc_total_work(power_data: &Vec<(Power, &DateTime<Local>)>) -> Work {
    Work(power_data.iter().map(|(Power(power), _)| *power).sum())
}

pub fn calc_normalized_power(power_only: &Vec<Power>) -> Option<Power> {
    // Returning simple average, if data size doesn't hit threshold
    if power_only.len() < 30 {
        return Average::average(power_only);
    }

    let avg: i64 = Average::average(
        rolling_averages(power_only, 30)
            .iter()
            .map(|Power(x)| x.pow(4))
            .collect::<Vec<i64>>(),
    )?;

    let result = (avg as f64).powf(0.25) as i64;
    Some(Power(result))
}

pub fn average<T>(elems: T) -> i64
where
    T: AsRef<[i64]>,
{
    let elems = elems.as_ref();
    elems.iter().sum::<i64>() / elems.len() as i64
}

pub fn calc_intensity_factor(ftp: &Power, normalized_power: &Power) -> f64 {
    let Power(ftp) = *ftp;
    let Power(normalized_power) = *normalized_power;
    normalized_power as f64 / ftp as f64
}

pub fn calc_variability_index(normalized_power: &Power, average_power: &Power) -> f64 {
    let Power(normalized_power) = *normalized_power;
    let Power(average_power) = *average_power;

    normalized_power as f64 / average_power as f64
}

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

pub fn calc_tss(ftp: &Power, duration: &Duration, normalized_power: &Power) -> i64 {
    let intensity_factor = calc_intensity_factor(ftp, normalized_power);
    let Power(ftp) = *ftp;
    let Power(normalized_power) = *normalized_power;
    let duration = duration.num_seconds() as f64;

    (((duration * (normalized_power as f64) * intensity_factor) / (ftp as f64 * 3_600.0)) * 100.0)
        as i64
}

pub fn calc_hr_tss(fthr: &HeartRate, heart_rate_data: &Vec<HeartRate>) -> i64 {
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

    zones_count.0 * 20
        + zones_count.1 * 30
        + zones_count.2 * 40
        + zones_count.3 * 50
        + zones_count.4 * 60
        + zones_count.5 * 75
        + zones_count.6 * 100
        + zones_count.7 * 105
        + zones_count.8 * 110
        + zones_count.9 * 120
}

pub fn calc_altitude_changes(
    altitude_only: Vec<Altitude>,
) -> (Option<AltitudeDiff>, Option<AltitudeDiff>) {
    let init: (
        Option<AltitudeDiff>,
        Option<AltitudeDiff>,
        Option<&Altitude>,
    ) = (None, None, None);
    let (gain, loss, _) = altitude_only.iter().fold(
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
mod test {
    use super::*;
    use chrono::{DateTime, Duration, Local};

    #[test]
    /// Don't panic on small data (less than 30 seconds)
    fn small_data() {
        let power_data: Vec<(Power, DateTime<Local>)> = vec![
            (Power(200), "2012-12-12 12:12:12Z".parse().unwrap()),
            (Power(200), "2012-12-12 12:12:13Z".parse().unwrap()),
            (Power(200), "2012-12-12 12:12:14Z".parse().unwrap()),
            (Power(200), "2012-12-12 12:12:15Z".parse().unwrap()),
        ];
        let power_data = power_data.iter().map(|(p, t)| (*p, t)).collect();

        assert_eq!(calc_normalized_power(&power_data), Power(200));
    }

    #[test]
    /// Constant effort NP should be equal to average power
    fn constant_effort() {
        let power_data: Vec<(Power, DateTime<Local>)> = (0..3600)
            .map(|s| {
                (
                    Power(200),
                    "2012-12-12 12:12:12Z".parse::<DateTime<Local>>().unwrap()
                        + Duration::seconds(s),
                )
            })
            .collect();
        let power_data = power_data.iter().map(|(p, t)| (*p, t)).collect();

        assert_eq!(calc_normalized_power(&power_data), Power(200));
    }
}
