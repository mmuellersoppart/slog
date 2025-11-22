mod config;
// mod sheets;

use std::fmt;
use std::str::FromStr;

use tokio::runtime::Runtime;

use sqlx::ConnectOptions;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};

use chrono::{Datelike, Days, Local, NaiveDate, NaiveDateTime, Utc};
use std::fmt::{Display, Formatter};

use inquire::formatter::CustomTypeFormatter;
use inquire::{
    Confirm, CustomType, DateSelect, MultiSelect, Select, Text,
    error::{CustomUserError, InquireResult},
    required,
};

use strum::EnumIter;
use strum::IntoEnumIterator;

use clap::{Parser, Subcommand};

use config::Config;
// use sheets::SheetsExporter;

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
        /// Configuration field to edit (start_time_default, end_time_default, db_file_path, google_sheets_id, google_credentials_path)
        field: String,
        /// New value for the field
        value: String,
    },
    /// Show current configuration
    ShowConfig,
    // /// Export all data to Google Sheets
    // Export,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Config { field, value }) => {
            edit_config(field, value)?;
        }
        Some(Commands::ShowConfig) => {
            show_config()?;
        }
        // Some(Commands::Export) => {
        //     export_to_sheets().await?;
        // }
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
    println!(
        "  google_sheets_id: {}",
        config
            .google_sheets_id
            .as_ref()
            .unwrap_or(&"Not set".to_string())
    );
    println!(
        "  google_credentials_path: {}",
        config
            .google_credentials_path
            .as_ref()
            .unwrap_or(&"Not set".to_string())
    );
    Ok(())
}

// async fn export_to_sheets() -> Result<(), Box<dyn std::error::Error>> {
//     let config = Config::load()?;
//
//     // Check if Google Sheets is configured
//     let sheets_id = config.google_sheets_id.as_ref().ok_or(
//         "Google Sheets ID not configured. Use: slog config google_sheets_id YOUR_SHEET_ID",
//     )?;
//     let credentials_path = config.google_credentials_path.as_ref()
//         .ok_or("Google credentials path not configured. Use: slog config google_credentials_path /path/to/credentials.json")?;
//
//     // Connect to database
//     let opts = SqliteConnectOptions::from_str(&config.get_db_url())?.create_if_missing(false);
//     let pool = SqlitePool::connect_with(opts).await?;
//
//     // Create exporter and export
//     println!("Connecting to Google Sheets...");
//     let exporter = SheetsExporter::new(credentials_path, sheets_id.clone()).await?;
//
//     println!("Exporting data...");
//     exporter.export_all_data(&pool).await?;
//
//     println!("Successfully exported all data to Google Sheets!");
//     Ok(())
// }

async fn record_sleep() -> Result<(), Box<dyn std::error::Error>> {
    // Display welcome message
    println!("\n╔═══════════════════════════════════╗");
    println!("║  🥱💤 SLOG (The Sleep Log) 💤🥱   ║");
    println!("╚═══════════════════════════════════╝");
    println!(
        "                                v{}\n",
        env!("CARGO_PKG_VERSION")
    );

    // Load config
    let config = Config::load()?;

    // Ensure database directory exists
    let db_path = std::path::Path::new(&config.db_file_path);
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // creates database if doesn't exist
    let opts = SqliteConnectOptions::from_str(&config.get_db_url())?.create_if_missing(true);

    let pool = SqlitePool::connect_with(opts)
        .await
        .expect("Failed to connect with db");

    let _ = sqlx::migrate!().run(&pool).await;

    let now = Local::now()
        .fixed_offset()
        .checked_sub_days(Days::new(1))
        .unwrap();
    let start_date: NaiveDate = DateSelect::new("Date:")
        .with_default(
            NaiveDate::from_ymd_opt(now.year(), now.month(), now.day())
                .expect("Failed to get start date"),
        )
        .prompt()?;
    let start_date_str = start_date.format("%Y-%m-%d").to_string();
    let end_date = start_date.checked_add_days(Days::new(1)).unwrap();
    let end_date_str = end_date.format("%Y-%m-%d").to_string();

    let _start_time_input: String = Text::new("Start Time (HH:MM)")
        .with_default(&config.start_time_default)
        .prompt()?;
    let _start_time = format!("{}:00", _start_time_input);
    let start = format!("{start_date_str} {_start_time}");

    // Check for existing entries on this date
    let existing_query = "SELECT id, start, end FROM sleep WHERE DATE(start) = DATE(?)";
    let existing: Option<(i64, String, Option<String>)> = sqlx::query_as(existing_query)
        .bind(&start)
        .fetch_optional(&pool)
        .await?;

    if let Some((existing_id, existing_start, existing_end)) = existing {
        println!("\n⚠️  An entry already exists for this date:");
        println!("   Start: {}", existing_start);
        if let Some(end) = existing_end {
            println!("   End: {}", end);
        }

        let should_delete =
            Confirm::new("Do you want to delete the existing entry and create a new one?")
                .with_default(false)
                .prompt()?;

        if should_delete {
            let delete_sql = "DELETE FROM sleep WHERE id = ?";
            sqlx::query(delete_sql)
                .bind(existing_id)
                .execute(&pool)
                .await?;
            println!("✓ Existing entry deleted.");
        } else {
            println!("Cancelled. No changes made.");
            return Ok(());
        }
    }

    let minutes_to_fall_asleep =
        CustomType::<i32>::new("How many minutes did it take you to fall asleep?")
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

    let time_awake = CustomType::<i32>::new(
        "How long were you awake if you add together the # of minutes awake?",
    )
    .with_error_message("Please type a valid number")
    .with_help_message("Type a number in minutes")
    .with_default(0)
    .prompt()
    .expect("Failed");

    let _end_time_input: String = Text::new("End Time (HH:MM)")
        .with_default(&config.end_time_default)
        .prompt()?;
    let _end_time = format!("{}:00", _end_time_input);
    let end = format!("{end_date_str} {_end_time}");

    let time_in_bed_after_waking =
        CustomType::<i32>::new("How long did you lie in bed after waking? (minutes)")
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
            // Calculate total sleep time and efficiency
            let start_dt = NaiveDateTime::parse_from_str(&start, "%Y-%m-%d %H:%M:%S")?;
            let end_dt = NaiveDateTime::parse_from_str(&end, "%Y-%m-%d %H:%M:%S")?;
            let total_time_in_bed = (end_dt - start_dt).num_minutes();
            let total_sleep_minutes =
                total_time_in_bed - minutes_to_fall_asleep as i64 - time_awake as i64 - time_in_bed_after_waking as i64;
            let sleep_efficiency = (total_sleep_minutes as f64 / total_time_in_bed as f64) * 100.0;

            println!("Sleep data recorded successfully!");
            println!("\n=== Sleep Summary ===");
            println!(
                "Total time in bed: {} minutes ({:.1} hours)",
                total_time_in_bed,
                total_time_in_bed as f64 / 60.0
            );
            println!("Sleep efficiency: {:.1}%", sleep_efficiency);
        }
        Err(e) => {
            eprintln!("Error executing SQL: {}", e);
            eprintln!("SQL: {}", sql);
            eprintln!(
                "Parameters: start={}, minutes_to_fall_asleep={}, end={}, awake_count={}, time_awake={}, time_in_bed_after_waking={}, quality={}, melatonin={}, benadryl={}, edible={}, exertion={}",
                start,
                minutes_to_fall_asleep,
                end,
                awake_count,
                time_awake,
                time_in_bed_after_waking,
                quality.db_value(),
                melatonin,
                benadryl,
                edible,
                exertion.db_value()
            );
            return Err(Box::new(e));
        }
    }

    Ok(())
}
