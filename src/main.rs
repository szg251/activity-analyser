#[macro_use]
extern crate prettytable;
use activity_analyser::activity::Activity;
use activity_analyser::activity_analysis::ActivityAnalysis;
use activity_analyser::athlete::{MeasurementRecord, MeasurementRecords};
use activity_analyser::daily_stats::{DailyStats, DailyTSS, SortedDailyTSS};
use activity_analyser::measurements::{HeartRate, Power, Weight};
use chrono::{Duration, Local, NaiveDate};
use clap::Parser;
use fitparser::{self, Error};
use prettytable::format;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::fs;
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
        verbose: bool,
    },
    MultiActivity {
        /// Path to the directory containing FIT files
        #[arg(short, long)]
        path: PathBuf,
        /// Print verbose logs
        #[arg(short, long)]
        verbose: bool,
    },
}

fn main() -> Result<(), Error> {
    let cli = Args::parse();

    match cli {
        Args::SingleActivity { path, verbose } => single_activity(path, verbose),
        Args::MultiActivity { path, verbose } => multi_activity(path, verbose),
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

fn def_measurements() -> MeasurementRecords {
    MeasurementRecords::new([
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
    ])
}

fn single_activity(path: PathBuf, verbose: bool) -> Result<(), Error> {
    let measurements = def_measurements();

    println!(
        "Parsing FIT files using Profile version: {}",
        fitparser::profile::VERSION
    );
    let mut fp = fs::File::open(path)?;
    let activity = Activity::from_reader(&mut fp)?;
    let peak_durations = HashSet::from([
        Duration::seconds(5),
        Duration::minutes(1),
        Duration::minutes(5),
        Duration::minutes(20),
    ]);
    let activity_analysis =
        ActivityAnalysis::from_activity(&measurements, &activity, &peak_durations);

    let mut data_table = table![
        ["Start time", DisplayableOption(activity.start_time)],
        ["Duration", DisplayableOption(activity.duration)],
        [
            "Average power",
            DisplayableOption(activity_analysis.average_power)
        ],
        [
            "Normalized power",
            DisplayableOption(activity_analysis.normalized_power)
        ],
        [
            "Variability Index",
            format!(
                "{:.2}",
                DisplayableOption(activity_analysis.variability_index)
            )
        ],
        [
            "Intensity Factor",
            format!(
                "{:.2}",
                DisplayableOption(activity_analysis.intensity_factor)
            )
        ],
        ["Total Work", activity_analysis.total_work],
        ["TSS", DisplayableOption(activity_analysis.tss)],
        ["hrTSS", DisplayableOption(activity_analysis.hr_tss)],
        [
            "Elevation gain",
            DisplayableOption(activity_analysis.elevation_gain)
        ],
        [
            "Elevation loss",
            DisplayableOption(activity_analysis.elevation_loss)
        ]
    ];

    data_table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    data_table.printstd();

    let mut peaks_table = table![
        [
            "Power (5s)",
            DisplayableOption(
                activity_analysis
                    .peak_performances
                    .power
                    .get(&Duration::seconds(5))
                    .map(|x| x.value)
            )
        ],
        [
            "Power (1m)",
            DisplayableOption(
                activity_analysis
                    .peak_performances
                    .power
                    .get(&Duration::minutes(1))
                    .map(|x| x.value)
            )
        ],
        [
            "Power (5m)",
            DisplayableOption(
                activity_analysis
                    .peak_performances
                    .power
                    .get(&Duration::minutes(5))
                    .map(|x| x.value)
            )
        ],
        [
            "Power (20m)",
            DisplayableOption(
                activity_analysis
                    .peak_performances
                    .power
                    .get(&Duration::minutes(20))
                    .map(|x| x.value)
            )
        ],
        [
            "Speed (5s)",
            DisplayableOption(
                activity_analysis
                    .peak_performances
                    .speed
                    .get(&Duration::seconds(5))
                    .map(|x| x.value)
            )
        ],
        [
            "Speed (1m)",
            DisplayableOption(
                activity_analysis
                    .peak_performances
                    .speed
                    .get(&Duration::minutes(1))
                    .map(|x| x.value)
            )
        ],
        [
            "Speed (5m)",
            DisplayableOption(
                activity_analysis
                    .peak_performances
                    .speed
                    .get(&Duration::minutes(5))
                    .map(|x| x.value)
            )
        ],
        [
            "Speed (20m)",
            DisplayableOption(
                activity_analysis
                    .peak_performances
                    .speed
                    .get(&Duration::minutes(20))
                    .map(|x| x.value)
            )
        ],
        [
            "Heart rate (5s)",
            DisplayableOption(
                activity_analysis
                    .peak_performances
                    .heart_rate
                    .get(&Duration::seconds(5))
                    .map(|x| x.value)
            )
        ],
        [
            "Heart rate (1m)",
            DisplayableOption(
                activity_analysis
                    .peak_performances
                    .heart_rate
                    .get(&Duration::minutes(1))
                    .map(|x| x.value)
            )
        ],
        [
            "Heart rate (5m)",
            DisplayableOption(
                activity_analysis
                    .peak_performances
                    .heart_rate
                    .get(&Duration::minutes(5))
                    .map(|x| x.value)
            )
        ],
        [
            "Heart rate (20m)",
            DisplayableOption(
                activity_analysis
                    .peak_performances
                    .heart_rate
                    .get(&Duration::minutes(20))
                    .map(|x| x.value)
            )
        ]
    ];

    peaks_table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    peaks_table.printstd();

    if verbose {
        println!("{:#?}", activity.records);
    };
    Ok(())
}

fn multi_activity(path: PathBuf, verbose: bool) -> Result<(), Error> {
    let measurements = def_measurements();
    let files = fs::read_dir(path)?;

    let (successes, failures): (Vec<Result<Activity, Error>>, Vec<Result<Activity, Error>>) = files
        .map(|entry| {
            let mut fp = fs::File::open(entry?.path())?;
            Ok(Activity::from_reader(&mut fp)?)
        })
        .partition(Result::is_ok);

    let successes = successes
        .iter()
        .map(|x| x.as_ref().unwrap())
        .collect::<Vec<_>>();
    let failures = failures
        .iter()
        .map(|x| x.as_ref().unwrap_err())
        .collect::<Vec<_>>();

    println!("Successfully parsed files: {}", successes.len());
    println!("Failed files: {}", failures.len());

    let peak_durations = HashSet::from([
        Duration::seconds(5),
        Duration::minutes(1),
        Duration::minutes(5),
        Duration::minutes(20),
    ]);
    let today = Local::now().date_naive();

    let activities_with_analyses = successes
        .iter()
        .map(|activity| {
            (
                activity,
                ActivityAnalysis::from_activity(&measurements, &activity, &peak_durations),
            )
        })
        .collect::<Vec<_>>();

    let daily_tss_data = activities_with_analyses
        .iter()
        .filter_map(|(activity, analysis)| {
            Some(DailyTSS(
                activity.start_time?.date_naive(),
                analysis.tss.or(analysis.hr_tss)?,
            ))
        })
        .collect::<Vec<_>>();
    let sorted_daily_tss = SortedDailyTSS::from_unsorted(&daily_tss_data, None);
    let daily_stats = DailyStats::calc_rolling(sorted_daily_tss, None);

    let todays_stats = daily_stats
        .iter()
        .find(|daily_stats| daily_stats.date == today);

    let mut pm_table = table![
        ["CTL", DisplayableOption(todays_stats.map(|x| x.ctl))],
        ["ATL", DisplayableOption(todays_stats.map(|x| x.atl))],
        ["TSB", DisplayableOption(todays_stats.map(|x| x.tsb))]
    ];

    pm_table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    pm_table.printstd();

    if verbose {
        println!("{:#?}", daily_stats);
    }
    Ok(())
}
