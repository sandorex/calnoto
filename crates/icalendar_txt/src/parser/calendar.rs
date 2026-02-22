use nom::{Parser, character::complete::{multispace0, space1}, combinator::{all_consuming, consumed}, multi::separated_list0, sequence::preceded};
use crate::entry::CalendarEntry;
use crate::error::prelude::*;
use super::iso8601::parse_interval;
use super::parse_description;

pub fn parse_calendar_entry(input: &str) -> IResult<&str, CalendarEntry> {
    let (leftover, (interval, (description, metadata))) = (
        parse_interval,
        preceded(space1, consumed(parse_description)),
    ).parse(input)?;

    // prevent empty description
    if description.trim().is_empty() {
        return Err(nom::Err::Failure(Error { input, kind: ErrorKind::EmptyDescription, context: None }))
    }

    Ok((leftover, CalendarEntry {
        interval,
        description: description.to_string(),
        metadata,
    }))
}

/// Parse file of calendar entries
pub fn parse_calendar_file(input: &str) -> IResult<&str, Vec<CalendarEntry>> {
    let (leftover, entries) = all_consuming(separated_list0(
        multispace0,
        parse_calendar_entry,
    )).parse(input.trim())?;

    Ok((leftover, entries))
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    use crate::{Interval};
    use crate::entry::EntryMetadata;

    use super::*;

    #[test]
    fn test_parse_calendar_entry() {
        assert_eq!(parse_calendar_entry("2020-01-01T20:00 Hello there"), Ok(("", CalendarEntry {
            interval: Interval {
                start: NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                    NaiveTime::from_hms_opt(20, 0, 0).unwrap()
                ),
                end: NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(2020, 1, 2).unwrap(),
                    NaiveTime::from_hms_opt(0, 0, 0).unwrap()
                ),
                interval: None,
            },
            description: "Hello there".to_string(),
            metadata: None,
        })));

        assert_eq!(parse_calendar_entry("2020-01-01T20:00     Hello there"), Ok(("", CalendarEntry {
            interval: Interval {
                start: NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                    NaiveTime::from_hms_opt(20, 0, 0).unwrap()
                ),
                end: NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(2020, 1, 2).unwrap(),
                    NaiveTime::from_hms_opt(0, 0, 0).unwrap()
                ),
                interval: None,
            },
            description: "Hello there".to_string(),
            metadata: None,
        })));

        assert_eq!(parse_calendar_entry("2020-01-01T20:00\t\tHello there"), Ok(("", CalendarEntry {
            interval: Interval {
                start: NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                    NaiveTime::from_hms_opt(20, 0, 0).unwrap()
                ),
                end: NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(2020, 1, 2).unwrap(),
                    NaiveTime::from_hms_opt(0, 0, 0).unwrap()
                ),
                interval: None,
            },
            description: "Hello there".to_string(),
            metadata: None,
        })));

        assert_eq!(parse_calendar_entry("2020-01-01T20:00/2020-01-01T23:00 Hello there +blah"), Ok(("", CalendarEntry {
            interval: Interval {
                start: NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                    NaiveTime::from_hms_opt(20, 0, 0).unwrap()
                ),
                end: NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                    NaiveTime::from_hms_opt(23, 0, 0).unwrap()
                ),
                interval: None,
            },
            description: "Hello there +blah".to_string(),
            metadata: Some(EntryMetadata {
                description: "Hello there blah".to_string(),
                projects: vec!["blah".to_string()],
                ..Default::default()
            }),
        })));

        // malformed entry
        assert_eq!(
            parse_calendar_entry("2020-\n01-01/2020-01-01T23:00 Hello there +blah"),
            Err(nom::Err::Failure(Error { input: "\n01-01/2020-01-01T23:00 Hello there +blah", kind: ErrorKind::Nom(NomErrorKind::Digit), context: None }))
        );

        assert_eq!(
            parse_calendar_entry("2020-01-01T\n"),
            Err(nom::Err::Failure(Error { input: "\n", kind: ErrorKind::Nom(NomErrorKind::Digit), context: None }))
        );

        assert_eq!(
            parse_calendar_entry("2020-01-01/"),
            Err(nom::Err::Failure(Error { input: "", kind: ErrorKind::Nom(NomErrorKind::Fail), context: Some("missing end of interval".to_string()) }))
        );

        assert_eq!(
            parse_calendar_entry("2020-01-01/2020- aa"),
            Err(nom::Err::Failure(Error { input: " aa", kind: ErrorKind::Nom(NomErrorKind::Digit), context: None }))
        );

        assert_eq!(
            parse_calendar_entry("2020-01-01/2020-01 aa"),
            Err(nom::Err::Failure(Error { input: " aa", kind: ErrorKind::Char('-'), context: None }))
        );
    }

    #[test]
    fn test_parse_calendar_file() {
        let entry = CalendarEntry {
            interval: Interval {
                start: NaiveDateTime::new(
                           NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                           NaiveTime::from_hms_opt(20, 0, 0).unwrap()
                       ),
                       end: NaiveDateTime::new(
                           NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                           NaiveTime::from_hms_opt(23, 0, 0).unwrap()
                       ),
                       interval: None,
            },
            description: "Hello there".to_string(),
            metadata: None,
        };

        // make sure newlines dont break things
        assert_eq!(parse_calendar_file(r#"2020-01-01T20:00/2020-01-01T23:00 Hello there
2020-01-01T20:00/2020-01-01T23:00 Hello there
2020-01-01T20:00/2020-01-01T23:00 Hello there


"#), Ok(("", vec![entry.clone(); 3])));

        // also missing newlines
        assert_eq!(parse_calendar_file(r#"2020-01-01T20:00/2020-01-01T23:00 Hello there
2020-01-01T20:00/2020-01-01T23:00 Hello there
2020-01-01T20:00/2020-01-01T23:00 Hello there"#), Ok(("", vec![entry.clone(); 3])));

        // newlines in middle
        assert_eq!(parse_calendar_file(r#"2020-01-01T20:00/2020-01-01T23:00 Hello there

2020-01-01T20:00/2020-01-01T23:00 Hello there
2020-01-01T20:00/2020-01-01T23:00 Hello there"#), Ok(("", vec![entry.clone(); 3])));
    }
}
