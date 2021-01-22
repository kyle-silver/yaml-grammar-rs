use lazy_static::lazy_static;
use serde_yaml::{Mapping, Value};
use crate::{obj_constr::{self, ObjectConstraint}, str_constr::{self, StringConstraint}};

#[macro_export]
macro_rules! valstr {
    ($val:expr) => {
        Value::String($val)
    };
}

#[macro_export]
macro_rules! valslice {
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

impl<'a> From<ParseErr<'a>> for YamlParseResult<'a> {
    fn from(pe: ParseErr<'a>) -> Self {
        YamlParseResult::Single(Err(pe))
    }
}

#[derive(Debug)]
pub enum YamlParseResult<'a> {
    Single(Result<Constraint<'a>, ParseErr<'a>>),
    Multi(Vec<Result<Constraint<'a>, ParseErr<'a>>>)
}

impl<'a> YamlParseResult<'a> {
    fn err(path: &[&'a Value], err: PEType<'a>) -> YamlParseResult<'a> {
        YamlParseResult::Single(Err(ParseErr::new(path, err)))
    }

    fn from_res(res: Result<Constraint<'a>, ParseErr<'a>>) -> YamlParseResult<'a> {
        YamlParseResult::Single(res)
    }

    pub fn all_ok(&self) -> bool {
        match self {
            YamlParseResult::Single(res) => {
                res.is_ok()
            }
            YamlParseResult::Multi(v) => {
                v.iter().all(Result::is_ok)
            }
        }
    }
}

impl<'a> IntoIterator for YamlParseResult<'a> {
    type Item = Result<Constraint<'a>, ParseErr<'a>>;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            YamlParseResult::Single(res) => vec![res].into_iter(),
            YamlParseResult::Multi(v) => v.into_iter()
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Constraint<'a> {
    Str(StringConstraint<'a>),
    Obj(ObjectConstraint<'a>)
}

impl<'a> From<Constraint<'a>> for YamlParseResult<'a> {
    fn from(c: Constraint<'a>) -> Self {
        YamlParseResult::Single(Ok(c))
    }
}

impl<'a> Constraint<'a> {
    pub fn parse(field_name: &'a Value, value: &'a Value, parent_path: &[&'a Value]) -> YamlParseResult<'a> {
        let mut path = parent_path.to_vec();
        path.push(field_name);
        match value {
            Value::Null => {
                YamlParseResult::err(&path, PEType::Unsupported)
            },
            Value::Bool(_) => {
                YamlParseResult::err(&path, PEType::Unsupported)
            },
            Value::Number(_) => {
                YamlParseResult::err(&path, PEType::Unsupported)
            },
            Value::String(field_type) => {
                Constraint::for_default(field_name, field_type, &path).into()
            }
            Value::Sequence(_) => {
                YamlParseResult::err(&path, PEType::Unsupported)
            },
            Value::Mapping(m) => {
                Constraint::for_mapping(field_name, m, &path)
            }
        }
    }

    pub fn field_name(&self) -> &'a Value {
        match self {
            Constraint::Str(c) => c.field_name,
            Constraint::Obj(c) => c.field_name,
        }
    }

    fn for_default(field_name: &'a Value, field_type: &'a str, path: &[&'a Value]) -> YamlParseResult<'a> {
        match field_type {
            "string" => Constraint::Str(StringConstraint::default(field_name)).into(),
            "object" => Constraint::Obj(ObjectConstraint::default(field_name)).into(),
            _ => ParseErr::new(path, PEType::UnknownType(field_type)).into(),
        }
    }

    fn for_mapping(field_name: &'a Value, config: &'a Mapping, path: &[&'a Value]) -> YamlParseResult<'a> {
        lazy_static! {
            static ref TYPE: Value = valstr!(String::from("type"));
        }
        if let Some(Value::String(field_type)) = config.get(&TYPE) {
            match field_type.as_str() {
                "string" => match str_constr::build(field_name, config, path) {
                    Ok(constr) => Constraint::Str(constr).into(),
                    Err(e) => e.into()
                },
                "object" => obj_constr::build(field_name, config, path),
                _ => ParseErr::new(path, PEType::UnknownType(field_type)).into(),
            }
        } else {
            ParseErr::new(path, PEType::InvalidTypeInfo(field_name)).into()
        }
    }
}