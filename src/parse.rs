use serde_yaml::Value;
use crate::{bubble::Bubble, constraint::Constraint};

#[macro_export]
macro_rules! valstr {
    ($val:expr) => {
        Value::String(String::from($val))
    };
}


#[derive(Debug, PartialEq)]
pub enum PEType<'a> {
    Unsupported,
    UnknownType(&'a str),
    InvalidTypeInfo(&'a Value),
    IncorrectType(&'a Value),
    Regex(regex::Error),
    InvalidDefault(&'a Value),
    InvalidAbsolutePath(&'a Value),
}

#[derive(Debug, PartialEq)]
pub struct ParseErr<'a> {
    pub path: Vec<&'a Value>,
    pub err: PEType<'a>,
}

impl<'a> ParseErr<'a> {
    pub fn new(path: &[&'a Value], err: PEType<'a>) -> ParseErr<'a> {
        ParseErr { path: path.to_vec(), err }
    }
}

impl From<regex::Error> for PEType<'_> {
    fn from(re_err: regex::Error) -> Self {
        PEType::Regex(re_err)
    }
}

impl<'a> From<ParseErr<'a>> for YamlParseResult<'a> {
    fn from(pe: ParseErr<'a>) -> Self {
        YamlParseResult::Single(Err(pe))
    }
}

pub type YamlParseResult<'a> = Bubble<Result<Constraint<'a>, ParseErr<'a>>>;
