use std::{collections::HashMap};
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
        if let Ok((_, metadata)) = crate::parser::parse_description(&self.description) {
            self.metadata = Some(metadata);
            true
        } else {
            false
        }
    }

    /// Parse todo entries from a file
    pub fn from_str(input: &str) -> nom::IResult<&str, Vec<Self>> {
        crate::parser::parse_file(input)
    }

    /// Get a string representation of todo entry
    pub fn into_string(&self) -> String {
        Into::<String>::into(self)
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

        // remove any leftover whitespace
        output.trim_end().to_string()
    }
}

impl Into::<String> for TodoEntry {
    fn into(self) -> String {
        return Into::<String>::into(&self)
    }
}
