use activity_analyser::activity::Activity;
use activity_analyser::activity_analysis::ActivityAnalysis;
use activity_analyser::athlete::{MeasurementRecord, MeasurementRecords};
use activity_analyser::measurements::{HeartRate, Power, Weight};
use chrono::{Duration, NaiveDate};
use clap::Parser;
use fitparser::{self, Error};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
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

struct DisplayableOption<T>(Option<T>);

impl<T> Display for DisplayableOption<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match &self.0 {
            Some(x) => T::fmt(&x, f),
            None => write!(f, "-"),
        }
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
    let peak_durations = HashSet::from([
        Duration::seconds(5),
        Duration::minutes(1),
        Duration::minutes(5),
        Duration::minutes(20),
    ]);
    let activity_analysis =
        ActivityAnalysis::from_activity(&measurements, &activity, peak_durations);

    println!("Start time: {}", DisplayableOption(activity.start_time));
    println!("Duration: {}", DisplayableOption(activity.duration));
    println!(
        "Average power: {}",
        DisplayableOption(activity_analysis.average_power)
    );
    println!(
        "Normalized power: {}",
        DisplayableOption(activity_analysis.normalized_power)
    );
    println!(
        "Variability Index: {:.2}",
        DisplayableOption(activity_analysis.variability_index)
    );
    println!(
        "Intensity Factor: {:.2}",
        DisplayableOption(activity_analysis.intensity_factor)
    );
    println!("Total Work: {}", activity_analysis.total_work);
    println!("TSS: {}", DisplayableOption(activity_analysis.tss));
    println!("hrTSS: {}", DisplayableOption(activity_analysis.hr_tss));
    println!(
        "Elevation gain: {}",
        DisplayableOption(activity_analysis.elevation_gain)
    );
    println!(
        "Elevation loss: {}",
        DisplayableOption(activity_analysis.elevation_loss)
    );
    Ok(())

    // -- when verbose $ print $ foldl' (\m k -> Map.lookup k =<< m) (Just activity.resolved) filterWords

    // when verbose $ case filterWords of
    //   [] -> pPrint activity.resolved
    //   [k] -> pPrint $ Map.lookup k activity.resolved
    //   k : k' : _ -> pPrint $ do
    //     resolvedMsgs <- Map.lookup k activity.resolved
    //     pure $ foldMap (\m -> catMaybes [Map.lookup k' m.fields, Map.lookup k' m.devFields]) resolvedMsgs
}
