mod iso8601;
pub use iso8601::Interval;

use iso8601::parse_interval;
use nom::{Parser, branch::alt, bytes::complete::{tag, take_till1, take_while1}, character::complete::{alphanumeric1, anychar, char, multispace0, newline, space0, space1}, combinator::{all_consuming, consumed, cut, fail}, error::context, multi::separated_list0, sequence::preceded};
use nom_locate::LocatedSpan;
use crate::{entry::{CalendarEntry, CalendarEntryMetadata}, error::{Error, ErrorKind}};

use crate::error::IResult;

/// Parse description of todo entry
pub fn parse_description(input: &str) -> IResult<&str, Option<CalendarEntryMetadata>> {
    let mut metadata = CalendarEntryMetadata::default();

    let (leftover, words) = separated_list0(
        space0,
        take_till1(|x: char| x.is_whitespace()),
    ).parse(input)?;

    for word in words {
        let clean = match word.chars().next().unwrap() {
            // project tag
            '+' => {
                let clean = &word[1..];
                metadata.projects.push(clean.to_string());

                clean
            },

            // context tag
            '@' => {
                let clean = &word[1..];
                metadata.contexts.push(clean.to_string());

                clean
            },

            // properties or just normal text
            _ => match word.split_once(":") {
                Some((key, value)) => {
                    if key.is_empty() || value.is_empty() {
                        word
                    } else {
                        metadata.properties.insert(key.to_string(), value.to_string());

                        // do not add properties to description
                        ""
                    }
                },
                _ => word,
            },
        };

        if !clean.is_empty() {
            metadata.description += &format!("{clean} ");
        }
    }

    // trim whitespace on ends
    metadata.description = metadata.description.trim().to_string();

    // do not return empty metadata
    Ok((leftover, if metadata.is_empty() { None } else { Some(metadata) }))
}

pub fn parse_entry(input: &str) -> IResult<&str, CalendarEntry> {
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

/// Parse file of todo entries
pub fn parse_file(input: &str) -> IResult<&str, Vec<CalendarEntry>> {
    let (leftover, entries) = all_consuming(separated_list0(
        newline,
        parse_entry,
    )).parse(input.trim())?;

    Ok((leftover, entries))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    use nom::Finish;

    use crate::error::ErrorInfo;

    use super::*;

    #[test]
    fn test_parse_description() {
        // no metadata if there is no tags or properties
        let (_, metadata) = parse_description("Barbecue with friends at toms_house").unwrap();
        assert_eq!(metadata, None);

        let (_, metadata) = parse_description("Barbecue with @friends at +toms_house key:val").unwrap();
        assert_eq!(metadata, Some(CalendarEntryMetadata {
            description: "Barbecue with friends at toms_house".to_string(),
            projects: vec!["toms_house".to_string()],
            contexts: vec!["friends".to_string()],
            properties: HashMap::from([ ("key".to_string(), "val".to_string()) ]),
        }));

        // NOTE as the format is human-editable it has to be fine with garbage input
        let (_, metadata) = parse_description("A num:1 tricky: o+ne w+ith+ :multiple malf@rmed we1.;rd things??!?").unwrap();
        assert_eq!(metadata, Some(CalendarEntryMetadata {
            description: "A tricky: o+ne w+ith+ :multiple malf@rmed we1.;rd things??!?".to_string(),
            projects: vec![],
            contexts: vec![],
            properties: HashMap::from([ ("num".to_string(), "1".to_string()) ]),
        }));
    }

    #[test]
    fn test_parse_entry() {
        assert_eq!(parse_entry("2020-01-01T20:00 Hello there"), Ok(("", CalendarEntry {
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

        assert_eq!(parse_entry("2020-01-01T20:00     Hello there"), Ok(("", CalendarEntry {
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

        assert_eq!(parse_entry("2020-01-01T20:00\t\tHello there"), Ok(("", CalendarEntry {
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

        assert_eq!(parse_entry("2020-01-01T20:00/2020-01-01T23:00 Hello there +blah"), Ok(("", CalendarEntry {
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
            metadata: Some(CalendarEntryMetadata {
                description: "Hello there blah".to_string(),
                projects: vec!["blah".to_string()],
                ..Default::default()
            }),
        })));

        // malformed entry
        let input = "2020-\n01-01/2020-01-01T23:00 Hello there +blah";
        let err = parse_entry(input).finish().unwrap_err().info(input, None);
        assert_eq!(err, ErrorInfo {
            line: 1,
            column: 6,
            file: None,
            message: "expected a digit".to_string(),
            context: "2020-".to_string(),
        });

        // malformed entry
        let input = "2020-01-01T\n";
        let err = parse_entry(input).finish().unwrap_err().info(input, None);
        assert_eq!(err, ErrorInfo {
            line: 1,
            column: 12,
            file: None,
            message: "expected a digit".to_string(),
            context: "2020-01-01T".to_string(),
        });

        // malformed entry
        let input = "2020-01-01/";
        let err = parse_entry(input).finish().unwrap_err().info(input, None);
        assert_eq!(err, ErrorInfo {
            line: 1,
            column: 12,
            file: None,
            message: "missing interval end".to_string(),
            context: "2020-01-01/".to_string(),
        });

        // TODO im getting invalid time instead of invalid date..
        // malformed entry
        let input = "2020-01-01/2020- aa";
        let err = parse_entry(input).finish().unwrap_err().info(input, None);
        assert_eq!(err, ErrorInfo {
            line: 1,
            column: 12,
            file: None,
            message: "missing interval end".to_string(),
            context: "2020-01-01/".to_string(),
        });

        todo!();
    }

    #[test]
    fn test_parse_file() {
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
        assert_eq!(parse_file(r#"2020-01-01T20:00/2020-01-01T23:00 Hello there
2020-01-01T20:00/2020-01-01T23:00 Hello there
2020-01-01T20:00/2020-01-01T23:00 Hello there


"#), Ok(("", vec![entry.clone(); 3])));

        // also missing newlines
        assert_eq!(parse_file(r#"2020-01-01T20:00/2020-01-01T23:00 Hello there
2020-01-01T20:00/2020-01-01T23:00 Hello there
2020-01-01T20:00/2020-01-01T23:00 Hello there"#), Ok(("", vec![entry.clone(); 3])));

        // newlines in middle
        assert_eq!(parse_file(r#"2020-01-01T20:00/2020-01-01T23:00 Hello there

2020-01-01T20:00/2020-01-01T23:00 Hello there
2020-01-01T20:00/2020-01-01T23:00 Hello there"#), Ok(("", vec![entry.clone(); 3])));

        // malformed entry
        assert_eq!(parse_file(r#"2020-01/2020-01-01T23:00 Hello there"#), Ok(("", vec![entry.clone(); 3])));
    }
}
