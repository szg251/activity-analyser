use chrono::{DateTime, Duration, Local};
use fitparser::profile::field_types::MesgNum;
use fitparser::{self, Error, FitDataRecord, Value};
use std::io::Read;

/// Parsed activity data with some basic fields
#[derive(Debug)]
pub struct Activity {
    pub workout_name: Option<String>,
    pub start_time: Option<DateTime<Local>>,
    pub duration: Option<Duration>,
    pub records: Vec<FitDataRecord>,
    pub bytes: Vec<u8>,
}

impl Activity {
    /// Parse a slice of bytes into an Activity
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let records = fitparser::from_bytes(bytes)?;
        let workout_name = find_one_value(&records, &MesgNum::Workout, "wkt_name")
            .and_then(value_to_str)
            .cloned();
        let start_time = find_one_value(&records, &MesgNum::Session, "start_time")
            .and_then(value_to_timestamp)
            .cloned();
        let duration = find_duration(&records);
        Ok(Self {
            workout_name,
            start_time,
            duration,
            records,
            bytes: bytes.to_vec(),
        })
    }

    /// Parse a file into an Activity
    pub fn from_reader<T: Read>(source: &mut T) -> Result<Self, Error> {
        let mut buffer = Vec::new();
        source.read_to_end(&mut buffer)?;
        Self::from_bytes(&buffer)
    }

    /// Find a singular raw FIT value
    pub fn find_one_value(&self, mesg_num: &MesgNum, field_name: &str) -> Option<&Value> {
        find_one_value(&self.records, mesg_num, field_name)
    }

    /// Find multiple raw FIT values
    pub fn find_many_values(&self, mesg_num: &MesgNum, field_name: &str) -> Vec<&Value> {
        self.records
            .iter()
            .filter_map(|record| {
                if record.kind() == *mesg_num {
                    Some(record.fields())
                } else {
                    None
                }
            })
            .filter_map(|fields| {
                let value = fields
                    .iter()
                    .find(|field| field.name() == field_name)?
                    .value();

                Some(value)
            })
            .collect()
    }

    /// Find multiple raw FIT values with their respective timestapms
    pub fn find_many_values_with_timestamps(
        &self,
        mesg_num: &MesgNum,
        field_name: &str,
    ) -> Vec<(&Value, &DateTime<Local>)> {
        self.records
            .iter()
            .filter_map(|record| {
                if record.kind() == *mesg_num {
                    Some(record.fields())
                } else {
                    None
                }
            })
            .filter_map(|fields| {
                let value = fields
                    .iter()
                    .find(|field| field.name() == field_name)?
                    .value();

                let timestamp = fields
                    .iter()
                    .find(|field| field.name() == "timestamp")?
                    .value();
                Some((value, value_to_timestamp(timestamp)?))
            })
            .collect()
    }

    /// Get a vector of converted data from an activity
    pub fn get_data<T>(&self, field_name: &str) -> Vec<T>
    where
        T: TryFrom<Value>,
    {
        self.find_many_values(&MesgNum::Record, field_name)
            .iter()
            .filter_map(|v| (*v).clone().try_into().ok())
            .collect()
    }

    /// Get a vector of converted data from an activity with their respective timestamps
    pub fn get_data_with_timestamps<T>(&self, field_name: &str) -> Vec<(T, &DateTime<Local>)>
    where
        T: TryFrom<Value>,
    {
        self.find_many_values_with_timestamps(&MesgNum::Record, field_name)
            .iter()
            .filter_map(|(v, t)| Some(((*v).clone().try_into().ok()?, *t)))
            .collect()
    }
}

/// Find a singular value
fn find_one_value<'a>(
    records: &'a [FitDataRecord],
    mesg_num: &MesgNum,
    field_name: &str,
) -> Option<&'a Value> {
    records
        .iter()
        .filter_map(|record| {
            if record.kind() == *mesg_num {
                Some(record.fields())
            } else {
                None
            }
        })
        .flatten()
        .find_map(|field| {
            if field.name() == field_name {
                Some(field.value())
            } else {
                None
            }
        })
}

/// Convert a Value to a String
fn value_to_str(value: &Value) -> Option<&String> {
    match value {
        Value::String(str) => Some(str),
        _ => None,
    }
}

/// Convert a Value to a timestamp
fn value_to_timestamp(value: &Value) -> Option<&DateTime<Local>> {
    match value {
        Value::Timestamp(timestamp) => Some(timestamp),
        _ => None,
    }
}

/// Find the duration of an activity based on multiple fallback values
fn find_duration(records: &[FitDataRecord]) -> Option<Duration> {
    let total_moving_time = find_one_value(records, &MesgNum::Session, "total_moving_time");
    let total_elapsed_time = find_one_value(records, &MesgNum::Session, "total_elapsed_time");
    let total_timer_time = find_one_value(records, &MesgNum::Session, "total_timer_time");

    let duration: f64 = total_moving_time
        .or(total_elapsed_time)
        .or(total_timer_time)?
        .clone()
        .try_into()
        .ok()?;

    Some(Duration::seconds(duration as i64))
}
