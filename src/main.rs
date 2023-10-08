use activity_analyser::activity::Activity;
use activity_analyser::activity_analysis::ActivityAnalysis;
use activity_analyser::athlete::{MeasurementRecord, MeasurementRecords};
use activity_analyser::measurements::{HeartRate, Power, Weight};
use chrono::NaiveDate;
use clap::Parser;
use fitparser::{self, Error};
use std::fs::File;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
enum Args {
    SingleActivity {
        /// FIT file path
        #[arg(short, long)]
        path: PathBuf,
        /// Print verbose logs
        #[arg(short, long)]
        verbose: Option<bool>,
        /// Filter for certain records
        #[arg(short, long)]
        filter: Vec<String>,
    },
    MultiActivity {
        /// Path to the directory containing FIT files
        #[arg(short, long)]
        path: PathBuf,
    },
}

fn main() -> Result<(), Error> {
    let cli = Args::parse();

    match cli {
        Args::SingleActivity {
            path,
            verbose: _,
            filter: _,
        } => single_activity(path),
        _ => Ok(()),
    }
}

fn single_activity(path: PathBuf) -> Result<(), Error> {
    let measurements = MeasurementRecords::new([
        (
            NaiveDate::from_ymd_opt(2022, 4, 20).unwrap(),
            MeasurementRecord::FTP(Power(260)),
        ),
        (
            NaiveDate::from_ymd_opt(2022, 4, 20).unwrap(),
            MeasurementRecord::FTHr(HeartRate(178)),
        ),
        (
            NaiveDate::from_ymd_opt(2022, 4, 20).unwrap(),
            MeasurementRecord::Weight(Weight(70.0)),
        ),
    ]);

    println!(
        "Parsing FIT files using Profile version: {}",
        fitparser::profile::VERSION
    );
    let mut fp = File::open(path)?;
    let activity = Activity::from_reader(&mut fp)?;
    let activity_analysis = ActivityAnalysis::from_activity(&measurements, &activity);
    println!("{:#?}", activity);
    println!("{:#?}", activity_analysis);
    Ok(())
}
