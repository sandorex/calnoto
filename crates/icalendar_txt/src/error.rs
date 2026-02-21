use std::fmt::{self, Debug, Display};
use nom::error::{ErrorKind as NomErrorKind, FromExternalError, ParseError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error<I> {
    pub input: I,
    pub kind: ErrorKind,
}

/// Error context for `VerboseError`
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub enum ErrorKind {
    /// Description is empty
    EmptyDescription,

    // // Invalid interval
    // InvalidInterval(Box<Self>),
    //
    // // Invalid timestamp
    // InvalidTimestamp(Box<Self>),

    /// Invalid period, or empty period
    InvalidPeriod,

    /// Indicates which character was expected by the `char` function
    Char(char),

    /// Error kind given by various nom parsers
    Nom(NomErrorKind),
}

impl<I> ParseError<I> for Error<I> {
    fn from_error_kind(input: I, kind: NomErrorKind) -> Self {
        Error {
            input,
            kind: ErrorKind::Nom(kind),
        }
    }

    fn append(_: I, _: NomErrorKind, other: Self) -> Self {
        other
    }

    fn from_char(input: I, c: char) -> Self {
        Error {
            input,
            kind: ErrorKind::Char(c),
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
            ErrorKind::EmptyDescription => write!(f, "empty description")?,
            ErrorKind::InvalidPeriod => write!(f, "empty period")?,
            ErrorKind::Nom(e) => match e {
                NomErrorKind::Digit => write!(f, "expected a digit")?,
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
            // errors: value
            //     .errors
            //     .into_iter()
            //     .map(|(i, e)| (i.to_owned(), e))
            //     .collect(),
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
#[derive(Debug, Clone)]
pub struct ErrorInfo {
    /// Line of the error
    pub line: usize,

    /// Column of the error
    pub column: usize,

    /// File where the error happen (if any)
    pub file: Option<String>,

    /// Message describing the error
    pub message: String,

    /// Preformatted error message
    pub(crate) error_mesage: String,
}

impl ErrorInfo {
    /// Converts error into nicer type which has position and nice error message generation
    pub fn from_error<I: core::ops::Deref<Target = str>>(input: I, e: &Error<I>, file: Option<&str>) -> Self {
        use std::fmt::Write;

        let (column, line, text) = {
            let pos = input.len() - e.input.len();

            // TODO limit the context to like 80chars total in case its a huge single line
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

        // pre-format the string
        let mut result = String::new();

        writeln!(&mut result,
            "error {} at {file}{}:{}",
            e,
            line,
            column,

            file=if let Some(file) = file {
                format!("{file}:")
            } else {
                "".to_string()
            }
        ).unwrap();

        // print arrow pointing to position of the error
        write!(&mut result,
            r#" {0:<3}|
 {line:<3}| {text}
 {0:<3}| {caret:>column$}"#,
            "",
            text=text,
            caret="^",
            column=column+1
        ).unwrap();

        Self {
            line,
            column: column + 1,
            file: file.map(|x| x.to_owned()),
            error_mesage: result,
            message: format!("{e}"),
        }
    }
}

impl PartialEq for ErrorInfo {
    fn eq(&self, other: &Self) -> bool {
        // NOTE intentionally not checking error_message
        self.line == other.line &&
            self.column == other.column &&
            self.file == other.file &&
            self.message == other.message
    }
}

impl Display for ErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.error_mesage)
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
        // let input = "bAaa";
        // let err = Error { input: "Aaa", kind: ErrorKind::Char('a') };
        // assert_eq!(ErrorInfo::from_error(input, &err, None), ErrorInfo {
        //     line: 1,
        //     column: 2,
        //     file: None,
        //     message: "expected 'a'".to_string(),
        //     error_mesage: "".to_string(),
        //     // error_mesage: "error expected 'a' at 1:1\n    |\n 1  | bAaa\n    |  ^".to_string(),
        // });

        // TODO error message formatting is broken here
        let input = "a=1\nb2\nc=3";
        let err = Error { input: "2\nc=3", kind: ErrorKind::Char('=') };
        println!("{}", ErrorInfo::from_error(input, &err, None));
        assert_eq!(ErrorInfo::from_error(input, &err, None), ErrorInfo {
            line: 2,
            column: 2,
            file: None,
            message: "expected '='".to_string(),
            error_mesage: "".to_string(),
        });

        todo!()
    }
}

/// Custom error type IResult like `nom::IResult`
pub type IResult<I, O, E = Error<I>> = Result<(I, O), nom::Err<E>>;
