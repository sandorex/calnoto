use std::{collections::HashMap, fmt::Display};
use chrono::NaiveDate;
use nom::Finish;
use crate::{error::Error, parser::Interval};

/// Parsed metadata from description, contains tags and properties
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct EntryMetadata {
    /// Processed description without tag prefixes or properties
    pub description: String,

    /// Projects referenced in the task
    pub projects: Vec<String>,

    /// Contexts referenced in the task
    pub contexts: Vec<String>,

    /// Properties set in the task
    pub properties: HashMap<String, String>,
}

impl EntryMetadata {
    /// True if there are no tags or properties
    pub(crate) fn is_empty(&self) -> bool {
        self.projects.is_empty() && self.contexts.is_empty() && self.properties.is_empty()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CalendarEntry {
    pub interval: Interval,

    /// Raw unprocessed description
    pub description: String,

    /// Processed metadata of the entry (includes tags and properties)
    pub metadata: Option<EntryMetadata>,
}

impl CalendarEntry {
    /// Try to update metadata from current description
    pub fn update_metadata(&mut self) -> Result<(), Error<&str>> {
        match crate::parser::parse_description(&self.description).finish() {
            Ok((_, metadata)) => {
                self.metadata = metadata;

                Ok(())
            },
            Err(x) => Err(x),
        }
    }

    /// Parse calendar entries from str
    pub fn from_str(input: &str) -> Result<Vec<Self>, crate::error::Error<&str>> {
        crate::parser::parse_calendar_file(input).finish().map(|(_, x)| x)
    }
}

impl Display for &CalendarEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.interval, self.description)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TodoEntry {
    pub completed: bool,

    /// Priority of the task, it must be A-Z ascii character
    pub priority: Option<char>,

    /// Completion date of the task
    pub completion_date: Option<NaiveDate>,

    /// Creation date of the task (must be present if completion_date is)
    pub creation_date: Option<NaiveDate>,

    /// Raw unprocessed description of the task
    pub description: String,

    /// Processed metadata of the entry (includes tags and properties)
    pub metadata: Option<EntryMetadata>,
}

impl TodoEntry {
    /// Try to update metadata from current description, true if successful
    pub fn update_metadata(&mut self) -> Result<(), Error<&str>> {
        match crate::parser::parse_description(&self.description).finish() {
            Ok((_, metadata)) => {
                self.metadata = metadata;

                Ok(())
            },
            Err(x) => Err(x),
        }
    }

    /// Parse calendar entries from str
    pub fn from_str(input: &str) -> Result<Vec<Self>, Error<&str>> {
        crate::parser::parse_todo_file(input).finish().map(|(_, x)| x)
    }
}

impl Display for &TodoEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.completed {
            write!(f, "x ")?;
        }

        if let Some(priority) = self.priority {
            write!(f, "({priority}) ")?;
        }

        // creation date has to be present if completion date is
        if let Some(creation_date) = self.creation_date {
            if let Some(completion_date) = self.completion_date {
                write!(f, "{completion_date} ")?;
            }

            write!(f, "{creation_date} ")?;
        }

        write!(f, "{}", self.description)?;

        Ok(())
    }
}
