-- Add migration script here
/*
┌────────────────┬────────────────┬─────────────────┐
│    quality     │   exertion     │   melatonin     │
├────────────────┼────────────────┼─────────────────┤
│ -2 devastation │ -1 did nothing │   float (mg)    │
│ -1 terrible    │ 0 normal       │                 │
│ 0 blah         │ 1 exhausted    │                 │
│ 1 okay         │                │                 │
│ 2 perfection   │                │                 │
└────────────────┴────────────────┴─────────────────┘
 */

CREATE TABLE IF NOT EXISTS sleep
(
    id      INTEGER primary key AUTOINCREMENT NOT NULL,
    start   TEXT NOT NULL,
    minutes_to_fall_asleep INTEGER NULL DEFAULT 0,
    end     TEXT NULL DEFAULT NULL,
    total_time_hours TEXT GENERATED ALWAYS AS ((JULIANDAY(end) - JULIANDAY(start)) * 24) STORED,
    awake_count INTEGER NULL DEFAULT 0,
    time_awake INTEGER NULL DEFAULT 0,
    time_in_bed_after_waking INTEGER NULL DEFAULT 0,
    quality INTEGER DEFAULT 0,
    melatonin FLOAT DEFAULT NULL, -- mg
    benadryl FLOAT DEFAULT 0, -- mg
    edible FLOAT DEFAULT 0, -- mg
    exertion INTEGER NULL DEFAULT NULL,
    UNIQUE(start, end)
);
