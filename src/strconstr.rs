use lazy_static::lazy_static;
use regex::Regex;
use serde_yaml::{Mapping, Value};

use crate::grammar::{ParseErr, ValueRef};
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
    default: Option<&'a String>
}

impl<'a> StringConstraint<'a> {
    pub fn default(field_name: &Value) -> StringConstraint {
        StringConstraint { field_name, constr: StrConstr::Any, default: None }
    }

    fn new(field_name: &'a Value, constr: StrConstr<'a>, default: Option<&'a String>) -> StringConstraint<'a> {
        StringConstraint { field_name, constr, default }
    }
}

struct StrConstrBuilder<'a, 'b> {
    field_name: &'a Value,
    map: &'a Mapping,
    path: &'b [&'a Value],
}

impl<'a, 'b> StrConstrBuilder<'a, 'b> {
    fn new(field_name: &'a Value, map: &'a Mapping, path: &'b [&'a Value]) -> StrConstrBuilder<'a, 'b> {
        StrConstrBuilder { field_name, map, path, }
    }

    fn from_mapping(&self) -> Result<StringConstraint<'a>, ParseErr<'a>> {
        lazy_static! {
            static ref ALLOWED: Value = valstr!(String::from("regex"));
            static ref DISALLOWED: Value = valstr!(String::from("disallowed"));
            static ref REGEX: Value = valstr!(String::from("regex"));
            static ref EQ: Value = valstr!(String::from("eq"));
            static ref NEQ: Value = valstr!(String::from("neq"));
            static ref DEFAULT: Value = valstr!(String::from("default"));
        }
        let default = self.field_default()?;
        if let Some(val) = self.map.get(&REGEX) {
            return self.regex(val, default);
        }
        Ok(StringConstraint::default(self.field_name))
    }

    fn field_default(&self) -> Result<Option<&'a String>, ParseErr<'a>> {
        lazy_static! {
            static ref DEFAULT: Value = valstr!(String::from("default"));
        }
        if let Some(val) = self.map.get(&DEFAULT) {
            match val {
                Value::String(s) => Ok(Some(s)),
                _ => Err(ParseErr::new(self.path, format!("Default is an invalid type")))
            }
        } else {
            Ok(None)
        }
    }

    fn regex(&self, re: &'a Value, default: Option<&'a String>) -> Result<StringConstraint<'a>, ParseErr<'a>> {
        if let Value::String(re) = re {
            match Regex::new(re) {
                Ok(regex) => Ok(StringConstraint { 
                    field_name: self.field_name,
                    constr: StrConstr::Regex(regex),
                    default,
                }),
                Err(e) => Err(ParseErr::new(self.path, format!("Could not parse regex \"{}\", \n{}", re, e)))
            }
        } else {
            Err(ParseErr::new(self.path, String::from("Type error for key \"regex\"")))
        }
    }
}

pub fn build<'a>(field_name: &'a Value, map: &'a Mapping, path: &[&'a Value]) -> Result<StringConstraint<'a>, ParseErr<'a>> {
    StrConstrBuilder::new(field_name, map, path).from_mapping()
}
