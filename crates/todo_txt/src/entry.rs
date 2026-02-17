use std::{collections::HashMap, fmt::Display};
use chrono::NaiveDate;

/// Parsed metadata from description, contains tags and properties
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct TodoEntryMetadata {
    /// Processed description without tag prefixes or properties
    pub description: String,

    /// Projects referenced in the task
    pub projects: Vec<String>,

    /// Contexts referenced in the task
    pub contexts: Vec<String>,

    /// Properties set in the task
    pub properties: HashMap<String, String>,
}

impl TodoEntryMetadata {
    /// True if there are no tags or properties
    pub(crate) fn is_empty(&self) -> bool {
        self.projects.is_empty() && self.contexts.is_empty() && self.properties.is_empty()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TodoEntry {
    pub completed: bool,

    /// Priority of the task, it must be A-Z ascii character
    pub priority: Option<char>,
    pub completion_date: Option<NaiveDate>,
    pub creation_date: Option<NaiveDate>,

    /// Raw unprocessed description of the task
    pub description: String,

    /// Processed metadata of the task (includes tags and properties)
    pub metadata: Option<TodoEntryMetadata>,
}

impl TodoEntry {
    // TODO this should probably return human-readable error
    /// Try to udpate metadata from current description, true if successful
    pub fn update_metadata(&mut self) -> bool {
        match crate::parser::parse_description(&self.description) {
            Ok((_, metadata)) => {
                self.metadata = metadata;

                true
            },
            _ => false,
        }
    }

    /// Parse todo entries from a file
    pub fn from_str(input: &str) -> nom::IResult<&str, Vec<Self>> {
        crate::parser::parse_file(input)
    }
}

impl Default for TodoEntry {
    fn default() -> Self {
        Self {
            completed: false,
            priority: None,
            completion_date: None,
            creation_date: None,
            description: "".to_string(),
            metadata: None,
        }
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
