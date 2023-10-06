use chrono::{Days, NaiveDate};
use derive_more::{Add, AddAssign};
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct DailyTSS(pub NaiveDate, pub TSS);

pub struct SortedDailyTSS(Vec<DailyTSS>);

#[derive(Clone)]
pub struct DailyStats {
    pub date: NaiveDate,
    pub tss: TSS,
    pub ctl: CTL,
    pub atl: ATL,
    pub tsb: TSB,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Add, AddAssign)]
pub struct TSS(i64);

#[derive(Clone, PartialEq, PartialOrd)]
pub struct CTL(f64);

#[derive(Clone, PartialEq, PartialOrd)]
pub struct ATL(f64);

#[derive(Clone, PartialEq, PartialOrd)]
pub struct TSB(f64);

/// Calculate training load with a given decay and impact constant
fn calc_training_load(
    decay_const: i64,
    impact_const: i64,
    yesterdays_tl: f64,
    daily_tss: &DailyTSS,
) -> f64 {
    let TSS(tss) = daily_tss.1;
    let decay_factor = (-1.0 / decay_const as f64).exp();
    let impact_factor = 1.0 - (-1.0 / impact_const as f64).exp();

    yesterdays_tl * decay_factor + tss as f64 * impact_factor
}

/// Calculating Chronic Training Load (CTL), a 42 day average of daily TSS values
pub fn calc_ctl(CTL(yesterdays_tl): &CTL, daily_tss: &DailyTSS) -> CTL {
    CTL(calc_training_load(42, 42, *yesterdays_tl, daily_tss))
}

/// Calculating Acute Training Load (ATL), a 7 day average of daily TSS values
pub fn calc_atl(ATL(yesterdays_tl): &ATL, daily_tss: &DailyTSS) -> ATL {
    ATL(calc_training_load(7, 7, *yesterdays_tl, daily_tss))
}

pub fn calc_tsb(CTL(ctl): &CTL, ATL(atl): &ATL) -> TSB {
    TSB(ctl - atl)
}

pub fn calc_daily_stats(yesterdays_stats: &DailyStats, daily_tss: &DailyTSS) -> DailyStats {
    let ctl = calc_ctl(&yesterdays_stats.ctl, daily_tss);
    let atl = calc_atl(&yesterdays_stats.atl, daily_tss);
    let tsb = calc_tsb(&ctl, &atl);

    let DailyTSS(date, tss) = daily_tss;

    DailyStats {
        date: date.clone(),
        ctl,
        atl,
        tsb,
        tss: tss.clone(),
    }
}

/// Calculating rolling daily statistics, starting from the last known point.
/// Any daily TSS before the last known point will be disregarded.
/// Daily TSS must be sorted, and there must not be any gaps between the days.
pub fn calc_rolling_daily_stats(
    last_known_stats: Option<&DailyStats>,
    SortedDailyTSS(sorted_daily_tss): &SortedDailyTSS,
) -> Vec<DailyStats> {
    let sorted_daily_tss = sorted_daily_tss
        .iter()
        .filter(|daily_tss| {
            if let Some(daily_stats) = last_known_stats {
                daily_tss.0 > daily_stats.date
            } else {
                false
            }
        })
        .cloned()
        .collect::<Vec<DailyTSS>>();

    if sorted_daily_tss.is_empty() {
        return Vec::new();
    };

    let DailyTSS(first_day, _) = sorted_daily_tss[0];
    let DailyTSS(last_day, _) = sorted_daily_tss[sorted_daily_tss.len() - 1];
    let infinite_range = (1..).map(|days| DailyTSS(last_day + Days::new(days), TSS(0)));

    let init = match last_known_stats {
        Some(stats) => stats.clone(),
        None => DailyStats {
            date: first_day - Days::new(1),
            tss: TSS(0),
            ctl: CTL(0.0),
            atl: ATL(0.0),
            tsb: TSB(0.0),
        },
    };
    let length = sorted_daily_tss.len();

    sorted_daily_tss
        .into_iter()
        .chain(infinite_range)
        .enumerate()
        .scan(init, |yesterdays_stats, (i, daily_tss)| {
            let next_daily_stats = calc_daily_stats(&yesterdays_stats, &daily_tss);
            if i <= length
                || next_daily_stats.ctl < CTL(0.5)
                || next_daily_stats.atl < ATL(0.5)
                || next_daily_stats.tsb < TSB(0.5)
            {}
            Some(next_daily_stats)
        })
        .skip(1)
        .collect()
}

/// This function will Accumulate Daily Training Stress Scores from the whole set of trainings
pub fn sort_and_fill_daily_tss(unsorted: &Vec<DailyTSS>) -> SortedDailyTSS {
    // By definitions BTreeMap converts into an iterator sorted by keys
    let acc = unsorted
        .iter()
        // Accumulation step
        .fold(BTreeMap::new(), |mut acc, DailyTSS(date, tss)| {
            acc.entry(*date)
                .and_modify(|acc_tss| *acc_tss += *tss)
                .or_insert(*tss);

            acc
        })
        .iter()
        .map(|(date, tss)| DailyTSS(date.clone(), tss.clone()))
        // Filling gaps
        .fold(Vec::with_capacity(unsorted.len()), |mut acc, daily_tss| {
            match acc.last() {
                Some(DailyTSS(last_date, _)) => {
                    let diff = (daily_tss.0 - *last_date).num_days() as u64;
                    let last_date = last_date.clone();

                    (1..diff).for_each(|days| {
                        acc.push(DailyTSS(last_date + Days::new(days), TSS(0)));
                    });
                    acc.push(daily_tss);
                }
                None => acc.push(daily_tss),
            }
            acc
        });

    SortedDailyTSS(acc)
}
