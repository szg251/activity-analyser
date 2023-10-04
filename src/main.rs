use crate::activity::Activity;
use crate::activity_analysis::ActivityAnalysis;
use crate::athlete::{Measurement, Measurements};
use crate::metrics::{HeartRate, Power, Weight};
use chrono::NaiveDate;
use fitparser::{self, Error};
use std::fs::File;

pub mod activity;
pub mod activity_analysis;
pub mod athlete;
pub mod metrics;

fn main() -> Result<(), Error> {
    let measurements = Measurements::new([
        (
            NaiveDate::from_ymd_opt(2022, 4, 20).unwrap(),
            Measurement::FTP(Power(260)),
        ),
        (
            NaiveDate::from_ymd_opt(2022, 4, 20).unwrap(),
            Measurement::FTHr(HeartRate(178)),
        ),
        (
            NaiveDate::from_ymd_opt(2022, 4, 20).unwrap(),
            Measurement::Weight(Weight(70.0)),
        ),
    ]);

    println!(
        "Parsing FIT files using Profile version: {}",
        fitparser::profile::VERSION
    );
    let mut fp =
        File::open("../../Cycling/FitFiles/Wahoo_SYSTM_N_Henderson_2_Rabbit_Mountain.fit")?;
    let activity = Activity::from_reader(&mut fp)?;
    let activity_analysis = ActivityAnalysis::from_activity(&measurements, &activity);
    println!("{:#?}", activity);
    println!("{:#?}", activity_analysis);
    Ok(())
}
