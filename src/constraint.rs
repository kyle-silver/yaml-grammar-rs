use lazy_static::lazy_static;

use serde_yaml::{Mapping, Value};

use crate::{obj::{self, ObjectConstraint}, parse::{PEType, ParseErr, YamlParseResult}, str::{self, StringConstraint}, valstr, value_ref::DefaultFetchErr};

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub fn from_spec(kv: (&'a Value, &'a Value)) -> YamlParseResult<'a> {
        Constraint::parse(kv.0, kv.1, &[])
    }

    pub fn parse(field_name: &'a Value, value: &'a Value, parent_path: &[&'a Value]) -> YamlParseResult<'a> {
        let mut path = parent_path.to_vec();
        path.push(field_name);
        match value {
            Value::Null => {
                ParseErr::new(&path, PEType::Unsupported).into()
            },
            Value::Bool(_) => {
                ParseErr::new(&path, PEType::Unsupported).into()
            },
            Value::Number(_) => {
                ParseErr::new(&path, PEType::Unsupported).into()
            },
            Value::String(field_type) => {
                Constraint::for_default(field_name, field_type, &path)
            }
            Value::Sequence(_) => {
                ParseErr::new(&path, PEType::Unsupported).into()
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

    pub fn fetch(&self, path: &[&'a Value]) -> Result<&Constraint<'a>, DefaultFetchErr<'a>> {
        match &self {
            Constraint::Str(s) => Err(DefaultFetchErr::IncorrectType {
                residual_path: path.to_vec(),
                constr: self.clone(),
            }),
            Constraint::Obj(o) => o.constraint(path),
        }
    }
}