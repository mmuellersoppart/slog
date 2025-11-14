mod config;

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

use clap::{Parser, Subcommand};

use config::Config;


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

#[derive(Parser)]
#[command(name = "slog")]
#[command(about = "CLI Sleep Log - Track your sleep data", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Record new sleep data
    Record,
    /// Edit configuration settings
    Config {
        /// Configuration field to edit (start_time_default, end_time_default, db_location)
        field: String,
        /// New value for the field
        value: String,
    },
    /// Show current configuration
    ShowConfig,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>  {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Config { field, value }) => {
            edit_config(field, value)?;
        }
        Some(Commands::ShowConfig) => {
            show_config()?;
        }
        Some(Commands::Record) | None => {
            record_sleep().await?;
        }
    }

    Ok(())
}

fn edit_config(field: String, value: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::load()?;
    config.update_field(&field, value)?;
    config.save()?;
    println!("Updated {} successfully!", field);
    Ok(())
}

fn show_config() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    println!("Current configuration:");
    println!("  Config file: {}", Config::config_path().display());
    println!("  start_time_default: {}", config.start_time_default);
    println!("  end_time_default: {}", config.end_time_default);
    println!("  db_file_path: {}", config.db_file_path);
    Ok(())
}

async fn record_sleep() -> Result<(), Box<dyn std::error::Error>> {
    // Load config
    let config = Config::load()?;

    // Ensure database directory exists
    let db_path = std::path::Path::new(&config.db_file_path);
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // creates database if doesn't exist
    let opts = SqliteConnectOptions::from_str(&config.get_db_url())?
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
        .with_default(&config.start_time_default)
        .prompt()?;
    let start = format!("{start_date_str} {_start_time}");

    let minutes_to_fall_asleep = CustomType::<i32>::new("How many minutes did it take you to fall asleep?")
        .with_error_message("Please type a valid number")
        .with_help_message("Type a number")
        .with_default(0)
        .prompt()
        .expect("Failed");

    let awake_count = CustomType::<i16>::new("How many times did you wake up minus one?")
        .with_error_message("Please type a valid number")
        .with_help_message("Type a number")
        .with_default(0)
        .prompt()
        .expect("Failed");

    let time_awake = CustomType::<i32>::new("How long were you awake if you add together the # of minutes awake?")
        .with_error_message("Please type a valid number")
        .with_help_message("Type a number in minutes")
        .with_default(0)
        .prompt()
        .expect("Failed");

    let _end_time: String = Text::new("End Time: in HH:MM:SS")
        .with_default(&config.end_time_default)
        .prompt()?;
    let end = format!("{end_date_str} {_end_time}");

    let time_in_bed_after_waking = CustomType::<i32>::new("How long did you lie in bed after waking? (minutes)")
        .with_error_message("Please type a valid number")
        .with_help_message("Type a number in minutes")
        .with_default(0)
        .prompt()
        .expect("Failed");

    let quality_options: Vec<Quality> = Quality::iter().collect();
    let quality = Select::new("Quality", quality_options)
        .with_starting_cursor(3)
        .prompt()
        .expect("failed to read Quality prompt.");

    let melatonin = CustomType::<f32>::new("How much melatonin did you use? (mg)")
        .with_error_message("Please type a valid number")
        .with_help_message("Type a number")
        .with_default(0.0)
        .prompt()
        .expect("Failed");

    let benadryl = CustomType::<f32>::new("How much benadryl did you use? (mg)")
        .with_error_message("Please type a valid number")
        .with_help_message("Type a number")
        .with_default(0.0)
        .prompt()
        .expect("Failed");

    let edible = CustomType::<f32>::new("How much edible did you use? (mg)")
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

    let sql = "INSERT INTO sleep (start, minutes_to_fall_asleep, end, awake_count, time_awake, time_in_bed_after_waking, quality, melatonin, benadryl, edible, exertion)
        VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)";

    let result = sqlx::query(sql)
        .bind(&start)
        .bind(minutes_to_fall_asleep)
        .bind(&end)
        .bind(awake_count)
        .bind(time_awake)
        .bind(time_in_bed_after_waking)
        .bind(quality.db_value())
        .bind(melatonin)
        .bind(benadryl)
        .bind(edible)
        .bind(exertion.db_value())
        .execute(&pool)
        .await;

    match result {
        Ok(_) => {
            println!("Sleep data recorded successfully!");
        }
        Err(e) => {
            eprintln!("Error executing SQL: {}", e);
            eprintln!("SQL: {}", sql);
            eprintln!("Parameters: start={}, minutes_to_fall_asleep={}, end={}, awake_count={}, time_awake={}, time_in_bed_after_waking={}, quality={}, melatonin={}, benadryl={}, edible={}, exertion={}",
                start, minutes_to_fall_asleep, end, awake_count, time_awake, time_in_bed_after_waking, quality.db_value(), melatonin, benadryl, edible, exertion.db_value());
            return Err(Box::new(e));
        }
    }

    Ok(())
}
