use std::{collections::HashMap};
use chrono::NaiveDate;

/// Parsed metadata from description
///
/// Contains tags and custom properties
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct TodoEntryMetadata {
    /// Processed description without tag prefixes or properties
    pub description: String,
    pub projects: Vec<String>,
    pub contexts: Vec<String>,
    pub properties: HashMap<String, String>,
}

#[derive(Debug)]
pub struct TodoEntry {
    pub completed: bool,

    /// Priority of the task, it must be A-Z ascii character
    pub priority: Option<char>,
    pub completion_date: Option<NaiveDate>,
    pub creation_date: Option<NaiveDate>,

    /// Raw unprocessed description of the task
    pub description: String,
}

impl TodoEntry {
    /// Parses description and returns metadata that contains clean description,
    /// tags and properties
    pub fn metadata(&self) -> TodoEntryMetadata {
        let mut metadata = TodoEntryMetadata::default();

        // NOTE manually parsing tags and properties, simpler than using nom tbh
        for word in self.description.split_ascii_whitespace() {
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

        metadata.description = metadata.description.trim().to_string();

        metadata
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todo_entry_metadata() {
        let entry = TodoEntry {
            description: "My good @friend +jack is coming over time:2pm".to_string(),
            ..Default::default()
        };

        assert_eq!(entry.metadata(), TodoEntryMetadata {
            description: "My good friend jack is coming over".to_string(),
            projects: vec!["jack".to_string()],
            contexts: vec!["friend".to_string()],
            properties: HashMap::from([
                ("time".to_string(), "2pm".to_string())
            ])
        });

        // test parsing of partial properties and weird characters
        let entry = TodoEntry {
            description: "A num:1 tricky: o+ne w+ith+ :multiple malf@rmed we1.;rd things??!?".to_string(),
            ..Default::default()
        };

        assert_eq!(entry.metadata(), TodoEntryMetadata {
            description: "A tricky: o+ne w+ith+ :multiple malf@rmed we1.;rd things??!?".to_string(),
            properties: HashMap::from([
                ("num".to_string(), "1".to_string())
            ]),
            ..Default::default()
        });
    }
}
