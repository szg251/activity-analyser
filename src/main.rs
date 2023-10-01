use activity::Activity;
use fitparser::{self, Error};
use std::fs::File;

mod activity;

fn main() -> Result<(), Error> {
    println!(
        "Parsing FIT files using Profile version: {}",
        fitparser::profile::VERSION
    );
    let mut fp =
        File::open("../../Cycling/FitFiles/Wahoo_SYSTM_N_Henderson_2_Rabbit_Mountain.fit")?;
    let activity = Activity::from_reader(&mut fp)?;
    println!("{:#?}", activity);
    Ok(())
}
