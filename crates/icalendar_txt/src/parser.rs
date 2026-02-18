mod iso8601;
pub use iso8601::Interval;

use iso8601::parse_interval;
use nom::{IResult, Parser, bytes::complete::take_till1, character::complete::{line_ending, space0}, combinator::consumed, multi::separated_list0};
use crate::entry::{CalendarEntry, CalendarEntryMetadata};

/// Parse description of todo entry
pub fn parse_description(input: &str) -> IResult<&str, Option<CalendarEntryMetadata>> {
    let mut metadata = CalendarEntryMetadata::default();

    let (leftover, words) = (separated_list0(
        space0,
        take_till1(|x: char| x.is_whitespace()),
    )).parse(input)?;

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
        consumed(parse_description),
    ).parse(input)?;

    Ok((leftover, CalendarEntry {
        interval,
        description: description.to_string(),
        metadata,
    }))
}

/// Parse file of todo entries
pub fn parse_file(input: &str) -> IResult<&str, Vec<CalendarEntry>> {
    let (leftover, entries) = (separated_list0(
        line_ending,
        parse_entry,
    )).parse(input)?;

    // filter empty entries or with just whitespace
    Ok((leftover, entries.into_iter().filter(|x| !x.description.trim().is_empty()).collect()))
}

