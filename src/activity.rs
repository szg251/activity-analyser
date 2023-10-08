use chrono::{DateTime, Duration, Local};
use fitparser::profile::field_types::MesgNum;
use fitparser::{self, Error, FitDataRecord, Value};
use std::io::Read;

#[derive(Debug)]
pub struct Activity {
    pub workout_name: Option<String>,
    pub start_time: Option<DateTime<Local>>,
    pub duration: Option<Duration>,
    pub records: Vec<FitDataRecord>,
    pub bytes: Vec<u8>,
}

impl Activity {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let bytes = bytes.clone();

        let records = fitparser::from_bytes(&bytes)?;
        let workout_name = find_one_value(&records, &MesgNum::Workout, "wkt_name")
            .map(get_str)
            .flatten()
            .cloned();
        let start_time = find_one_value(&records, &MesgNum::Session, "start_time")
            .map(get_timestamp)
            .flatten()
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

    pub fn from_reader<T: Read>(source: &mut T) -> Result<Self, Error> {
        let mut buffer = Vec::new();
        source.read_to_end(&mut buffer)?;
        Self::from_bytes(&buffer)
    }

    pub fn find_one_value(self: &Self, mesg_num: &MesgNum, field_name: &str) -> Option<&Value> {
        find_one_value(&self.records, mesg_num, field_name)
    }

    pub fn find_many_values(self: &Self, mesg_num: &MesgNum, field_name: &str) -> Vec<&Value> {
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

    pub fn find_many_values_with_timestamps(
        self: &Self,
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
                Some((value, get_timestamp(timestamp)?))
            })
            .collect()
    }

    pub fn get_data<T>(self: &Self, field_name: &str) -> Vec<T>
    where
        T: TryFrom<Value>,
    {
        self.find_many_values(&MesgNum::Record, field_name)
            .iter()
            .filter_map(|v| Some((*v).clone().try_into().ok()?))
            .collect()
    }

    pub fn get_data_with_timestamps<T>(self: &Self, field_name: &str) -> Vec<(T, &DateTime<Local>)>
    where
        T: TryFrom<Value>,
    {
        self.find_many_values_with_timestamps(&MesgNum::Record, field_name)
            .iter()
            .filter_map(|(v, t)| Some(((*v).clone().try_into().ok()?, *t)))
            .collect()
    }
}

fn find_one_value<'a>(
    records: &'a Vec<FitDataRecord>,
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

fn get_str<'a>(value: &'a Value) -> Option<&'a String> {
    match value {
        Value::String(str) => Some(&str),
        _ => None,
    }
}

fn get_timestamp<'a>(value: &'a Value) -> Option<&'a DateTime<Local>> {
    match value {
        Value::Timestamp(timestamp) => Some(&timestamp),
        _ => None,
    }
}

fn find_duration(records: &Vec<FitDataRecord>) -> Option<Duration> {
    let total_moving_time = find_one_value(&records, &MesgNum::Session, "total_moving_time");
    let total_elapsed_time = find_one_value(&records, &MesgNum::Session, "total_elapsed_time");
    let total_timer_time = find_one_value(&records, &MesgNum::Session, "total_timer_time");

    let duration: f64 = total_moving_time
        .or(total_elapsed_time)
        .or(total_timer_time)?
        .clone()
        .try_into()
        .ok()?;

    Some(Duration::seconds(duration as i64))
}
