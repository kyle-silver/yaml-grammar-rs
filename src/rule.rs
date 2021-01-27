use serde_yaml::Value;

use crate::{bubble::Bubble, constraint::Constraint, obj::ObjectRule, str::StringRule, value_ref::ValueResolutionErr};

pub type RuleEvalResult<'a> = Bubble<Result<RuleEvalSuccess<'a>, RuleEvalErr<'a>>>;

#[derive(Debug, Clone, PartialEq)]
pub enum RuleErrType<'a> {
    MissingRequired,
    KeyNotFound(&'a Value),
    IncorrectType(&'a Value),
    Resolution(ValueResolutionResult<'a>),
}

impl<'a> From<ValueResolutionResult<'a>> for RuleErrType<'a> {
    fn from(vrr: ValueResolutionResult<'a>) -> Self {
        RuleErrType::Resolution(vrr)
    }
}

#[derive(Debug, Clone, PartialEq)]
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


#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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
            Constraint::Obj(oc) => ObjectRule::new(oc, root),
        }
    }

    pub fn field_name(&self) -> &'a Value {
        match self {
            Rule::Str(sr) => sr.field_name,
            Rule::Obj(or) => or.field_name,
        }
    }

    pub fn eval(&'a self, value: &'a Value, parent_path: &[&'a Value]) -> RuleEvalResult<'a> {
        let mut path = parent_path.to_vec();
        path.push(self.field_name());
        match self {
            Rule::Str(sr) => sr.eval(value, &path),
            Rule::Obj(or) => or.eval(value, &path),
        }
    }
}