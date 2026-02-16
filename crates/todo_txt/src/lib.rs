use std::collections::HashMap;
use chrono::NaiveDate;

// TODO add option to redo tags
#[derive(Debug)]
pub struct TodoEntry {
    pub completed: bool,

    /// Priority of the task, it must be A-Z ascii character
    pub priority: Option<char>,
    pub completion_date: Option<NaiveDate>,
    pub creation_date: Option<NaiveDate>,

    /// Description of the task (includes tags)
    pub description: String,

    // TODO could i make these readonly somehow?
    pub special: HashMap<String, String>,
    pub project_tags: Vec<String>,
    pub context_tags: Vec<String>,
}

impl Into::<String> for &TodoEntry {
    fn into(self) -> String {
        let mut output = String::new();

        if self.completed {
            output += "x ";
        }

        if let Some(priority) = self.priority {
            output += &format!("({}) ", priority);
        }

        // creation date has to be present if completion date is
        if let Some(creation_date) = self.creation_date {
            if let Some(completion_date) = self.completion_date {
                output += &format!("{} ", completion_date);
            }

            output += &format!("{} ", creation_date);
        }

        output += &format!("{} ", self.description);

        for (k, v) in &self.special {
            output += &format!("{k}:{v} ");
        }

        // remove any leftover whitespace
        output.trim_end().to_string()
    }
}

impl Into::<String> for TodoEntry {
    fn into(self) -> String {
        return Into::<String>::into(&self)
    }
}

mod parsers {
    use super::*;
    use nom::branch::alt;
    use nom::character::complete::{alpha1, alphanumeric1, anychar, char, digit1, multispace0, multispace1, not_line_ending, one_of};
    use nom::{IResult, Parser};
    use nom::combinator::{eof, map_res, not, opt, recognize, rest, success, value};
    use nom::bytes::complete::{take_till, take_while1, is_not, take, take_while};
    use nom::sequence::{pair, delimited, terminated, preceded};
    use nom::multi::{count, many0, many1};

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
        let mut special: HashMap<String, String> = HashMap::new();

        let (leftover, parsed) = many1((
            alt((
                // +project @context
                (
                    one_of("+@"),
                    take_till(|x: char| x.is_whitespace()),
                ),

                // key:val
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
                // TODO error when invalid key:val pair is found
                ':' => match text.split_once(":") {
                    Some((k, v)) => {
                        special.insert(k.to_string(), v.to_string());
                    }
                    _ => panic!("Could not parse {text:?} key:val pair"),
                },

                _ => {
                    // preserve the origina description
                    description += &format!("{text} ");
                },
            }
        }

        Ok((leftover, (description.trim().to_string(), projects, context, special)))
    }

    pub fn parse_entry(input: &str) -> IResult<&str, TodoEntry> {
        let (leftover, (completed, priority, completion_date, creation_date, (description, project_tags, context_tags, special))) = (
            // parse completion mark
            opt(terminated(char('x'), multispace0)).map(|x| x.is_some()),

            // parse priority
            opt(terminated(delimited(char('('), one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ"), char(')')), multispace0)),

            // parse completion date
            opt(terminated(parse_date, multispace0)),

            // parse creation date
            opt(terminated(parse_date, multispace0)),

            parse_description,
            // parse description until end of line
            // not_line_ending,
        ).parse(input)?;

        // TODO should i just do tag parsing on demand?
        let entry = TodoEntry {
            completed,
            priority,
            completion_date,
            creation_date,
            description,
            project_tags,
            context_tags,
            special,
        };

        Ok((leftover, entry))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::parsers::*;

    #[test]
    fn todo_entry_to_string() {
        let now = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let entry = TodoEntry {
            completed: true,
            priority: Some('Z'),
            completion_date: Some(now),
            creation_date: Some(now),
            description: "Some kind of +task @test".to_string(),
            special: HashMap::from([
                ("location".to_string(), "italy".to_string())
            ]),

            // these are empty intentionally
            project_tags: vec![],
            context_tags: vec![],
        };

        assert_eq!(Into::<String>::into(&entry), "x (Z) 2026-01-01 2026-01-01 Some kind of +task @test location:italy".to_string());
    }

    #[test]
    fn todo_entry_parse_entry() {
        // test date parsing
        assert_eq!(parse_date("1992-01-2"), Ok(("", NaiveDate::from_ymd_opt(1992, 1, 2).unwrap())));
        // assert_eq!(parse_entry("x (yy)"), Ok(("a", ())));

        let entry = parse_entry("x (B) 2022-01-01 2021-01-01 Hello from +my dear @friends key:val\r\n");
        dbg!(&entry);
        todo!();
        // assert_eq!(parse_entry("x (yy)"), Ok(("a", ())));
        // assert_eq!(parse_entry("(adwawff34[]g$@/;])"), Ok(("", "")));
    }
}
