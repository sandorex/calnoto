use std::{fmt::Display, ops::Add};
use nom::{Parser, branch::alt, bytes::complete::tag, character::complete::{char, digit1}, combinator::{consumed, cut, fail, map_res, opt, success}, error::context, sequence::{preceded, terminated}};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeDelta};
use crate::error::prelude::*;

/// Holds duration (like `chrono::TimeDelta` but seconds, minutes etc are stored separately)
///
/// Converts losslessly back and forth to a ISO8601 period
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeDuration {
    pub years: u16,
    pub months: u16,
    pub weeks: u16,
    pub days: u16,
    pub hours: u16,
    pub minutes: u16,
    pub seconds: u16,
}

#[allow(dead_code)]
impl TimeDuration {
    pub fn years(years: u16) -> Self {
        Self {
            years,
            ..Default::default()
        }
    }

    pub fn months(months: u16) -> Self {
        Self {
            months,
            ..Default::default()
        }
    }

    pub fn weeks(weeks: u16) -> Self {
        Self {
            weeks,
            ..Default::default()
        }
    }

    pub fn days(days: u16) -> Self {
        Self {
            days,
            ..Default::default()
        }
    }

    pub fn hours(hours: u16) -> Self {
        Self {
            hours,
            ..Default::default()
        }
    }

    pub fn minutes(minutes: u16) -> Self {
        Self {
            minutes,
            ..Default::default()
        }
    }

    pub fn seconds(seconds: u16) -> Self {
        Self {
            seconds,
            ..Default::default()
        }
    }

    fn is_zero(&self) -> bool {
        self.years == 0
            && self.months == 0
            && self.weeks == 0
            && self.days == 0
            && self.hours == 0
            && self.minutes == 0
            && self.seconds == 0
    }
}

// TODO test
impl Display for TimeDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_zero() {
            // something has to be written for it to be valid
            write!(f, "T{}S", self.seconds)?;
        } else {
            if self.years > 0 {
                write!(f, "{}Y", self.years)?;
            }

            if self.months > 0 {
                write!(f, "{}M", self.months)?;
            }

            if self.weeks > 0 {
                write!(f, "{}W", self.weeks)?;
            }

            if self.days > 0 {
                write!(f, "{}D", self.days)?;
            }

            if self.hours > 0 || self.minutes > 0 || self.seconds > 0 {
                write!(f, "T")?;

                if self.hours > 0 {
                    write!(f, "{}H", self.hours)?;
                }
                if self.minutes > 0 {
                    write!(f, "{}M", self.minutes)?;
                }
                if self.seconds > 0 {
                    write!(f, "{}S", self.seconds)?;
                }
            }
        }

        Ok(())
    }
}

// allow adding TimeDuration to chrono
impl Add<TimeDuration> for chrono::NaiveDateTime {
    type Output = Option<Self>;

    fn add(self, rhs: TimeDuration) -> Self::Output {
        use chrono::{Months, Days};

        // just add together all the parts
        Some(
            self.checked_add_months(Months::new((rhs.years * 12).into()))?
            .checked_add_months(Months::new(rhs.months.into()))?
            .checked_add_days(Days::new((rhs.weeks * 7).into()))?
            .checked_add_days(Days::new((rhs.days).into()))?
            + std::time::Duration::from_hours(rhs.hours.into())
            + std::time::Duration::from_mins(rhs.minutes.into())
            + std::time::Duration::from_secs(rhs.seconds.into())
        )
    }
}

#[derive(Debug)]
#[allow(dead_code)]
enum Time {
    DateTime(NaiveDateTime),
    Date(NaiveDate),
    Time(NaiveTime),
    Period(TimeDuration),
}

impl Into<Time> for NaiveDateTime {
    fn into(self) -> Time {
        Time::DateTime(self)
    }
}

impl Into<Time> for NaiveDate {
    fn into(self) -> Time {
        Time::Date(self)
    }
}

impl Into<Time> for NaiveTime {
    fn into(self) -> Time {
        Time::Time(self)
    }
}

impl Into<Time> for TimeDuration {
    fn into(self) -> Time {
        Time::Period(self)
    }
}

fn parse_date(input: &str) -> IResult<&str, NaiveDate> {
    // TODO i think this one is kinda broken
    let (leftover, (year, (month, day))) = (
        terminated(map_res(digit1, str::parse::<u16>), char('-')),
        cut((
            terminated(map_res(digit1, str::parse::<u16>), char('-')),
            map_res(digit1, str::parse::<u16>),
        )),
    ).parse(input)?;

    // invalid date should produce an error
    match NaiveDate::from_ymd_opt(year.into(), month.into(), day.into()) {
        Some(x) => Ok((leftover, x)),
        None => Err(nom::Err::Failure(Error { input: leftover, kind: ErrorKind::InvalidDate, context: None })),
    }
}

fn parse_time(input: &str) -> IResult<&str, NaiveTime> {
    let (leftover, (hr, min, sec)) = (
        map_res(digit1, str::parse::<u16>),
        opt(preceded(char(':'), cut(map_res(digit1, str::parse::<u16>))))
            .map(|x| x.unwrap_or(0)),
        opt(preceded(char(':'), cut(map_res(digit1, str::parse::<u16>))))
            .map(|x| x.unwrap_or(0))
    ).parse(input)?;

    // invalid time should produce an error
    match NaiveTime::from_hms_opt(hr.into(), min.into(), sec.into()) {
        Some(x) => Ok((leftover, x)),
        None => Err(nom::Err::Failure(Error { input: leftover, kind: ErrorKind::InvalidTime, context: None })),
    }
}

fn parse_timestamp(input: &str) -> IResult<&str, NaiveDateTime> {
    let (leftover, (date, time)) = (
        parse_date,
        preceded(char('T'), cut(parse_time)),
    ).parse(input)?;

    Ok((leftover, NaiveDateTime::new(date, time)))
}

// TODO this could give better errors instead of InvalidPeriod of reverything
fn parse_period(input: &str) -> IResult<&str, TimeDuration> {
    let (leftover, (raw, (yr, mon, week, day, (hr, min, sec)))) = consumed((
        opt(terminated(map_res(digit1, str::parse::<u16>), char('Y'))),
        opt(terminated(map_res(digit1, str::parse::<u16>), char('M'))),
        opt(terminated(map_res(digit1, str::parse::<u16>), char('W'))),
        opt(terminated(map_res(digit1, str::parse::<u16>), char('D'))),

        opt(preceded(
            char('T'),
            (
                opt(terminated(map_res(digit1, str::parse::<u16>), char('H'))),
                opt(terminated(map_res(digit1, str::parse::<u16>), char('M'))),
                opt(terminated(map_res(digit1, str::parse::<u16>), char('S'))),
            )
        // removing one layer of option
        )).map(|x| x.unwrap_or((None, None, None)))
    )).parse(input)?;

    if raw.is_empty() || raw == "T" {
        // empty period
        Err(nom::Err::Failure(Error { input, kind: ErrorKind::InvalidPeriod, context: None }))
    } else {
        Ok((leftover, TimeDuration {
            years: yr.unwrap_or(0),
            months: mon.unwrap_or(0),
            weeks: week.unwrap_or(0),
            days: day.unwrap_or(0),
            hours: hr.unwrap_or(0),
            minutes: min.unwrap_or(0),
            seconds: sec.unwrap_or(0),
        }))
    }
}

// TODO implement display (requires TimeDuration)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Interval {
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    // TODO use TimeDuration here
    pub interval: Option<TimeDuration>,
}

/// Parses interval like
pub fn parse_interval(input: &str) -> IResult<&str, Interval> {
    // parses <datetime|date>
    fn start_parser(input: &str) -> IResult<&str, NaiveDateTime> {
        alt((
            parse_timestamp,
            parse_date.map(|x| NaiveDateTime::from(x)),
        )).parse(input)
    }

    // parses /<datetime|date|time|Pperiod>
    fn end_parser(input: &str) -> IResult<&str, Time> {
        // TODO it tries to parse /2020- as time... smh
        alt((
            // TODO merge parse_timestamp and parse_date here so it optionally parses time
            preceded(char('P'), cut(parse_period)).map(|x| Into::<Time>::into(x)),
            parse_timestamp.map(|x| Into::<Time>::into(x)),
            parse_date.map(|x| Into::<Time>::into(x)),
            parse_time.map(|x| Into::<Time>::into(x)),
            context("missing end of interval", fail()),
        )).parse(input)
    }

    let (leftover, (start, end, recurrance)) = alt((
        // NOTE this is technically not ISO8601 compliant as first argument can be
        // a period as well but oh well
        //
        // recurring R/<datetime|date>[/<datetime|date|time|Pperiod>]/F<period>
        (
            preceded(tag("R/"), cut(start_parser)),
            opt(preceded(char('/'), end_parser)),
            preceded(tag("/F"), cut(parse_period)).map(|x| Some(x)),
        ),
        // non-recurring
        // <datetime|date>[/<datetime|time|Pperiod>]
        (
            start_parser,
            opt(preceded(char('/'), cut(end_parser))),
            success(None),
        )
    )).parse(input)?;

    let end = match end {
        // if its time then assume same date
        Some(Time::Time(x)) => NaiveDateTime::new(start.date(), x),

        // if its a date then assume midnight
        Some(Time::Date(x)) => NaiveDateTime::from(x),

        // if its a period just do the math
        Some(Time::Period(x)) => {
            // TODO idk if this is the nicest way to do this
            match start + x {
                Some(end) => end,
                None => return Err(nom::Err::Failure(Error { input: leftover, kind: ErrorKind::InvalidPeriod, context: None }))
            }
        },

        // do not touch full datetime
        Some(Time::DateTime(x)) => x,

        // if its none, assume midnight of next day
        None => NaiveDateTime::new(
            (start + TimeDelta::days(1)).date(),
            NaiveTime::from_hms_opt(0, 0, 0).unwrap()
        ),
    };

    if let Some(recurrance) = recurrance {
        Ok((leftover, Interval {
            start,
            end,
            interval: Some(recurrance),
        }))
    } else {
        Ok((leftover, Interval {
            start,
            end,
            interval: None,
        }))
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike, Timelike};
    use nom::Finish;

    use super::*;

    #[test]
    fn test_parse_date() {
        assert_eq!(parse_date("2020-01-01"), Ok(("", NaiveDate::from_ymd_opt(2020, 1, 1).unwrap())));
        assert_eq!(parse_date("2020-10-20"), Ok(("", NaiveDate::from_ymd_opt(2020, 10, 20).unwrap())));
    }

    #[test]
    fn test_parse_time() {
        assert_eq!(parse_time("14:00"), Ok(("", NaiveTime::from_hms_opt(14, 0, 0).unwrap())));
        assert_eq!(parse_time("13:10:05"), Ok(("", NaiveTime::from_hms_opt(13, 10, 5).unwrap())));
    }

    #[test]
    fn test_parse_timestamp() {
        assert_eq!(parse_timestamp("1997-07-23T14:30"), Ok(("", NaiveDateTime::new(
            NaiveDate::from_ymd_opt(1997, 07, 23).unwrap(),
            NaiveTime::from_hms_opt(14, 30, 0).unwrap(),
        ))));

        assert_eq!(parse_timestamp("1997-07-23T14:30:01"), Ok(("", NaiveDateTime::new(
            NaiveDate::from_ymd_opt(1997, 07, 23).unwrap(),
            NaiveTime::from_hms_opt(14, 30, 1).unwrap(),
        ))));
    }

    #[test]
    fn test_parse_interval() {
        let datetime = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        );

        // R/2020-01-01/F2D
        assert_eq!(parse_interval("R/2020-01-01/F2D"), Ok(("", Interval {
            start: datetime.with_hour(0).unwrap(),
            end: datetime.with_day(2).unwrap(),
            interval: Some(TimeDuration::days(2)),
        })));

        // R/2020-01-01T20:00/F2D
        assert_eq!(parse_interval("R/2020-01-01T20:00/F2D"), Ok(("", Interval {
            start: datetime.with_hour(20).unwrap(),
            end: datetime.with_day(2).unwrap(),
            interval: Some(TimeDuration::days(2)),
        })));

        // R/2020-01-01T20:00/22:00/F2D
        assert_eq!(parse_interval("R/2020-01-01T20:00/22:00/F2D"), Ok(("", Interval {
            start: datetime.with_hour(20).unwrap(),
            end: datetime.with_hour(22).unwrap(),
            interval: Some(TimeDuration::days(2)),
        })));

        // R/2020-01-01T20:00/2020-01-02/F2D
        assert_eq!(parse_interval("R/2020-01-01T20:00/2020-01-02/F2D"), Ok(("", Interval {
            start: datetime.with_hour(20).unwrap(),
            end: datetime.with_day(2).unwrap(),
            interval: Some(TimeDuration::days(2)),
        })));

        // R/2020-01-01T20:00/2020-01-01T22:00/F2D
        assert_eq!(parse_interval("R/2020-01-01T20:00/2020-01-01T22:00/F2D"), Ok(("", Interval {
            start: datetime.with_hour(20).unwrap(),
            end: datetime.with_hour(22).unwrap(),
            interval: Some(TimeDuration::days(2)),
        })));

        // 2020-01-01
        assert_eq!(parse_interval("2020-01-01"), Ok(("", Interval {
            start: datetime,
            end: datetime.with_day(2).unwrap(),
            interval: None,
        })));

        // 2020-01-01/PT20M
        assert_eq!(parse_interval("2020-01-01/PT20M"), Ok(("", Interval {
            start: datetime,
            end: datetime.with_minute(20).unwrap(),
            interval: None,
        })));

        // 2020-01-01T20:00
        assert_eq!(parse_interval("2020-01-01T20:00"), Ok(("", Interval {
            start: datetime.with_hour(20).unwrap(),
            end: datetime.with_day(2).unwrap(),
            interval: None,
        })));

        // 2020-01-01/20:00
        assert_eq!(parse_interval("2020-01-01/20:00"), Ok(("", Interval {
            start: datetime.with_hour(0).unwrap(),
            end: datetime.with_hour(20).unwrap(),
            interval: None,
        })));

        // 2020-01-01T20:00/22:00
        assert_eq!(parse_interval("2020-01-01T20:00/22:00"), Ok(("", Interval {
            start: datetime.with_hour(20).unwrap(),
            end: datetime.with_hour(22).unwrap(),
            interval: None,
        })));

        // 2020-01-01T20:00/2020-01-01T22:00
        assert_eq!(parse_interval("2020-01-01T20:00/2020-01-01T22:00"), Ok(("", Interval {
            start: datetime.with_hour(20).unwrap(),
            end: datetime.with_hour(22).unwrap(),
            interval: None,
        })));

        // 2020-01-01T20:00/PT2H20M
        assert_eq!(parse_interval("2020-01-01T20:00/PT2H20M"), Ok(("", Interval {
            start: datetime.with_hour(20).unwrap(),
            end: datetime
                .with_hour(22).unwrap()
                .with_minute(20).unwrap(),
            interval: None,
        })));

        // malformed
        assert_eq!(
            parse_interval("2020-01-/PT2H20M").finish(),
            Err(Error { input: "/PT2H20M", kind: ErrorKind::Nom(NomErrorKind::Digit), context: None })
        );

        assert_eq!(
            parse_interval("2020-01/PT2H20M").finish(),
            Err(Error { input: "/PT2H20M", kind: ErrorKind::Char('-'), context: None })
        );

        assert_eq!(
            parse_interval("2020-01-/PT2H20M").finish(),
            Err(Error { input: "/PT2H20M", kind: ErrorKind::Nom(NomErrorKind::Digit), context: None })
        );

        assert_eq!(
            parse_interval("2020-01-01/").finish(),
            Err(Error { input: "", kind: ErrorKind::Nom(NomErrorKind::Fail), context: Some("missing end of interval".to_string()) })
        );

        assert_eq!(
            parse_interval("2020-01-01/PT").finish(),
            Err(Error { input: "T", kind: ErrorKind::InvalidPeriod, context: None })
        );
    }

    #[test]
    fn test_parse_period() {
        // NOTE i know this is ugly but eh
        assert_eq!(parse_period("1Y2M3W4DT5H6M7S"), Ok(("", TimeDuration {
            years: 1,
            months: 2,
            weeks: 3,
            days: 4,
            hours: 5,
            minutes: 6,
            seconds: 7,
        })));

        assert_eq!(parse_period("T7S"), Ok(("", TimeDuration::seconds(7))));
        assert_eq!(parse_period("T2H"), Ok(("", TimeDuration::hours(2))));

        // fail properly when invalid period
        assert_eq!(parse_period("T"), Err(nom::Err::Failure(Error { input: "T", kind: ErrorKind::InvalidPeriod, context: None })));
        assert_eq!(parse_period(""), Err(nom::Err::Failure(Error { input: "", kind: ErrorKind::InvalidPeriod, context: None })));
        assert_eq!(parse_period("M"), Err(nom::Err::Failure(Error { input: "M", kind: ErrorKind::InvalidPeriod, context: None })));
    }
}
