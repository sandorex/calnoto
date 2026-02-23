//! Example reading calendar entries from file

use chrono::NaiveDateTime;
use icalendar_txt::{CalendarEntry, Interval};

const DEFAULT_FILE: &str = "~/icalendar.txt";

fn get_recurring_entry_date(entry: &CalendarEntry, target: NaiveDateTime) -> Option<CalendarEntry> {
    let mut current = entry.interval.start.clone();
    let offset = entry.interval.end.signed_duration_since(entry.interval.start);
    let interval = entry.interval.interval.as_ref().unwrap();

    while current.date() < target.date() {
        let start = current;
        let end = current + offset;

        if start < target && target < end {
            return Some(CalendarEntry {
                interval: Interval {
                    start: current,
                    end: current + entry.interval.end.signed_duration_since(entry.interval.start),
                    interval: entry.interval.interval.clone(),
                },
                description: entry.description.clone(),
                metadata: entry.metadata.clone(),
            });
        }

        // advance by the interval
        current = (current + interval)?;
    }

    None
}

fn format_event(entry: &CalendarEntry, now: NaiveDateTime) -> String {
    // NOTE event is whole day if the start date and end dates arent the current date
    let time = if now.date() != entry.interval.start.date() && now.date() != entry.interval.end.date() {
        "     all day    ".to_string()
    } else {
        let start = if now.date() != entry.interval.start.date() {
            "        ".to_string()
        } else {
            format!("{}", entry.interval.start.time())
        };

        let end = if now.date() != entry.interval.end.date() {
            "        ".to_string()
        } else {
            format!("{}", entry.interval.end.time())
        };

        format!("{start} - {end}")
    };

    format!("{}({time}) {}", if entry.interval.interval.is_some() { "R " } else { "  " }, entry.description)
}

fn main() {
    let args: Vec<_> = std::env::args().collect();

    // use the provided file or default file
    let file = if let Some(x) = args.get(1) {
        x.as_str()
    } else if std::fs::exists(DEFAULT_FILE).unwrap_or_default() {
        DEFAULT_FILE
    } else {
        eprintln!("Could not find {}", DEFAULT_FILE);
        std::process::exit(1);
    };

    let contents = match std::fs::read_to_string(file) {
        Ok(x) => x,
        Err(err) => {
            eprintln!("Could not read {file:?}: {err}");
            std::process::exit(1);
        }
    };

    let now = chrono::Local::now().naive_local();

    let mut entries: Vec<CalendarEntry> = vec![];
    match CalendarEntry::from_str(&contents) {
        Ok(raw_entries) => for entry in raw_entries {
            if entry.interval.interval.is_some() {
                if let Some(x) = get_recurring_entry_date(&entry, now) {
                    entries.push(x);
                }
            } else {
                if entry.interval.start.date() <= now.date() && entry.interval.end.date() >= now.date() {
                    entries.push(entry);
                }
            }
        },
        Err(err) => {
            eprintln!("Error parsing {file:?}: {err}");
            std::process::exit(1);
        }
    };

    // sort by start date
    entries.sort_by(|a, b| {
        a.interval.start.cmp(&b.interval.start)
    });

    println!("Today's agenda:");
    for entry in entries {
        println!(" {}", format_event(&entry, now));
    }
}
