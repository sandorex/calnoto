use std::collections::HashMap;
use chrono::NaiveDate;

// TODO figure out what to do as raw description cannot be retrieved from
// processed one as tags are used in middle of the description
#[derive(Debug)]
pub struct TodoEntry {
    pub completed: bool,

    /// Priority of the task, it must be A-Z ascii character
    pub priority: Option<char>,
    pub completion_date: Option<NaiveDate>,
    pub creation_date: Option<NaiveDate>,

    /// Raw unprocessed description of the task
    pub(crate) raw_description: String,

    // these are parsed from raw_description
    /// Clean description without tag prefix or special properties
    pub(crate) description: String,
    pub(crate) projects: Vec<String>,
    pub(crate) contexts: Vec<String>,
    pub(crate) properties: HashMap<String, String>,
}

// TODO really plan out when and where to complain about malformed description
impl TodoEntry {
    pub fn new(completed: bool, priority: Option<char>, completion_date: Option<NaiveDate>, creation_date: Option<NaiveDate>, description: String) -> Self {
        let mut entry = Self {
            completed,
            priority,
            completion_date,
            creation_date,
            description,
            ..Default::default()
        };

        // TODO Figure out if i want to return result here
        entry.parse_description().unwrap();

        entry
    }

    // TODO this needs a proper error type!
    /// Set description for the entry
    pub fn set_description(&mut self, description: &str) -> Result<(), String> {
        self.raw_description = description.to_string();
        self.parse_description().map_err(|err| format!("{err}"))?;

        Ok(())
    }

    /// Updates internal values for projects, contexts, properties and clean description
    fn parse_description(&mut self) -> nom::IResult<&str, ()> {
        let (leftover, (description, projects, contexts, properties)) = crate::parser::parse_description(&self.raw_description)?;

        self.description = description;
        self.projects = projects;
        self.contexts = contexts;
        self.properties = properties;

        Ok((leftover, ()))
    }

    /// Get raw description that includes tags and properties
    pub fn raw_description(&self) -> &str {
        self.raw_description.as_str()
    }

    /// Get description with tags (prefixes and properties removed)
    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    /// Return projects of todo entry
    pub fn projects(&self) -> &Vec<String> {
        &self.projects
    }

    /// Return contexts of todo entry
    pub fn contexts(&self) -> &Vec<String> {
        &self.contexts
    }

    /// Get all properties defined
    pub fn properties(&self) -> &[(&str, &str)] {
        todo!()
    }

    /// Get single property by name
    pub fn property(&self, key: &str) -> Option<&str> {
        self.properties.get(key).map(|x| x.as_str())
    }
}

impl Default for TodoEntry {
    fn default() -> Self {
        Self {
            completed: false,
            priority: None,
            completion_date: None,
            creation_date: None,
            raw_description: "".to_string(),
            description: "".to_string(),
            projects: vec![],
            contexts: vec![],
            properties: HashMap::new(),
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

        output += &format!("{} ", self.raw_description);

        // remove any leftover whitespace
        output.trim_end().to_string()
    }
}

impl Into::<String> for TodoEntry {
    fn into(self) -> String {
        return Into::<String>::into(&self)
    }
}
