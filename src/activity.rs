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
        let workout_name = find_one_str(&records, &MesgNum::Workout, "wkt_name").cloned();
        let start_time = find_one_timestamp(&records, &MesgNum::Session, "start_time").cloned();
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
}

fn find_one<'a>(
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

fn find_one_str<'a>(
    records: &'a Vec<FitDataRecord>,
    mesg_num: &MesgNum,
    field_name: &str,
) -> Option<&'a String> {
    match find_one(records, mesg_num, field_name)? {
        Value::String(str) => Some(&str),
        _ => None,
    }
}

fn find_one_timestamp<'a>(
    records: &'a Vec<FitDataRecord>,
    mesg_num: &MesgNum,
    field_name: &str,
) -> Option<&'a DateTime<Local>> {
    match find_one(records, mesg_num, field_name)? {
        Value::Timestamp(timestamp) => Some(&timestamp),
        _ => None,
    }
}

fn find_duration(records: &Vec<FitDataRecord>) -> Option<Duration> {
    let total_moving_time = find_one(records, &MesgNum::Session, "total_moving_time");
    let total_elapsed_time = find_one(records, &MesgNum::Session, "total_elapsed_time");
    let total_timer_time = find_one(records, &MesgNum::Session, "total_timer_time");

    let duration: f64 = total_moving_time
        .or(total_elapsed_time)
        .or(total_timer_time)?
        .clone()
        .try_into()
        .ok()?;

    Some(Duration::seconds(duration as i64))
}
