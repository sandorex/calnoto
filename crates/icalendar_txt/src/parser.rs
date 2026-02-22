mod todo;
mod calendar;
mod iso8601;

use nom::{Parser, bytes::complete::take_till1, character::complete::space0, multi::separated_list0};
use crate::entry::EntryMetadata;
use crate::error::prelude::*;

pub use iso8601::Interval;
pub use calendar::parse_calendar_file;
pub use todo::parse_todo_file;

/// Parse description of todo entry
pub fn parse_description(input: &str) -> IResult<&str, Option<EntryMetadata>> {
    let mut metadata = EntryMetadata::default();

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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::*;

    #[test]
    fn test_parse_description() {
        // no metadata if there is no tags or properties
        let (_, metadata) = parse_description("Barbecue with friends at toms_house").unwrap();
        assert_eq!(metadata, None);

        let (_, metadata) = parse_description("Barbecue with @friends at +toms_house key:val").unwrap();
        assert_eq!(metadata, Some(EntryMetadata {
            description: "Barbecue with friends at toms_house".to_string(),
            projects: vec!["toms_house".to_string()],
            contexts: vec!["friends".to_string()],
            properties: HashMap::from([ ("key".to_string(), "val".to_string()) ]),
        }));

        // NOTE as the format is human-editable it has to be fine with garbage input
        let (_, metadata) = parse_description("A num:1 tricky: o+ne w+ith+ :multiple malf@rmed we1.;rd things??!?").unwrap();
        assert_eq!(metadata, Some(EntryMetadata {
            description: "A tricky: o+ne w+ith+ :multiple malf@rmed we1.;rd things??!?".to_string(),
            projects: vec![],
            contexts: vec![],
            properties: HashMap::from([ ("num".to_string(), "1".to_string()) ]),
        }));
    }
}
