use nom::branch::alt;
use nom::character::complete::{alpha1, alphanumeric1, anychar, char, digit1, multispace0, multispace1, not_line_ending, one_of};
use nom::{IResult, Parser};
use nom::combinator::{eof, map_res, not, opt, recognize, rest, success, value};
use nom::bytes::complete::{take_till, take_while1, is_not, take, take_while};
use nom::sequence::{pair, delimited, terminated, preceded};
use nom::multi::{count, many0, many1};
use std::collections::HashMap;
use chrono::NaiveDate;
use crate::entry::TodoEntry;

pub fn parse_date(input: &str) -> IResult<&str, NaiveDate> {
    // let num = map_res(digit1, str::parse::<u16>);
    let (leftover, (year, month, day)) = (
        // parse year
        map_res(digit1, str::parse::<u16>),

        // TODO use sparated_list1 here!
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

    // TODO this will panic probably
    Ok((leftover, NaiveDate::from_ymd_opt(year.into(), month.into(), day.into()).expect("Invalid calendar day {year}-{month}-{day}")))
}

pub fn parse_description(input: &str) -> IResult<&str, (String, Vec<String>, Vec<String>, HashMap<String, String>)> {
    let mut description: String = String::new();
    let mut projects: Vec<String> = vec![];
    let mut context: Vec<String> = vec![];
    let mut properties: HashMap<String, String> = HashMap::new();

    let (leftover, parsed) = many1((
        alt((
            // +project @context
            (
                one_of("+@"),
                take_till(|x: char| x.is_whitespace()),
            ),

            // key:val properties
            (
                success(':'),
                recognize((
                    take_till(|x: char| x == ':' || x.is_whitespace()),
                    char(':'),
                    take_till(|x: char| x.is_whitespace()),
                )),
            ),

            // other
            (
                success('?'),
                value("", take_till(|x: char| x.is_whitespace())),
            )
        )),
        value("", multispace1),
    ))
    .parse(input)?;

    for ((ch, text), _) in parsed {
        match ch {
            '+' => {
                description += &format!("+{text} ");
                projects.push(text.to_string());
            },
            '@' => {
                description += &format!("@{text} ");
                context.push(text.to_string());
            },
            ':' => match text.split_once(":") {
                Some((k, v)) => {
                    properties.insert(k.to_string(), v.to_string());
                }

                // ignore invalid properties
                _ => {
                    description += &format!("{text} ");
                },
            },

            _ => {
                // preserve the origina description
                description += &format!("{text} ");
            },
        }
    }

    Ok((leftover, (description.trim().to_string(), projects, context, properties)))
}

pub fn parse_entry(input: &str) -> IResult<&str, TodoEntry> {
    let (leftover, (completed, priority, completion_date, creation_date, description)) = (
        // parse completion mark
        opt(terminated(char('x'), multispace0)).map(|x| x.is_some()),

        // parse priority
        opt(terminated(delimited(char('('), one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ"), char(')')), multispace0)),

        // parse completion date
        opt(terminated(parse_date, multispace0)),

        // parse creation date
        opt(terminated(parse_date, multispace0)),

        // parse description until end of line
        not_line_ending,
    ).parse(input)?;

    Ok((leftover, TodoEntry::new(completed, priority, completion_date, creation_date, description.to_string())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todo_entry_to_string() {
        // TODO this is kinda useless atm
        let now = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let entry = TodoEntry::new(true, Some('Z'), Some(now), Some(now), "Some kind of +task @test location:italy".to_string());
        // {
        //     completed: true,
        //     priority: Some('Z'),
        //     completion_date: Some(now),
        //     creation_date: Some(now),
        //     raw_description: "Some kind of +task @test location:italy".to_string(),
        //     ..Default::default()
        // };

        assert_eq!(Into::<String>::into(&entry), "x (Z) 2026-01-01 2026-01-01 Some kind of +task @test location:italy".to_string());
    }

    #[test]
    fn todo_entry_parse_entry() {
        // test date parsing
        assert_eq!(parse_date("1992-01-2"), Ok(("", NaiveDate::from_ymd_opt(1992, 1, 2).unwrap())));

        let (_, entry) = parse_entry("x (B) 2022-01-01 2021-01-01 Hello from +my dear @friends key:val\r\n").unwrap();
        let entry2 = TodoEntry {
            raw_description: "smth".to_string(),
            ..Default::default()
        };

        assert_eq!(Into::<String>::into(entry), Into::<String>::into(entry2));
    }
}
