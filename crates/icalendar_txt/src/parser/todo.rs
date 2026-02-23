use nom::{Parser, character::complete::{char, multispace1, one_of, space0}, combinator::{all_consuming, consumed, opt}, multi::separated_list0, sequence::{delimited, terminated}};
use crate::{entry::TodoEntry, error::prelude::*, parser::iso8601::parse_date};
use super::parse_description;

pub fn parse_todo_entry(input: &str) -> IResult<&str, TodoEntry> {
    let (leftover, (completed, priority, completion_date, creation_date, (description, metadata))) = (
        // parse completion mark
        opt(terminated(char('x'), space0)).map(|x| x.is_some()),

        // parse priority
        opt(terminated(
            delimited(char('('), one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ"), char(')')),
            space0
        )),

        // parse completion date
        opt(terminated(parse_date, space0)),

        // parse creation date
        opt(terminated(parse_date, space0)),

        consumed(parse_description),
    ).parse(input)?;

    // prevent empty description
    if description.trim().is_empty() {
        return Err(nom::Err::Failure(Error { input: leftover, kind: ErrorKind::EmptyDescription, context: None }))
    }

    Ok((leftover, TodoEntry {
        completed,
        priority,
        completion_date,
        creation_date,
        description: description.to_string(),
        metadata,
    }))
}

/// Parse file of todo entries
pub fn parse_todo_file(input: &str) -> IResult<&str, Vec<TodoEntry>> {
    let (leftover, entries) = all_consuming(separated_list0(
        multispace1,
        parse_todo_entry,
    )).parse(input.trim())?;

    Ok((leftover, entries))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use chrono::NaiveDate;
    use crate::entry::EntryMetadata;
    use super::*;

    #[test]
    fn test_parse_todo_entry() {
        assert_eq!(
            parse_todo_entry("x (B) 2022-01-01 2021-01-01 Hello from +my dear @friends key:val"),
            Ok(("", TodoEntry {
                completed: true,
                priority: Some('B'),
                completion_date: Some(NaiveDate::from_ymd_opt(2022, 1, 1).unwrap()),
                creation_date: Some(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap()),
                description: "Hello from +my dear @friends key:val".to_string(),
                metadata: Some(EntryMetadata {
                    description: "Hello from my dear friends".to_string(),
                    projects: vec!["my".to_string()],
                    contexts: vec!["friends".to_string()],
                    properties: HashMap::from([ ("key".to_string(), "val".to_string()) ]),
                }),
            }))
        );

        assert_eq!(
            parse_todo_entry("x Hello from +my dear @friends key:val"),
            Ok(("", TodoEntry {
                completed: true,
                priority: None,
                completion_date: None,
                creation_date: None,
                description: "Hello from +my dear @friends key:val".to_string(),
                metadata: Some(EntryMetadata {
                    description: "Hello from my dear friends".to_string(),
                    projects: vec!["my".to_string()],
                    contexts: vec!["friends".to_string()],
                    properties: HashMap::from([ ("key".to_string(), "val".to_string()) ]),
                }),
            }))
        );

        assert_eq!(
            parse_todo_entry("Yadda yadda"),
            Ok(("", TodoEntry {
                completed: false,
                priority: None,
                completion_date: None,
                creation_date: None,
                description: "Yadda yadda".to_string(),
                metadata: None,
            }))
        );

        assert_eq!(
            parse_todo_entry("(A) Yadda yadda"),
            Ok(("", TodoEntry {
                completed: false,
                priority: Some('A'),
                completion_date: None,
                creation_date: None,
                description: "Yadda yadda".to_string(),
                metadata: None,
            }))
        );

        assert_eq!(
            parse_todo_entry("(A)"),
            Err(nom::Err::Failure(Error { input: "", kind: ErrorKind::EmptyDescription, context: None }))
        );

        assert_eq!(
            parse_todo_entry("(A) "),
            Err(nom::Err::Failure(Error { input: "", kind: ErrorKind::EmptyDescription, context: None }))
        );

        assert_eq!(
            parse_todo_entry("x"),
            Err(nom::Err::Failure(Error { input: "", kind: ErrorKind::EmptyDescription, context: None }))
        );
    }

    #[test]
    fn test_parse_todo_file() {
        let result = parse_todo_file(r"x (A) this is kinda cool

(B) some other task
yet another task

		
it should be working even with so many newlines and tabs

");

        assert!(result.is_ok());
        let (leftover, entries) = result.unwrap();

        assert_eq!(leftover, "");
        assert_eq!(entries[0], TodoEntry {
            completed: true,
            priority: Some('A'),
            completion_date: None,
            creation_date: None,
            description: "this is kinda cool".to_string(),
            metadata: None,
        });
        assert_eq!(entries[1], TodoEntry {
            completed: false,
            priority: Some('B'),
            completion_date: None,
            creation_date: None,
            description: "some other task".to_string(),
            metadata: None,
        });
        assert_eq!(entries[2], TodoEntry {
            completed: false,
            priority: None,
            completion_date: None,
            creation_date: None,
            description: "yet another task".to_string(),
            metadata: None,
        });
        assert_eq!(entries[3], TodoEntry {
            completed: false,
            priority: None,
            completion_date: None,
            creation_date: None,
            description: "it should be working even with so many newlines and tabs".to_string(),
            metadata: None,
        });
    }
}
