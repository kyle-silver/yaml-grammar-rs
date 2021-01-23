use lazy_static::lazy_static;
use obj::ObjectRule;
use regex::Regex;
use serde_yaml::{Mapping, Value};
use crate::{bubble::Bubble, obj::{self, ObjectConstraint}, str::{self, StringConstraint, StringRule}, value_ref::ValueResolutionErr};

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

#[derive(Debug)]
pub enum YamlParseResult<'a> {
    Single(Result<Constraint<'a>, ParseErr<'a>>),
    Multi(Vec<Result<Constraint<'a>, ParseErr<'a>>>)
}

impl<'a> YamlParseResult<'a> {
    fn err(path: &[&'a Value], err: PEType<'a>) -> YamlParseResult<'a> {
        YamlParseResult::Single(Err(ParseErr::new(path, err)))
    }

    pub fn get(self) -> Vec<Result<Constraint<'a>, ParseErr<'a>>> {
        self.into_iter().collect()
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
                Constraint::for_default(field_name, field_type, &path)
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
            static ref TYPE: Value = valstr!("type");
        }
        if let Some(Value::String(field_type)) = config.get(&TYPE) {
            match field_type.as_str() {
                "string" => match str::build(field_name, config, path) {
                    Ok(constr) => Constraint::Str(constr).into(),
                    Err(e) => e.into()
                },
                "object" => obj::build(field_name, config, path),
                _ => ParseErr::new(path, PEType::UnknownType(field_type)).into(),
            }
        } else {
            ParseErr::new(path, PEType::InvalidTypeInfo(field_name)).into()
        }
    }
}

#[derive(Debug)]
pub enum Rule<'a> {
    Str(StringRule<'a>),
    Obj(ObjectRule<'a>),
}

pub type ValueResolutionResult<'a> = Bubble<Result<Rule<'a>, ValueResolutionErr<'a>>>;

impl<'a> Rule<'a> {
    pub fn new(constraint: &'a Constraint, root: &'a Value) -> ValueResolutionResult<'a> {
        match constraint {
            Constraint::Str(sc) => {
                match StringRule::new(sc, root) {
                    Ok(sr) => Bubble::Single(Ok(sr.into())),
                    Err(e) => Bubble::Single(Err(e))
                }
            }
            Constraint::Obj(_) => todo!(),
        }
    }

    pub fn field_name(&self) -> &'a Value {
        match self {
            Rule::Str(sr) => sr.field_name,
            Rule::Obj(or) => or.field_name,
        }
    }
}