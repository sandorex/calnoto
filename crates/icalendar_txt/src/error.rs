use nom::error::{ErrorKind, ParseError};

#[derive(Debug)]
#[allow(dead_code)]
pub enum Error<I> {
    /// Period without any arguments
    EmptyPeriod,

    /// Nom error
    Nom(I, ErrorKind),

    /// List of errors in order they appeared
    List(Vec<Box<Self>>),

    /// Generic rust error
    Generic(I, ErrorKind, Box<dyn std::error::Error>),
}

impl<I: PartialEq + Debug> PartialEq for Error<I> {
    fn eq(&self, other: &Self) -> bool {
        format!("{self}") == format!("{other}")
        // match (self, other) {
        //     (Self::EmptyPeriod, Self::EmptyPeriod) => true,
        //     (Self::Nom(input1, kind1), Self::Nom(input2, kind2)) => (input1 == input2) && (kind1 == kind2),
        //     (Self::List(list1), Self::List(list2)) => list1 == list2,
        //     (Self::Generic(input1, kind1, err1), Self::Generic(input1, kind1, err1)) => (input1])
        //     // _ => todo!(),
        // }
    }
}

impl<I: Debug> Display for Error<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyPeriod => write!(f, "Empty time period")?,

        }
        write!(f, "TODO")?;

        Ok(())
    }
}

impl<I: Debug> std::error::Error for Error<I> {
//     fn cause(&self) -> Option<&dyn std::error::Error> {
//         match self {
//             Self::Generic(_, _, x) => x.downcast_ref(),
//             _ => None
//         }
//     }
//
//     fn description(&self) -> &str {
//
//     }
//
//     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//
//     }
}

impl<I, E> FromExternalError<I, E> for Error<I>
    where E: std::error::Error + Clone + 'static {
    fn from_external_error(input: I, kind: ErrorKind, e: E) -> Self {
        Self::Generic(input, kind, Box::new(e.clone()))
    }
}

impl<T> ParseError<T> for Error<T> {
    fn from_error_kind(input: T, kind: ErrorKind) -> Self {
        Self::Nom(input, kind)
    }

    fn append(input: T, kind: ErrorKind, other: Self) -> Self {
        match other {
            Self::List(mut x) => {
                x.push(Box::new(Self::from_error_kind(input, kind)));

                Self::List(x)
            },
            x => Self::List(vec![
                Box::new(x),
                Box::new(Self::from_error_kind(input, kind)),
            ]),
        }
    }
}

/// Custom error type IResult like `nom::IResult`
type IResult<I, O, E = Error<I>> = Result<(I, O), nom::Err<E>>;
