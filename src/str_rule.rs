use serde_yaml::Value;

use crate::{rule::{RuleErrType, RuleEvalResult}, str_constr::{StrConstr, StringConstraint, WrappedRegex}, value_ref::ValueResolutionErr};

#[derive(Debug)]
pub enum StrRule<'a> {
    Allowed(Vec<&'a String>),
    Disallowed(Vec<&'a String>),
    Regex(&'a Box<WrappedRegex>),
    Equals(&'a String),
    NotEquals(&'a String),
    Any,
}

impl<'a> StrRule<'a> {
    pub fn new(constr: &'a StrConstr, root: &'a Value) -> Result<StrRule<'a>, ValueResolutionErr<'a>> {
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
                Ok(StrRule::Regex(re))
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

#[derive(Debug)]
pub struct StringRule<'a> {
    field_name: &'a Value,
    rule: StrRule<'a>,
}

impl<'a> StringRule<'a> {
    pub fn new(constraint: &'a StringConstraint, root: &'a Value) -> Result<StringRule<'a>, ValueResolutionErr<'a>> {
        match StrRule::new(&constraint.constr, root) {
            Ok(rule) => Ok(StringRule {
                field_name: constraint.field_name,
                rule,
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
                    RuleEvalResult::suc(list.contains(&x), path)
                }
                StrRule::Disallowed(list) => {
                    RuleEvalResult::suc(!list.contains(&x), path)
                }
                StrRule::Regex(re) => {
                    RuleEvalResult::suc(re.is_match(x), path)
                }
                StrRule::Equals(other) => {
                    RuleEvalResult::suc(x == *other, path)
                }
                StrRule::NotEquals(other) => {
                    RuleEvalResult::suc(x != *other, path)
                }
                StrRule::Any => {
                    RuleEvalResult::suc(true, path)
                }
            }
        } else {
            RuleEvalResult::err(path, RuleErrType::IncorrectType(value))
        }
    }
}