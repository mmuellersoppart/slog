# CLI Sleep Log (slog)

A command-line application for tracking and logging sleep data.

## Features

- Record sleep sessions with detailed metrics
- Track sleep quality, mood, exertion levels
- Monitor sleep interruptions and factors (stress, heartburn, melatonin usage, etc.)
- Configurable default values via YAML config file
- SQLite database for persistent storage

## Installation

```bash
cargo build --release
```

## File Locations

### Configuration File
- **Path**: `~/.config/slog/config.yml`
- **Auto-created**: Yes, on first run with default values

### Database
- **Default Path**: `~/.local/share/slog/slog.db`
- **Configurable**: Yes, via config file

## Configuration

The configuration file (`~/.config/slog/config.yml`) contains the following fields:

```yaml
start_time_default: '21:30:00'
end_time_default: '05:30:00'
db_file_path: /Users/yourusername/.local/share/slog/slog.db
google_sheets_id: null
google_credentials_path: null
```

### Configuration Fields

- **`start_time_default`**: Default start time for sleep sessions (format: HH:MM:SS)
- **`end_time_default`**: Default end time for sleep sessions (format: HH:MM:SS)
- **`db_file_path`**: Full path to the SQLite database file
- **`google_sheets_id`**: Google Sheets spreadsheet ID for data export (optional)
- **`google_credentials_path`**: Path to Google service account credentials JSON file (optional)

## Usage

### Record Sleep Data

Record a new sleep session (uses config defaults):

```bash
slog
# or explicitly
slog record
```

This will prompt you for:
- Date
- Start time (defaults to `start_time_default`)
- Minutes to fall asleep
- Number of times woken up
- Total time awake (minutes)
- End time (defaults to `end_time_default`)
- Time in bed after waking (minutes)
- Sleep quality (Devastation, Terrible, Blah, Okay, Perfection)
- Melatonin usage (mg)
- Benadryl usage (mg)
- Edible usage (mg)
- Exertion level (Lazy, Normal, Exhausted)

### View Configuration

Show current configuration settings:

```bash
slog show-config
```

### Edit Configuration

Update configuration fields:

```bash
# Change start time default
slog config start_time_default 22:00:00

# Change end time default
slog config end_time_default 06:00:00

# Change database file path
slog config db_file_path /path/to/your/custom/db
```

You can also manually edit `~/.config/slog/config.yml` with any text editor.

### Export to Google Sheets

Export all sleep data to a Google Sheets spreadsheet:

```bash
slog export
```

**Setup Requirements:**

1. Create a Google Cloud Project and enable the Google Sheets API
2. Create a Service Account and download the credentials JSON file
3. Share your Google Sheet with the service account email (found in the credentials JSON)
4. Configure the app with your spreadsheet ID and credentials path:

```bash
# Set the Google Sheets ID (found in the sheet URL)
slog config google_sheets_id YOUR_SPREADSHEET_ID

# Set the path to your service account credentials
slog config google_credentials_path /path/to/credentials.json
```

The export will replace all data in Sheet1 of the spreadsheet with the current database contents.

## Data Tracked

Each sleep session records:
- Start and end timestamps
- Minutes to fall asleep
- Wake-up count
- Total time awake (minutes)
- Time in bed after waking (minutes)
- Sleep quality rating (-2 to 2)
- Melatonin dosage
- Benadryl dosage
- Edible dosage
- Physical exertion level (-1 to 1)

## Database Schema

The SQLite database contains a `sleep` table with all tracked metrics. Migrations are automatically applied on startup.

## release
`cargo build release`
`cp ./target/debug/slog ~/.local/bin/slog`