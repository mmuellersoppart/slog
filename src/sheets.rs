use google_sheets4::{
    api::{ValueRange, ClearValuesRequest},
    hyper, hyper_rustls, oauth2, Sheets,
};
use sqlx::SqlitePool;
use std::error::Error;

pub struct SheetsExporter {
    hub: Sheets<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    spreadsheet_id: String,
}

impl SheetsExporter {
    pub async fn new(
        credentials_path: &str,
        spreadsheet_id: String,
    ) -> Result<Self, Box<dyn Error>> {
        let secret = oauth2::read_service_account_key(credentials_path).await?;
        let auth = oauth2::ServiceAccountAuthenticator::builder(secret)
            .build()
            .await?;

        let hub = Sheets::new(
            hyper::Client::builder().build(
                hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .https_or_http()
                    .enable_http1()
                    .build(),
            ),
            auth,
        );

        Ok(SheetsExporter {
            hub,
            spreadsheet_id,
        })
    }

    pub async fn export_all_data(&self, pool: &SqlitePool) -> Result<(), Box<dyn Error>> {
        // Fetch all data from database
        let rows = sqlx::query!(
            r#"
            SELECT
                id,
                start,
                minutes_to_fall_asleep,
                end,
                total_time_hours,
                awake_count,
                time_awake,
                time_in_bed_after_waking,
                quality,
                melatonin,
                benadryl,
                edible,
                exertion
            FROM sleep
            ORDER BY start DESC
            "#
        )
        .fetch_all(pool)
        .await?;

        // Prepare data for sheets
        let mut values: Vec<Vec<String>> = Vec::new();

        // Header row
        values.push(vec![
            "ID".to_string(),
            "Start".to_string(),
            "Minutes to Fall Asleep".to_string(),
            "End".to_string(),
            "Total Hours".to_string(),
            "Awake Count".to_string(),
            "Time Awake (min)".to_string(),
            "Time in Bed After Waking (min)".to_string(),
            "Quality".to_string(),
            "Melatonin (mg)".to_string(),
            "Benadryl (mg)".to_string(),
            "Edible (mg)".to_string(),
            "Exertion".to_string(),
        ]);

        // Data rows
        for row in rows {
            values.push(vec![
                row.id.to_string(),
                row.start.unwrap_or_default(),
                row.minutes_to_fall_asleep.map(|v| v.to_string()).unwrap_or_default(),
                row.end.unwrap_or_default(),
                row.total_time_hours.unwrap_or_default(),
                row.awake_count.map(|v| v.to_string()).unwrap_or_default(),
                row.time_awake.map(|v| v.to_string()).unwrap_or_default(),
                row.time_in_bed_after_waking.map(|v| v.to_string()).unwrap_or_default(),
                row.quality.map(|v| v.to_string()).unwrap_or_default(),
                row.melatonin.map(|v| v.to_string()).unwrap_or_default(),
                row.benadryl.map(|v| v.to_string()).unwrap_or_default(),
                row.edible.map(|v| v.to_string()).unwrap_or_default(),
                row.exertion.map(|v| v.to_string()).unwrap_or_default(),
            ]);
        }

        // Clear existing data in the sheet
        let range = "Sheet1!A:M";
        let _clear_result = self
            .hub
            .spreadsheets()
            .values_clear(ClearValuesRequest::default(), &self.spreadsheet_id, range)
            .doit()
            .await?;

        // Write new data
        let value_range = ValueRange {
            major_dimension: Some("ROWS".to_string()),
            range: Some(range.to_string()),
            values: Some(values.into_iter().map(|row| {
                row.into_iter().map(|cell| serde_json::Value::String(cell)).collect()
            }).collect()),
        };

        let _result = self
            .hub
            .spreadsheets()
            .values_update(value_range, &self.spreadsheet_id, range)
            .value_input_option("RAW")
            .doit()
            .await?;

        Ok(())
    }
}
