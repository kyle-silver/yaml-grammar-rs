use serde_yaml::Value;

use crate::{bubble::Bubble, value_ref::ValueResolutionErr};

pub type RuleEvalResult<'a> = Bubble<Result<RuleEvalSuccess<'a>, RuleEvalErr<'a>>>;

#[derive(Debug)]
pub enum RuleErrType<'a> {
    MissingRequired,
    KeyNotFound(&'a Value),
    IncorrectType(&'a Value),
    Resolution(ValueResolutionErr<'a>),
}

impl<'a> From<ValueResolutionErr<'a>> for RuleErrType<'a> {
    fn from(vre: ValueResolutionErr<'a>) -> Self {
        RuleErrType::Resolution(vre)
    }
}

#[derive(Debug)]
pub struct RuleEvalErr<'a> {
    path: Vec<&'a Value>,
    err: RuleErrType<'a>,
}

impl<'a> RuleEvalErr<'a> {
    pub fn new(path: &[&'a Value], err: RuleErrType<'a>) -> RuleEvalErr<'a> {
        RuleEvalErr { path: path.to_vec(), err }
    }
}

impl<'a> From<RuleEvalErr<'a>> for RuleEvalResult<'a> {
    fn from(ree: RuleEvalErr<'a>) -> Self {
        Bubble::Single(Err(ree))
    }
}


#[derive(Debug)]
pub struct RuleEvalSuccess<'a> {
    result: bool,
    path: Vec<&'a Value>,
}

impl<'a> RuleEvalSuccess<'a> {
    pub fn new(result: bool, path: &[&'a Value]) -> RuleEvalSuccess<'a> {
        RuleEvalSuccess { result, path: path.to_vec(), }
    }
}

impl<'a> From<RuleEvalSuccess<'a>> for RuleEvalResult<'a> {
    fn from(res: RuleEvalSuccess<'a>) -> Self {
        Bubble::Single(Ok(res))
    }
}
