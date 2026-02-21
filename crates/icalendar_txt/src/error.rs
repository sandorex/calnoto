use std::fmt::{self, Debug, Display};
use nom::error::{ContextError, ErrorKind as NomErrorKind, FromExternalError, ParseError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error<I> {
    pub input: I,
    pub kind: ErrorKind,
    pub context: Option<String>,
}

impl<I: core::ops::Deref<Target = str>> Error<I> {
    /// Convert the error into `ErrorInfo`
    pub fn info(&self, input: I, file: Option<&str>) -> ErrorInfo {
        ErrorInfo::from_error(input, self, file)
    }
}

/// Error context for `VerboseError`
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub enum ErrorKind {
    /// Description is empty
    EmptyDescription,

    /// Time is invalid
    InvalidTime,

    /// Date is invalid
    InvalidDate,

    /// Invalid or empty period
    InvalidPeriod,

    /// Indicates which character was expected by the `char` function
    Char(char),

    /// Error kind given by various nom parsers
    Nom(NomErrorKind),
}

impl<I> ContextError<I> for Error<I> {
    fn add_context(_: I, ctx: &'static str, mut other: Self) -> Self {
        // NOTE intentionally not changing the input so the location stays correct
        other.context = Some(ctx.to_owned());
        other
    }
}

impl<I> ParseError<I> for Error<I> {
    fn from_error_kind(input: I, kind: NomErrorKind) -> Self {
        Error {
            input,
            kind: ErrorKind::Nom(kind),
            context: None,
        }
    }

    fn append(_: I, _: NomErrorKind, other: Self) -> Self {
        other
    }

    fn from_char(input: I, c: char) -> Self {
        Error {
            input,
            kind: ErrorKind::Char(c),
            context: None,
        }
    }
}

impl<I, E> FromExternalError<I, E> for Error<I> {
    /// Create a new error from an input position and an external error
    fn from_external_error(input: I, kind: NomErrorKind, _e: E) -> Self {
        Self::from_error_kind(input, kind)
    }
}

impl<I> fmt::Display for Error<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            // if context use it over any other error
            _ if self.context.is_some() => {
                write!(f, "{}", self.context.as_ref().unwrap())?;
            },
            ErrorKind::EmptyDescription => write!(f, "empty description")?,
            ErrorKind::InvalidTime => write!(f, "invalid time")?,
            ErrorKind::InvalidDate => write!(f, "invalid date")?,
            ErrorKind::InvalidPeriod => write!(f, "invalid period")?,
            ErrorKind::Nom(e) => match e {
                NomErrorKind::Digit => write!(f, "expected a digit")?,
                NomErrorKind::Alpha => write!(f, "expected a letter")?,
                NomErrorKind::AlphaNumeric => write!(f, "expected a letter or digit")?,
                NomErrorKind::Space => write!(f, "expected a whitespace")?,
                _ => write!(f, "{:?}", e)?,
            },
            ErrorKind::Char(c) => write!(f, "expected '{}'", c)?,
        }

        Ok(())
    }
}

impl<I: fmt::Debug + fmt::Display> std::error::Error for Error<I> {}

// impl From<Error<&[u8]>> for Error<Vec<u8>> {
//     fn from(value: Error<&[u8]>) -> Self {
//         Error {
//             errors: value
//                 .errors
//                 .into_iter()
//                 .map(|(i, e)| (i.to_owned(), e))
//                 .collect(),
//         }
//     }
// }

impl From<Error<&str>> for Error<String> {
    fn from(value: Error<&str>) -> Self {
        Error {
            input: value.input.to_owned(),
            kind: value.kind,
            context: None,
        }
    }
}

// TODO are these useful at all?
// impl<I> ErrorConvert<Error<I>> for Error<(I, usize)> {
//     fn convert(self) -> Error<I> {
//         Error {
//             errors: self.errors.into_iter().map(|(i, e)| (i.0, e)).collect(),
//         }
//     }
// }
//
// impl<I> ErrorConvert<Error<(I, usize)>> for Error<I> {
//     fn convert(self) -> Error<(I, usize)> {
//         Error {
//             errors: self.errors.into_iter().map(|(i, e)| ((i, 0), e)).collect(),
//         }
//     }
// }

/// Parsed error information including line and column with preformatted error message
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorInfo {
    /// Line of the error
    pub line: usize,

    /// Column of the error
    pub column: usize,

    /// File where the error happen (if any)
    pub file: Option<String>,

    /// Message describing the error
    pub message: String,

    /// Context for the error
    pub context: String,
}

impl ErrorInfo {
    /// Converts error into nicer type which has position and nice error message generation
    pub fn from_error<I: core::ops::Deref<Target = str>>(input: I, e: &Error<I>, file: Option<&str>) -> Self {
        let (column, line, text) = {
            let pos = input.len() - e.input.len();

            // count which line is it on
            let line = (&input.as_bytes()[..pos])
                .iter()
                .filter(|x| **x == b'\n')
                .count() + 1;

            // find first char after newline
            let line_begin = (&input.as_bytes()[..pos])
                .iter()
                .rposition(|x| *x == b'\n')
                .map(|x| x + 1)
                .unwrap_or(0);

            // find last char before next newline
            let line_end = (&input.as_bytes()[pos..])
                .iter()
                .position(|x| *x == b'\n')
                // .map(|x| x - 1)
                .unwrap_or(input.len() - pos);

            // TODO limit the text to like 80chars total in case its a huge single line
            let text = &input[line_begin..pos + line_end];

            (pos - line_begin, line, text)
        };

        Self {
            line,
            column: column + 1,
            file: file.map(|x| x.to_owned()),
            message: format!("{e}"),
            context: text.to_owned(),
        }
    }
}

// impl PartialEq for ErrorInfo {
//     fn eq(&self, other: &Self) -> bool {
//         // NOTE intentionally not checking error_message
//         self.line == other.line &&
//             self.column == other.column &&
//             self.file == other.file &&
//             self.message == other.message
//     }
// }

impl Display for ErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // show file only if defined
        let file = if let Some(file) = &self.file {
            format!("{file}:")
        } else {
            "".to_string()
        };

        writeln!(f,
            "error {} at {file}{}:{}",
            self.message,
            self.line,
            self.column,
        )?;

        // print arrow pointing to position of the error
        write!(f,
            r#" {0:<3}|
 {line:<3}| {text}
 {0:<3}| {caret:>column$}"#,
            "",
            line=self.line,
            text=self.context,
            caret="^",
            column=self.column
        )?;

        Ok(())
    }
}

impl std::error::Error for ErrorInfo {}

/// Converts `Error<I>` to string error message
#[deprecated = "Use `ErrorInfo` instead"]
pub fn convert_error<I: core::ops::Deref<Target = str> + Debug>(input: I, e: &Error<I>, file: Option<&str>) -> String {
    let (column, line, text) = {
        let pos = input.len() - e.input.len();

        let line_begin = (&input.as_bytes()[..pos])
            .iter()
            .rposition(|x| *x == b'\n')
            .unwrap_or(0);

        let line_end = (&input.as_bytes()[pos..])
            .iter()
            .position(|x| *x == b'\n')
            .unwrap_or(input.len() - pos);

        let text = &input[line_begin..pos + line_end];
        let line = (&input.as_bytes()[..pos])
            .iter()
            .filter(|x| **x == b'\n')
            .count() + 1;

        (pos - line_begin, line, text)
    };

    format!(
        "error {} at {file}{line}:{column}\n{text}\n{caret:>column$}",
        e,
        column=column + 1,
        caret="^",

        // add file if defined
        file=if let Some(file) = file {
            format!("{file}:")
        } else {
            "".to_string()
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_error() {
        // basic single line
        let input = "bAaa";
        let err = Error { input: "Aaa", kind: ErrorKind::Char('a'), context: None };
        let err_info = ErrorInfo::from_error(input, &err, None);
        assert_eq!(&err_info, &ErrorInfo {
            line: 1,
            column: 2,
            file: None,
            message: "expected 'a'".to_string(),
            context: "bAaa".to_string(),
        });

        // multiline input in middle
        let input = "a=1\nb2\nc=3";
        let err = Error { input: "2\nc=3", kind: ErrorKind::Char('='), context: None };
        let err_info = ErrorInfo::from_error(input, &err, None);
        assert_eq!(&err_info, &ErrorInfo {
            line: 2,
            column: 2,
            file: None,
            message: "expected '='".to_string(),
            context: "b2".to_string(),
        });

        // multiline test at the end
        let input = "a=1\nb2\nc3";
        let err = Error { input: "3", kind: ErrorKind::Char('='), context: None };
        let err_info = ErrorInfo::from_error(input, &err, None);
        assert_eq!(&err_info, &ErrorInfo {
            line: 3,
            column: 2,
            file: None,
            message: "expected '='".to_string(),
            context: "c3".to_string(),
        });

        // error at end of empty line
        let input = "a=1\n";
        let err = Error { input: "", kind: ErrorKind::Char('='), context: None };
        let err_info = ErrorInfo::from_error(input, &err, None);
        println!("{err_info}");
        assert_eq!(&err_info, &ErrorInfo {
            line: 2,
            column: 1,
            file: None,
            message: "expected '='".to_string(),
            context: "".to_string(),
        });
    }
}

/// Custom error type IResult like `nom::IResult`
pub type IResult<I, O, E = Error<I>> = Result<(I, O), nom::Err<E>>;
