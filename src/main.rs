use std::fmt;
use std::str::FromStr;

use tokio::runtime::Runtime;

use sqlx::ConnectOptions;
use sqlx::sqlite::{SqlitePool, SqliteConnectOptions};

use std::fmt::{Display, Formatter};
use chrono::{Datelike, Days, Local, NaiveDate, Utc};

use inquire::{error::{CustomUserError, InquireResult}, required, CustomType, DateSelect, MultiSelect, Select, Text, Confirm};
use inquire::formatter::CustomTypeFormatter;

use strum::IntoEnumIterator;
use strum::EnumIter;


// #[derive(Debug, sqlx::FromRow)]
// struct SleepLog {
//     id: Option<i32>,
//     start: String,
//     end: Option<String>,
//     awake_count: Option<i32>,
//     quality: Option<i32>,
//     is_sleep_ritual: Option<bool>,
//     is_stress: Option<bool>,
//     mood: Option<i32>,
//     is_heartburn: Option<bool>,
//     is_ibs_flareup: Option<bool>,
//     melatonin: Option<f32>,
//     exertion: Option<i32>,
// }
//
// struct CreateSleepLog {
//     start: String,
//     end: Option<String>,
//     awake_count: Option<i32>,
//     quality: Option<i32>,
//     is_sleep_ritual: Option<bool>,
//     is_stress: Option<bool>,
//     mood: Option<i32>,
//     is_heartburn: Option<bool>,
//     is_ibs_flareup: Option<bool>,
//     melatonin: Option<f32>,
//     exertion: Option<i32>,
// }

// impl CreateSleepLog {
//     fn new(
//         start: String,
//         end: Option<String>,
//         awake_count: Option<i32>,
//         quality: Option<i32>,
//         is_sleep_ritual: Option<bool>,
//         is_stress: Option<bool>,
//         mood: Option<i32>,
//         is_heartburn: Option<bool>,
//         is_ibs_flareup: Option<bool>,
//         melatonin: Option<f32>,
//         exertion: Option<i32>,
//     ) -> CreateSleepLog {
//         CreateSleepLog {
//             start,
//             end,
//             awake_count,
//             quality,
//             is_sleep_ritual,
//             is_stress,
//             mood,
//             is_heartburn,
//             is_ibs_flareup,
//             melatonin,
//             exertion,
//         }
//     }
// }

#[derive(Debug, EnumIter, strum_macros::Display)]
enum Quality {
    Devastation,
    Terrible,
    Blah,
    Okay,
    Perfection,
}


impl Quality {
    fn db_value(&self) -> i8 {
        match self {
            Quality::Devastation => -2,
            Quality::Terrible => -1,
            Quality::Blah => 0,
            Quality::Okay => 1,
            Quality::Perfection => 2,
        }
    }
}

#[derive(Debug, EnumIter, strum_macros::Display)]
enum Mood {
    Sad,
    Neutral,
    Exuberant,
}


impl Mood {
    fn db_value(&self) -> i8 {
        match self {
            Mood::Sad => -1,
            Mood::Neutral => 0,
            Mood::Exuberant => 1,
        }
    }
}

#[derive(Debug, EnumIter, strum_macros::Display)]
enum Exertion {
    Lazy,
    Normal,
    Exhausted,
}


impl Exertion {
    fn db_value(&self) -> i8 {
        match self {
            Exertion::Lazy => -1,
            Exertion::Normal => 0,
            Exertion::Exhausted => 1,
        }
    }
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    // creates database if doesn't exist
    let opts = SqliteConnectOptions::from_str("sqlite:/Users/marlonmuellersoppart/.sleep_log/db")?
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(opts).await.expect("Failed to connect with db");

    let _ = sqlx::migrate!().run(&pool).await;

    let now = Local::now().fixed_offset().checked_sub_days(Days::new(1)).unwrap();
    let start_date: NaiveDate = DateSelect::new("Date:")
        .with_default(NaiveDate::from_ymd_opt(
            now.year(),
            now.month(),
            now.day())
            .expect("Failed to get start date")
        )
        .prompt()?;
    let start_date_str = start_date.format("%Y-%m-%d").to_string();
    let end_date = start_date.checked_add_days(Days::new(1)).unwrap();
    let end_date_str = end_date.format("%Y-%m-%d").to_string();

    let _start_time: String = Text::new("Start Time: in HH:MM:SS")
        .with_default("21:30:00")
        .prompt()?;
    let start = format!("{start_date_str} {_start_time}");

    let _end_time: String = Text::new("End Time: in HH:MM:SS")
        .with_default("05:30:00")
        .prompt()?;
    let end = format!("{end_date_str} {_end_time}");

    let awake_count = CustomType::<i16>::new("How many times did you wake up minus one?")
        .with_error_message("Please type a valid number")
        .with_help_message("Type a number")
        .with_default(0)
        .prompt()
        .expect("Failed");

    let quality_options: Vec<Quality> = Quality::iter().collect();
    let quality = Select::new("Quality", quality_options)
        .with_starting_cursor(3)
        .prompt()
        .expect("failed to read Quality prompt.");

    let is_sleep_ritual = Confirm::new("sleep ritual?")
        .with_default(true)
        .prompt()
        .unwrap();

    let is_stress = Confirm::new("stressed?")
        .with_default(false)
        .prompt()
        .unwrap();

    let mood_options: Vec<Mood> = Mood::iter().collect();
    let mood: Mood = Select::new("Mood", mood_options)
        .with_starting_cursor(1)
        .prompt()
        .expect("failed to read Mood prompt.");

    let is_heartburn = Confirm::new("heartburn?")
        .with_default(false)
        .prompt()
        .unwrap();

    let is_ibs_flareup = Confirm::new("is_ibs_flareup?")
        .with_default(false)
        .prompt()
        .unwrap();

    let melatonin = CustomType::<f32>::new("How much melatonin did you use?")
        .with_error_message("Please type a valid number")
        .with_help_message("Type a number")
        .with_default(0.0)
        .prompt()
        .expect("Failed");

    let exertion_options: Vec<Exertion> = Exertion::iter().collect();
    let exertion: Exertion = Select::new("Exertion", exertion_options)
        .with_starting_cursor(1)
        .prompt()
        .expect("failed to read Exertion prompt.");

    let result = sqlx::query(
        "INSERT INTO sleep (start, end, awake_count, quality, is_sleep_ritual, is_stress, mood, is_heartburn, is_ibs_flareup, melatonin, exertion)
        VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)")
        .bind(start)
        .bind(end)
        .bind(awake_count)
        .bind(quality.db_value())
        .bind(is_sleep_ritual)
        .bind(is_stress)
        .bind(mood.db_value())
        .bind(is_heartburn)
        .bind(is_ibs_flareup)
        .bind(melatonin)
        .bind(exertion.db_value())
        .execute(&pool).await?;

    Ok(())
}
