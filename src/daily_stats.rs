use crate::metrics::{DailyTSS, ATL, CTL, TSB, TSS};
use chrono::{Days, NaiveDate};
use std::collections::BTreeMap;

/// Peformance management metrics
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DailyStats {
    pub date: NaiveDate,
    pub tss: TSS,
    pub ctl: CTL,
    pub atl: ATL,
    pub tsb: TSB,
}

impl DailyStats {
    /// Calculate next day's performance management metrics based on the metrics of yesterday
    /// and the daily accumulated TSS
    pub fn calc_next(yesterdays_stats: &DailyStats, daily_tss: &DailyTSS) -> DailyStats {
        let ctl = CTL::calculate(&yesterdays_stats.ctl, daily_tss);
        let atl = ATL::calculate(&yesterdays_stats.atl, daily_tss);
        let tsb = TSB::calculate(&ctl, &atl);

        let DailyTSS(date, tss) = daily_tss;

        DailyStats {
            date: *date,
            ctl,
            atl,
            tsb,
            tss: *tss,
        }
    }

    /// Calculating rolling daily statistics, starting from the last known point.
    /// Any daily TSS before the last known point will be disregarded.
    /// Daily TSS must be sorted, and there must not be any gaps between the days.
    pub fn calc_rolling(
        SortedDailyTSS(sorted_daily_tss): SortedDailyTSS,
        last_known_stats: Option<&DailyStats>,
    ) -> Vec<DailyStats> {
        if sorted_daily_tss.is_empty() {
            return Vec::new();
        };

        let DailyTSS(first_day, _) = sorted_daily_tss[0];
        let DailyTSS(last_day, _) = sorted_daily_tss[sorted_daily_tss.len() - 1];

        let ending_days = (1..).map(|days| DailyTSS(last_day + Days::new(days), TSS(0)));

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
            .chain(ending_days)
            .enumerate()
            .scan(init, |yesterdays_stats, (i, daily_tss)| {
                let next_daily_stats = DailyStats::calc_next(yesterdays_stats, &daily_tss);
                *yesterdays_stats = next_daily_stats.clone();

                if i < length + 1
                    || next_daily_stats.ctl >= CTL(0.45)
                    || next_daily_stats.atl >= ATL(0.45)
                    || next_daily_stats.tsb >= TSB(0.45)
                {
                    Some(next_daily_stats)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SortedDailyTSS(Vec<DailyTSS>);

impl SortedDailyTSS {
    /// This function will accumulate daily Training Stress Scores from the whole set of activities
    /// in a format that is accepted by `DailyStats::calc_rolling`
    /// This does:
    /// - filter tss records older than last known stats, if exists
    /// - sort
    /// - fill gaps with 0 TSS days
    /// - fill beginning of the list with 0 TSS days, if last known stats exist
    pub fn from_unsorted(
        unsorted: &[DailyTSS],
        last_known_stats: Option<&DailyStats>,
    ) -> SortedDailyTSS {
        // In order to fill gap between the last known stat and the first daily tss record,
        // we initialise the BTreeMap with 0 TSS values.
        let init_map: BTreeMap<NaiveDate, TSS> = match last_known_stats {
            None => BTreeMap::new(),
            Some(stats) => match unsorted.first() {
                None => BTreeMap::new(),
                // As this is still unsorted, this is practically a random date in the date range.
                Some(DailyTSS(random_date, _)) => {
                    let days = (*random_date - stats.date).num_days();
                    (1..days)
                        .map(|days| (stats.date + Days::new(days as u64), TSS(0)))
                        .collect()
                }
            },
        };
        // By definitions BTreeMap converts into an iterator sorted by keys
        let acc = unsorted
            .iter()
            // Accumulation step
            .fold(init_map, |mut acc, DailyTSS(date, tss)| {
                acc.entry(*date)
                    .and_modify(|acc_tss| *acc_tss += *tss)
                    .or_insert(*tss);

                acc
            })
            .iter()
            .map(|(date, tss)| DailyTSS(*date, *tss))
            // Filling gaps
            .fold(Vec::with_capacity(unsorted.len()), |mut acc, daily_tss| {
                match acc.last() {
                    Some(DailyTSS(last_date, _)) => {
                        let diff = (daily_tss.0 - *last_date).num_days() as u64;
                        let last_date = *last_date;

                        (1..diff).for_each(|days| {
                            acc.push(DailyTSS(last_date + Days::new(days), TSS(0)));
                        });
                        acc.push(daily_tss);
                    }
                    None => acc.push(daily_tss),
                }
                acc
            })
            .into_iter()
            .skip_while(|DailyTSS(date, _)| {
                if let Some(daily_stats) = last_known_stats {
                    date <= &daily_stats.date
                } else {
                    false
                }
            })
            .collect();

        SortedDailyTSS(acc)
    }
}

#[cfg(test)]
mod daily_stats_tests {
    use crate::daily_stats::{DailyStats, DailyTSS, SortedDailyTSS, ATL, CTL, TSB, TSS};
    use assertables::*;
    use chrono::{Days, NaiveDate};
    use proptest::collection::vec;
    use proptest::option;
    use proptest::prelude::*;

    fn arb_daily_tss() -> impl Strategy<Value = DailyTSS> {
        ((0..100u64), (100..300i64)).prop_map(|(days, tss)| {
            DailyTSS(
                NaiveDate::from_ymd_opt(2023, 10, 7).unwrap() + Days::new(days),
                TSS(tss),
            )
        })
    }

    prop_compose! {
        fn arb_daily_stats()
        (
            days in (0..100u64),
            tss in (100..300i64),
            ctl in (0.0..60.0f64),
            atl in (0.0..100.0f64),
            tsb in (-40.0..40.0f64),
        )
        -> DailyStats {
            DailyStats {
                date: NaiveDate::from_ymd_opt(2023, 10, 7).unwrap() + Days::new(days),
                tss: TSS(tss),
                ctl: CTL(ctl),
                atl: ATL(atl),
                tsb: TSB(tsb),
            }
        }
    }

    proptest! {
        #[test]
        fn daily_tss_is_sorted(daily_tss_vec in vec(arb_daily_tss(), 20)) {

            let SortedDailyTSS(sorted) = SortedDailyTSS::from_unsorted(&daily_tss_vec, None);
            let dates = sorted.iter().map(|DailyTSS(date, _)| date).collect::<Vec<_>>();

            let mut expected = dates.clone();
            expected.sort();

            assert_eq!(dates, expected);
        }
    }

    proptest! {
        #[test]
        fn daily_tss_is_filled(
            daily_stats in option::of(arb_daily_stats()),
            daily_tss_vec in vec(arb_daily_tss(), 20)
            ) {

            let SortedDailyTSS(sorted) = SortedDailyTSS::from_unsorted(&daily_tss_vec, daily_stats.as_ref());
            let dates = sorted.iter().map(|DailyTSS(date, _)| date).collect::<Vec<_>>();

            assert!(dates.windows(2).all(|days| *days[0] + Days::new(1) == *days[1]));

            if let (Some(stats), Some(first_date)) = (daily_stats, dates.first()) {
                assert_eq!(stats.date + Days::new(1) , **first_date);
            }
        }
    }

    proptest! {
        #[test]
        fn daily_stats_is_at_least_as_long_as_input(daily_stats in option::of(arb_daily_stats()), daily_tss_vec in vec(arb_daily_tss(), 50)) {
            let sorted = SortedDailyTSS::from_unsorted(&daily_tss_vec, None);
            let daily_stats = DailyStats::calc_rolling(sorted.clone(), daily_stats.as_ref() );
            assert_ge!(daily_stats.len(), sorted.0.len());
        }
    }

    proptest! {
        #[test]
        fn daily_stats_is_extended(daily_stats in option::of(arb_daily_stats()), daily_tss_vec in vec(arb_daily_tss(), 50)) {
            let sorted = SortedDailyTSS::from_unsorted(&daily_tss_vec, daily_stats.as_ref());
            // Occasionally the base case (DailyStats) comes after all the TSS data, resulting
            // in an empty sorted vector
            prop_assume!(!sorted.0.is_empty());

            let daily_stats = DailyStats::calc_rolling(sorted, daily_stats.as_ref());

            let last = daily_stats.last().unwrap();
            assert_le!(last.ctl, CTL(0.5));
            assert_le!(last.atl, ATL(0.5));
            assert_le!(last.tsb, TSB(0.5));

        }
    }
}
