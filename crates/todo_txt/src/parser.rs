use nom::character::complete::{char, digit1, line_ending, multispace0, one_of, space0};
use nom::{IResult, Parser};
use nom::combinator::{consumed, map_res, opt};
use nom::bytes::complete::take_till1;
use nom::sequence::{delimited, terminated, preceded};
use nom::multi::separated_list0;
use chrono::NaiveDate;
use crate::entry::{TodoEntry, TodoEntryMetadata};

pub fn parse_date(input: &str) -> IResult<&str, NaiveDate> {
    // let num = map_res(digit1, str::parse::<u16>);
    let (leftover, (year, month, day)) = (
        // parse year
        map_res(digit1, str::parse::<u16>),

        // parse month
        preceded(
            char('-'),
            map_res(digit1, str::parse::<u16>)
        ),

        // parse day
        preceded(
            char('-'),
            map_res(digit1, str::parse::<u16>)
        )
    ).parse(input)?;

    // TODO this will panic probably, just nudge it to the first 1st of next month
    Ok((leftover, NaiveDate::from_ymd_opt(year.into(), month.into(), day.into()).expect("Invalid calendar day {year}-{month}-{day}")))
}

pub fn parse_entry(input: &str) -> IResult<&str, TodoEntry> {
    let (leftover, (completed, priority, completion_date, creation_date, (description, metadata))) = (
        // parse completion mark
        opt(terminated(char('x'), space0)).map(|x| x.is_some()),

        // parse priority
        opt(terminated(delimited(char('('), one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ"), char(')')), multispace0)),

        // parse completion date
        opt(terminated(parse_date, space0)),

        // parse creation date
        opt(terminated(parse_date, space0)),

        consumed(parse_description),
        // parse description until end of line
        // not_line_ending,
    ).parse(input)?;

    Ok((leftover, TodoEntry {
        completed,
        priority,
        completion_date,
        creation_date,
        description: description.to_string(),
        metadata: Some(metadata)
    }))
}

/// Parse file of todo entries
pub fn parse_file(input: &str) -> IResult<&str, Vec<TodoEntry>> {
    let (leftover, entries) = (separated_list0(
        line_ending,
        parse_entry,
    )).parse(input)?;

    // filter empty entries or with just whitespace
    Ok((leftover, entries.into_iter().filter(|x| !x.description.trim().is_empty()).collect()))
}

/// Parse description of todo entry
pub fn parse_description(input: &str) -> IResult<&str, TodoEntryMetadata> {
    let mut metadata = TodoEntryMetadata::default();

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

    Ok((leftover, metadata))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::*;

    #[test]
    fn todo_entry_to_string() {
        let now = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let entry = TodoEntry {
            completed: true,
            priority: Some('Z'),
            completion_date: Some(now),
            creation_date: Some(now),
            description: "Some kind of +task @test location:italy".to_string(),
            ..Default::default()
        };

        assert_eq!(Into::<String>::into(&entry), "x (Z) 2026-01-01 2026-01-01 Some kind of +task @test location:italy".to_string());
    }

    #[test]
    fn todo_entry_parse_date() {
        assert_eq!(parse_date("1992-01-02"), Ok(("", NaiveDate::from_ymd_opt(1992, 1, 2).unwrap())));
        assert_eq!(parse_date("1992-1-2"), Ok(("", NaiveDate::from_ymd_opt(1992, 1, 2).unwrap())));
    }

    #[test]
    fn todo_entry_parse_description() {
        let (_, metadata) = parse_description("Barbecue with @friends at +toms_house key:val").unwrap();
        assert_eq!(metadata, TodoEntryMetadata {
            description: "Barbecue with friends at toms_house".to_string(),
            projects: vec!["toms_house".to_string()],
            contexts: vec!["friends".to_string()],
            properties: HashMap::from([ ("key".to_string(), "val".to_string()) ]),
        });

        // NOTE as the format is human-editable it has to be fine with garbage input
        let (_, metadata) = parse_description("A num:1 tricky: o+ne w+ith+ :multiple malf@rmed we1.;rd things??!?").unwrap();
        assert_eq!(metadata, TodoEntryMetadata {
            description: "A tricky: o+ne w+ith+ :multiple malf@rmed we1.;rd things??!?".to_string(),
            projects: vec![],
            contexts: vec![],
            properties: HashMap::from([ ("num".to_string(), "1".to_string()) ]),
        });
    }

    #[test]
    fn todo_entry_parse_entry() {
        let (_, entry) = parse_entry("x (B) 2022-01-01 2021-01-01 Hello from +my dear @friends key:val").unwrap();
        let entry2 = TodoEntry {
            completed: true,
            priority: Some('B'),
            completion_date: Some(NaiveDate::from_ymd_opt(2022, 1, 1).unwrap()),
            creation_date: Some(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap()),
            description: "Hello from +my dear @friends key:val".to_string(),
            metadata: Some(TodoEntryMetadata {
                description: "Hello from my dear friends".to_string(),
                projects: vec!["my".to_string()],
                contexts: vec!["friends".to_string()],
                properties: HashMap::from([ ("key".to_string(), "val".to_string()) ]),
            }),
        };

        assert_eq!(entry, entry2);
    }

    #[test]
    fn todo_entry_parse_file() {
        let (_, entries) = parse_file(r#"x (B) Buy groceries +qol
(C) Wash car +qol



Do the +project_x with @alice

        "#).unwrap();

        assert_eq!(entries[0], TodoEntry {
            completed: true,
            priority: Some('B'),
            description: "Buy groceries +qol".to_string(),
            ..Default::default()
        });

        assert_eq!(entries[1], TodoEntry {
            priority: Some('C'),
            description: "Wash car +qol".to_string(),
            ..Default::default()
        });

        assert_eq!(entries[2], TodoEntry {
            description: "Do the +project_x with @alice".to_string(),
            ..Default::default()
        });

        // test what happens if there is no newline
        let (_, entries) = parse_file("Something to do\r\n").unwrap();
        assert_eq!(entries.len(), 1);

        // it will not work without newlines
        let (_, entries) = parse_file("Something to do").unwrap();
        assert_eq!(entries.len(), 1);
    }
}
