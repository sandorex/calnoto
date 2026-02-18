use std::{collections::HashMap, fmt::Display};
use crate::parser::Interval;

/// Parsed metadata from description, contains tags and properties
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct CalendarEntryMetadata {
    /// Processed description without tag prefixes or properties
    pub description: String,

    /// Projects referenced in the task
    pub projects: Vec<String>,

    /// Contexts referenced in the task
    pub contexts: Vec<String>,

    /// Properties set in the task
    pub properties: HashMap<String, String>,
}

impl CalendarEntryMetadata {
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
    pub metadata: Option<CalendarEntryMetadata>,
}

impl CalendarEntry {
    // TODO this should probably return human-readable error
    /// Try to update metadata from current description, true if successful
    pub fn update_metadata(&mut self) -> bool {
        todo!()
        // match crate::parser::parse_description(&self.description) {
        //     Ok((_, metadata)) => {
        //         self.metadata = metadata;
        //
        //         true
        //     },
        //     _ => false,
        // }
    }

    /// Parse todo entries from a file
    pub fn from_str(input: &str) -> nom::IResult<&str, Vec<Self>> {
        todo!()
        // crate::parser::parse_file(input)
    }
}

// TODO requires Interval to impl Display
// impl Display for &CalendarEntry {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         // if let Some(priority) = self.priority {
//         //     write!(f, "({priority}) ")?;
//         // }
//         //
//         // // creation date has to be present if completion date is
//         // if let Some(creation_date) = self.creation_date {
//         //     if let Some(completion_date) = self.completion_date {
//         //         write!(f, "{completion_date} ")?;
//         //     }
//         //
//         //     write!(f, "{creation_date} ")?;
//         // }
//
//         write!(f, "{}", self.description)?;
//
//         Ok(())
//     }
// }
