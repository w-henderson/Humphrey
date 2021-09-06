use std::convert::TryFrom;

pub const FCGI_VERSION: u8 = 1;
pub const FCGI_HEADER_SIZE: usize = 8;

/// Represents a type of request.
#[derive(Debug, PartialEq, Eq)]
pub enum FcgiType {
    Begin = 1,
    Abort,
    End,
    Params,
    Stdin,
    Stdout,
    Stderr,
    Data,
    GetValues,
    GetValuesResult,
    UnknownType,
}

impl TryFrom<u8> for FcgiType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Begin),
            2 => Ok(Self::Abort),
            3 => Ok(Self::End),
            4 => Ok(Self::Params),
            5 => Ok(Self::Stdin),
            6 => Ok(Self::Stdout),
            7 => Ok(Self::Stderr),
            8 => Ok(Self::Data),
            9 => Ok(Self::GetValues),
            10 => Ok(Self::GetValuesResult),
            11 => Ok(Self::UnknownType),
            _ => Err(()),
        }
    }
}
