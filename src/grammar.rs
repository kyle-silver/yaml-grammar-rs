use lazy_static::lazy_static;
use serde_yaml::{Mapping, Value};
use crate::string_constraint::StringConstraint;

#[macro_export]
macro_rules! valstr {
    ($val:expr) => {
        Value::String($val)
    };
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
    pub fn parse(field_name: &'a Value, value: &'a Value, path: &[&'a Value]) -> Result<Constraint<'a>, String> {
        match value {
            Value::Null => Err(format!{""}),
            Value::Bool(_) => Err(format!{""}),
            Value::Number(_) => Err(format!{""}),
            Value::String(field_type) => Constraint::for_default(field_name, field_type, path),
            Value::Sequence(_) => Err(format!{""}),
            Value::Mapping(m) => Constraint::for_mapping(field_name, m, path)
        }
    }

    fn for_default(field_name: &'a Value, field_type: &str, path: &[&'a Value]) -> Result<Constraint<'a>, String> {
        match field_type {
            "string" => Ok(Constraint::Str(StringConstraint::default(field_name))),
            _ => Err(format!("Unknown type \"{:?}\" at \"{:?}\": \"{:?}\"", field_type, field_name, path))
        }
    }

    fn for_mapping(field_name: &'a Value, map: &'a Mapping, path: &[&'a Value]) -> Result<Constraint<'a>, String> {
        lazy_static! {
            static ref TYPE: Value = valstr!(String::from("type"));
        }
        if let Some(Value::String(field_type)) = map.get(&TYPE) {
            match field_type.as_str() {
                "string" => Ok(Constraint::Str(StringConstraint::from_mapping(field_name, map, path)?)),
                _ => Err(format!("Unknown type \"{:?}\" at \"{:?}\": \"{:?}\"", field_type, field_name, path))
            }
        } else {
            Err(format!("Type information for field \"{:?}\" was missing or incorrect at at \"{:?}\"", field_name, path))
        }
    }
}