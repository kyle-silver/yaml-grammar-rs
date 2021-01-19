use lazy_static::lazy_static;
use regex::Regex;
use serde_yaml::{Mapping, Value};

use crate::{grammar::{PEType, ParseErr}, str_rule::{StrRule, StringRule}, value_ref::{ValueRef, ValueResolutionErr}};
use crate::valstr;

#[derive(Debug)]
pub enum StrConstr<'a> {
    Allowed(Vec<ValueRef<'a, String>>),
    Disallowed(Vec<ValueRef<'a, String>>),
    Regex(Box<Regex>),
    Equals(ValueRef<'a, String>),
    NotEquals(ValueRef<'a, String>),
    Any,
}

#[derive(Debug)]
pub struct StringConstraint<'a> {
    pub field_name: &'a Value,
    pub constr: StrConstr<'a>,
    pub default: Option<&'a String>
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
    default: Option<&'a String>
}

impl<'a, 'b> StrConstrBuilder<'a, 'b> {
    fn new(field_name: &'a Value, map: &'a Mapping, path: &'b [&'a Value]) -> Result<StrConstrBuilder<'a, 'b>, ParseErr<'a>> {
        let default = StrConstrBuilder::field_default(map, path)?;
        Ok(StrConstrBuilder { field_name, map, path, default })
    }

    fn field_default(map: &'a Mapping, path: &'b [&'a Value]) -> Result<Option<&'a String>, ParseErr<'a>> {
        lazy_static! {
            static ref DEFAULT: Value = valstr!(String::from("default"));
        }
        if let Some(val) = map.get(&DEFAULT) {
            match val {
                Value::String(s) => Ok(Some(s)),
                _ => Err(ParseErr::new(path, PEType::InvalidDefault(val)))
            }
        } else {
            Ok(None)
        }
    }

    fn from_mapping(&self) -> Result<StringConstraint<'a>, ParseErr<'a>> {
        lazy_static! {
            static ref ALLOWED: Value = valstr!(String::from("allowed"));
            static ref DISALLOWED: Value = valstr!(String::from("disallowed"));
            static ref REGEX: Value = valstr!(String::from("regex"));
            static ref EQ: Value = valstr!(String::from("eq"));
            static ref NEQ: Value = valstr!(String::from("neq"));
            static ref DEFAULT: Value = valstr!(String::from("default"));
        }
        if let Some(val) = self.map.get(&REGEX) {
            return self.regex(val);
        }
        if let Some(val) = self.map.get(&ALLOWED) {
            return self.allowed(val);
        }
        if let Some(val) = self.map.get(&DISALLOWED) {
            return self.disallowed(val);
        }
        if let Some(val) = self.map.get(&EQ) {
            return self.eq(val);
        }
        if let Some(val) = self.map.get(&NEQ) {
            return self.neq(val);
        }
        if self.default.is_some() {
            return Ok(StringConstraint::new(self.field_name, StrConstr::Any, self.default));
        }
        Ok(StringConstraint::default(self.field_name))
    }

    fn regex(&self, re: &'a Value) -> Result<StringConstraint<'a>, ParseErr<'a>> {
        if let Value::String(re) = re {
            match Regex::new(re) {
                Ok(regex) => Ok(StringConstraint { 
                    field_name: self.field_name,
                    constr: StrConstr::Regex(Box::new(regex)),
                    default: self.default,
                }),
                Err(e) => Err(ParseErr::new(self.path, PEType::Regex(e)))
            }
        } else {
            Err(ParseErr::new(self.path, PEType::IncorrectType(re)))
        }
    }

    fn allowed(&self, allowed: &'a Value) -> Result<StringConstraint<'a>, ParseErr<'a>> {
        if let Value::Sequence(seq) = allowed {
            let res = seq.iter().map(ValueRef::new).collect();
            match res {
                Ok(vals) => Ok(StringConstraint::new(self.field_name, StrConstr::Allowed(vals), self.default)),
                Err(err) => Err(ParseErr::new(self.path, err)),
            }
        } else {
            Err(ParseErr::new(self.path, PEType::IncorrectType(allowed)))
        }
    }

    fn disallowed(&self, disallowed: &'a Value) -> Result<StringConstraint<'a>, ParseErr<'a>> {
        if let Value::Sequence(seq) = disallowed {
            let res = seq.iter().map(ValueRef::new).collect();
            match res {
                Ok(vals) => Ok(StringConstraint::new(self.field_name, StrConstr::Disallowed(vals), self.default)),
                Err(err) => Err(ParseErr::new(self.path, err)),
            }
        } else {
            Err(ParseErr::new(self.path, PEType::IncorrectType(disallowed)))
        }
    }

    fn eq(&self, to: &'a Value) -> Result<StringConstraint<'a>, ParseErr<'a>> {
        match ValueRef::new(to) {
            Ok(vr) => Ok(StringConstraint::new(self.field_name, StrConstr::Equals(vr), self.default)),
            Err(err) => Err(ParseErr::new(self.path, err)),
        }
    }

    fn neq(&self, to: &'a Value) -> Result<StringConstraint<'a>, ParseErr<'a>> {
        match ValueRef::new(to) {
            Ok(vr) => Ok(StringConstraint::new(self.field_name, StrConstr::NotEquals(vr), self.default)),
            Err(err) => Err(ParseErr::new(self.path, err)),
        }
    }
}

impl<'a> ValueRef<'a, String> {
    fn new(value: &'a Value) -> Result<ValueRef<'a, String>, PEType<'a>> {
        match value {
            Value::String(literal) => Ok(ValueRef::Literal(literal)),
            Value::Sequence(abs_path) => ValueRef::abs_path(abs_path),    
            _ => Err(PEType::IncorrectType(value))       
        }
    }
}

pub fn build<'a>(field_name: &'a Value, map: &'a Mapping, path: &[&'a Value]) -> Result<StringConstraint<'a>, ParseErr<'a>> {
    StrConstrBuilder::new(field_name, map, path)?.from_mapping()
}