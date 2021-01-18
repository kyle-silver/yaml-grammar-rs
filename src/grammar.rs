use lazy_static::lazy_static;
use serde_yaml::{Mapping, Value};
use crate::strconstr::{self, StringConstraint};

#[macro_export]
macro_rules! valstr {
    ($val:expr) => {
        Value::String($val)
    };
}

#[derive(Debug)]
pub struct ParseErr<'a> {
    path: Vec<&'a Value>,
    msg: String,
}

impl<'a> ParseErr<'a> {
    pub fn new(path: &[&'a Value], msg: String) -> ParseErr<'a> {
        ParseErr { path: path.to_vec(), msg }
    }
}

#[derive(Debug)]
pub enum YamlParseResult<'a> {
    Single(Result<Constraint<'a>, ParseErr<'a>>),
    Multi(Vec<Result<Constraint<'a>, ParseErr<'a>>>)
}

impl<'a> YamlParseResult<'a> {
    fn err(path: &[&'a Value], msg: String) -> YamlParseResult<'a> {
        YamlParseResult::Single(Err(ParseErr::new(path, msg)))
    }

    fn from(res: Result<Constraint<'a>, ParseErr<'a>>) -> YamlParseResult<'a> {
        YamlParseResult::Single(res)
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

#[derive(Debug)]
pub enum ValueRef<'a, T> {
    Literal(&'a T),
    AbsolutePath(Vec<&'a Value>)
}

#[derive(Debug)]
pub enum Constraint<'a> {
    Str(StringConstraint<'a>),
}


impl<'a> Constraint<'a> {
    pub fn parse(field_name: &'a Value, value: &'a Value, parent_path: &[&'a Value]) -> YamlParseResult<'a> {
        let mut path = parent_path.to_vec();
        path.push(field_name);
        match value {
            Value::Null => {
                YamlParseResult::err(&path, String::from("unimplemented"))
            },
            Value::Bool(_) => {
                YamlParseResult::err(&path, String::from("unimplemented"))
            },
            Value::Number(_) => {
                YamlParseResult::err(&path, String::from("unimplemented"))
            },
            Value::String(field_type) => {
                let constr = Constraint::for_default(field_name, field_type, &path);
                YamlParseResult::from(constr)
            }
            Value::Sequence(_) => {
                YamlParseResult::err(&path, String::from("unimplemented"))
            },
            Value::Mapping(m) => {
                Constraint::for_mapping(field_name, m, &path)
            }
        }
    }

    fn for_default(field_name: &'a Value, field_type: &str, path: &[&'a Value]) -> Result<Constraint<'a>, ParseErr<'a>> {
        match field_type {
            "string" => Ok(Constraint::Str(StringConstraint::default(field_name))),
            _ => Err(ParseErr::new(path, format!("Unknown type \"{:?}\"", field_type)))
        }
    }

    fn for_mapping(field_name: &'a Value, map: &'a Mapping, path: &[&'a Value]) -> YamlParseResult<'a> {
        lazy_static! {
            static ref TYPE: Value = valstr!(String::from("type"));
        }
        if let Some(Value::String(field_type)) = map.get(&TYPE) {
            match field_type.as_str() {
                "string" => {
                    match strconstr::build(field_name, map, path) {
                        Ok(constr) => YamlParseResult::from(Ok(Constraint::Str(constr))),
                        Err(e) => YamlParseResult::from(Err(e))
                    }
                }
                _ => YamlParseResult::err(path, format!("Unknown type \"{:?}\"", field_type)),
            }
        } else {
            YamlParseResult::err(path, format!("Type information for field \"{:?}\" was missing or incorrect", field_name))
        }
    }
}