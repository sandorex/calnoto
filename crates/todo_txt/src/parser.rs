use nom::branch::alt;
use nom::character::complete::{alpha1, alphanumeric1, anychar, char, digit1, line_ending, multispace0, multispace1, not_line_ending, one_of};
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

    Ok((leftover, TodoEntry {
        completed,
        priority,
        completion_date,
        creation_date,
        description: description.to_string(),
    }))
}

/// Parse file of todo entries
pub fn parse_file(input: &str) -> IResult<&str, Vec<TodoEntry>> {
    let (leftover, x) = many0(terminated(
        parse_entry,
        line_ending
    )).parse(input)?;

    Ok((leftover, x))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todo_entry_to_string() {
        let now = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let entry = TodoEntry {
            completed: true,
            priority: Some('Z'),
            completion_date: Some(now),
            creation_date: Some(now),
            description: "Some kind of +task @test location:italy".to_string()
        };

        assert_eq!(Into::<String>::into(&entry), "x (Z) 2026-01-01 2026-01-01 Some kind of +task @test location:italy".to_string());
    }

    #[test]
    fn todo_entry_parse_entry() {
        // test date parsing
        assert_eq!(parse_date("1992-01-2"), Ok(("", NaiveDate::from_ymd_opt(1992, 1, 2).unwrap())));

        let (_, entry) = parse_entry("x (B) 2022-01-01 2021-01-01 Hello from +my dear @friends key:val\r\n").unwrap();
        let entry2 = TodoEntry {
            description: "smth".to_string(),
            ..Default::default()
        };

        assert_eq!(Into::<String>::into(entry), Into::<String>::into(entry2));
    }

    #[test]
    fn todo_entry_parse_entry_file() {
        let file = r#"x (B) Buy groceries +qol
(C) Wash car +qol
Do the +project_x with @alice

        "#;

        let (_, entries) = parse_file(file).unwrap();

        // TODO
    }
}
