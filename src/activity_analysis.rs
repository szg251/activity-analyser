use crate::activity::Activity;
use crate::measurements::{AltitudeDiff, Average, HeartRate, Power, Speed, Work};
use crate::metrics::{calc_altitude_changes, calc_normalized_power, calc_total_work, IF, TSS, VI};
use crate::peak::Peak;
use chrono::{DateTime, Duration, Local};
use std::collections::{HashMap, HashSet};

/// Results of a full activity analysis
#[derive(Debug)]
pub struct ActivityAnalysis {
    pub total_work: Work,
    pub normalized_power: Option<Power>,
    pub intensity_factor: Option<IF>,
    pub variability_index: Option<VI>,
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
        ftp: &Option<Power>,
        fthr: &Option<HeartRate>,
        activity: &Activity,
        peak_durations: &HashSet<Duration>,
    ) -> Self {
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
            (Some(ftp), Some(normalized_power)) => Some(IF::calculate(&ftp, &normalized_power)),
            _ => None,
        };
        let variability_index = match (normalized_power, average_power) {
            (Some(normalized_power), Some(average_power)) => {
                Some(VI::calculate(&normalized_power, &average_power))
            }
            _ => None,
        };
        let tss = match (ftp, &activity.duration, &normalized_power) {
            (Some(ftp), Some(duration), Some(normalized_power)) => {
                Some(TSS::calculate(&ftp, &duration, &normalized_power))
            }
            _ => None,
        };
        let hr_tss = fthr.map(|fthr| TSS::calculate_hr_tss(&fthr, &heart_rate_data));
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

/// Highest performance values achieved for certain time durations
#[derive(Debug, Clone)]
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
