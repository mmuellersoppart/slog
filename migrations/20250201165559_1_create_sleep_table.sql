-- Add migration script here
/*
┌────────────────┬───────────────┬────────────────┬─────────────────┐
│    quality     │    stress     │    exertion    │      mood       │
├────────────────┼───────────────┼────────────────┼─────────────────┤
│ -2 devastation │ true stressed │ -1 did nothing │ -1 sad/moody    │
│ -1 terrible    │ false calm    │ 0 normal       │ 0 neutral/calm  │
│ 0 blah         │               │ 1 exhausted    │ 1 exuberant     │
│ 1 okay         │               │                │                 │
│ 2 perfection   │               │                │                 │
└────────────────┴───────────────┴────────────────┴─────────────────┘
 */

CREATE TABLE IF NOT EXISTS sleep
(
    id      INTEGER primary key AUTOINCREMENT NOT NULL,
    start   TEXT NOT NULL,
    end     TEXT NULL DEFAULT NULL,
    total_time_hours TEXT GENERATED ALWAYS AS ((JULIANDAY(end) - JULIANDAY(start)) * 24) STORED,
    awake_count INTEGER NULL DEFAULT 0,
    quality INTEGER DEFAULT 0,
    is_sleep_ritual BOOLEAN NULL DEFAULT TRUE,
    is_stress BOOLEAN NULL DEFAULT FALSE,
    mood INTEGER DEFAULT 0,
    is_heartburn BOOLEAN NULL DEFAULT FALSE,
    is_ibs_flareup BOOLEAN NULL DEFAULT FALSE,  -- the day of
    melatonin FLOAT DEFAULT NULL, -- mg
    exertion INTEGER NULL DEFAULT NULL,
    UNIQUE(start, end)
);