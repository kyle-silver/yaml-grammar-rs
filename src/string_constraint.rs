use lazy_static::lazy_static;
use regex::Regex;
use serde_yaml::{Mapping, Value};

use crate::grammar::ValueRef;
use crate::valstr;

#[derive(Debug)]
pub enum StrConstr<'a> {
    Allowed(Vec<ValueRef<'a, String>>),
    Disallowed(Vec<ValueRef<'a, String>>),
    Regex(Regex),
    Equals(ValueRef<'a, String>),
    NotEquals(ValueRef<'a, String>),
    Any,
}

#[derive(Debug)]
pub struct StringConstraint<'a> {
    field_name: &'a Value,
    constr: StrConstr<'a>,
    default: Option<String>
}

impl<'a> StringConstraint<'a> {
    pub fn default(field_name: &Value) -> StringConstraint {
        StringConstraint { field_name, constr: StrConstr::Any, default: None }
    }

    pub fn from_mapping(field_name: &'a Value, map: &'a Mapping, path: &[&'a Value]) -> Result<StringConstraint<'a>, String> {
        lazy_static! {
            static ref ALLOWED: Value = valstr!(String::from("regex"));
            static ref DISALLOWED: Value = valstr!(String::from("disallowed"));
            static ref REGEX: Value = valstr!(String::from("regex"));
            static ref EQ: Value = valstr!(String::from("eq"));
            static ref NEQ: Value = valstr!(String::from("neq"));
            static ref DEFAULT: Value = valstr!(String::from("default"));
        }
        let str_default = map.get(&DEFAULT);
        if let Some(val) = map.get(&REGEX) {
            return StringConstraint::regex(field_name, val, path);
        }
        Ok(StringConstraint::default(field_name))
    }

    fn regex(field_name: &'a Value, re: &'a Value, path: &[&'a Value]) -> Result<StringConstraint<'a>, String> {
        if let Value::String(re) = re {
            match Regex::new(re) {
                Ok(regex) => Ok(StringConstraint { 
                    field_name,
                    constr: StrConstr::Regex(regex),
                    default: None,
                }),
                Err(e) => Err(format!("Could not parse regex \"{}\" at \"{:?}\", \"{}\"", re, path, e))
            }
        } else {
            Err(format!("Type error for key \"regex\" at \"{:?}\"", path))
        }
    }
}