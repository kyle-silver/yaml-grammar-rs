use lazy_static::lazy_static;
use regex::Regex;
use serde_yaml::{Mapping, Value};
use std::ops::Deref;

use crate::{parse::{PEType, ParseErr}, rule::{Rule, RuleErrType, RuleEvalErr, RuleEvalResult, RuleEvalSuccess}, value_ref::{ValueRef, ValueResolutionErr}};
use crate::valstr;

// A wrapper type because Regex doesn't implement Eq or PartialEq. In fairness,
// most of the time you would never want to use a regex as a key -- but really we
// just want this for test coverage
#[derive(Debug, Clone)]
pub struct WrappedRegex(Regex);

impl Deref for WrappedRegex {
    type Target = Regex;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for WrappedRegex {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_str() == other.as_str()
    }
}

impl Eq for WrappedRegex {}

#[derive(Debug, PartialEq, Eq)]
pub enum StrConstr<'a> {
    Allowed(Vec<ValueRef<'a, String>>),
    Disallowed(Vec<ValueRef<'a, String>>),
    Regex(Box<WrappedRegex>),
    Equals(ValueRef<'a, String>),
    NotEquals(ValueRef<'a, String>),
    Any,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StringConstraint<'a> {
    pub field_name: &'a Value,
    pub constr: StrConstr<'a>,
    pub default: Option<&'a String>,
}

impl<'a> StringConstraint<'a> {
    pub fn default(field_name: &Value) -> StringConstraint {
        StringConstraint { field_name, constr: StrConstr::Any, default: None }
    }

    fn new(field_name: &'a Value, constr: StrConstr<'a>, default: Option<&'a String>) -> StringConstraint<'a> {
        StringConstraint { field_name, constr, default }
    }
}

#[derive(Debug)]
struct StringConstraintBuilder<'a, 'b> {
    field_name: &'a Value,
    config: &'a Mapping,
    path: &'b [&'a Value],
    default: Option<&'a String>
}

impl<'a, 'b> StringConstraintBuilder<'a, 'b> {
    fn new(field_name: &'a Value, config: &'a Mapping, path: &'b [&'a Value]) -> Result<Self, ParseErr<'a>> {
        let default = Self::field_default(config, path)?;
        Ok(Self { field_name, config, path, default })
    }

    fn field_default(map: &'a Mapping, path: &'b [&'a Value]) -> Result<Option<&'a String>, ParseErr<'a>> {
        lazy_static! {
            static ref DEFAULT: Value = valstr!("default");
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
            static ref ALLOWED: Value = valstr!("allowed");
            static ref DISALLOWED: Value = valstr!("disallowed");
            static ref REGEX: Value = valstr!("regex");
            static ref EQ: Value = valstr!("eq");
            static ref NEQ: Value = valstr!("neq");
            static ref DEFAULT: Value = valstr!("default");
        }
        if let Some(val) = self.config.get(&REGEX) {
            return self.regex(val);
        }
        if let Some(val) = self.config.get(&ALLOWED) {
            return self.allowed(val);
        }
        if let Some(val) = self.config.get(&DISALLOWED) {
            return self.disallowed(val);
        }
        if let Some(val) = self.config.get(&EQ) {
            return self.eq(val);
        }
        if let Some(val) = self.config.get(&NEQ) {
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
                    constr: StrConstr::Regex(Box::new(WrappedRegex(regex))),
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
            Value::Sequence(path) => ValueRef::abs_path(path),    
            _ => Err(PEType::IncorrectType(value))       
        }
    }
}

pub fn build<'a>(field_name: &'a Value, map: &'a Mapping, path: &[&'a Value]) -> Result<StringConstraint<'a>, ParseErr<'a>> {
    StringConstraintBuilder::new(field_name, map, path)?.from_mapping()
}

#[derive(Debug, Clone, PartialEq)]
pub enum StrRule<'a> {
    Allowed(Vec<&'a String>),
    Disallowed(Vec<&'a String>),
    Regex(Box<WrappedRegex>),
    Equals(&'a String),
    NotEquals(&'a String),
    Any,
}

impl<'a> StrRule<'a> {
    pub fn new(constr: StrConstr<'a>, root: &'a Value) -> Result<StrRule<'a>, ValueResolutionErr<'a>> {
        match constr {
            StrConstr::Allowed(v) => {
                let resolved: Result<_,_> = v.iter().map(|val| val.resolve(root)).collect();
                Ok(StrRule::Allowed(resolved?))
            }
            StrConstr::Disallowed(v) => {
                let resolved: Result<_,_> = v.iter().map(|val| val.resolve(root)).collect();
                Ok(StrRule::Disallowed(resolved?))
            }
            StrConstr::Regex(re) => {
                Ok(StrRule::Regex(Box::new(*re)))
            }
            StrConstr::Equals(vr) => {
                let resolved = vr.resolve(root)?;
                Ok(StrRule::Equals(resolved))
            }
            StrConstr::NotEquals(vr) => {
                let resolved = vr.resolve(root)?;
                Ok(StrRule::NotEquals(resolved))
            }
            StrConstr::Any => {
                Ok(StrRule::Any)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringRule<'a> {
    pub field_name: &'a Value,
    rule: StrRule<'a>,
    default: Option<&'a String>,
}

impl<'a> From<StringRule<'a>> for Rule<'a> {
    fn from(sr: StringRule<'a>) -> Self {
        Rule::Str(sr)
    }
}

impl<'a> StringRule<'a> {
    pub fn new(constraint: StringConstraint<'a>, root: &'a Value) -> Result<StringRule<'a>, ValueResolutionErr<'a>> {
        match StrRule::new(constraint.constr, root) {
            Ok(rule) => Ok(StringRule {
                field_name: constraint.field_name,
                rule,
                default: constraint.default,
            }),
            Err(ValueResolutionErr::NotFound) => {
                todo!("Implement search for default values in other rules")
            }
            Err(v) => Err(v)
        }
    }

    pub fn eval(&self, value: &'a Value, path: &[&'a Value]) -> RuleEvalResult<'a> {
        if let Value::String(x) = value {
            match &self.rule {
                StrRule::Allowed(list) => {
                    RuleEvalSuccess::new(list.contains(&x), path).into()
                }
                StrRule::Disallowed(list) => {
                    RuleEvalSuccess::new(!list.contains(&x), path).into()
                }
                StrRule::Regex(re) => {
                    RuleEvalSuccess::new(re.is_match(x), path).into()
                }
                StrRule::Equals(other) => {
                    RuleEvalSuccess::new(x == *other, path).into()
                }
                StrRule::NotEquals(other) => {
                    RuleEvalSuccess::new(x != *other, path).into()
                }
                StrRule::Any => {
                    RuleEvalSuccess::new(true, path).into()
                }
            }
        } else {
            RuleEvalErr::new(path, RuleErrType::IncorrectType(value)).into()
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_yaml::Number;

    use super::*;
    use crate::lit;
    use crate::valstr;
    
    #[test]
    fn str_eq_valid() {
        let raw = concat!(
            "type: string\n",
            "eq: hello",
        );
        let map: Mapping = serde_yaml::from_str(raw).unwrap();
        let name = valstr!("f");
        let acutal = build(&name, &map, &vec![]);
        if let Ok(string_constraint) = acutal {
            assert_eq!(string_constraint.field_name, &valstr!("f"));
            assert_eq!(string_constraint.constr, StrConstr::Equals(lit!("hello")));
            assert_eq!(string_constraint.default, None);
        } else {
            panic!("didn't parse valid input")
        }
    }
    
    #[test]
    fn str_eq_invalid() {
        let raw = concat!(
            "type: string\n",
            "eq: 7",
        );
        let map: Mapping = serde_yaml::from_str(raw).unwrap();
        let name = valstr!("f");
        let acutal = build(&name, &map, &vec![]);
        if let Err(pe) = acutal {
            assert_eq!(pe, ParseErr {
                err: PEType::IncorrectType(&Value::Number(Number::from(7))),
                path: Vec::<&Value>::new(),
            })
        } else {
            panic!()
        }
    }
}