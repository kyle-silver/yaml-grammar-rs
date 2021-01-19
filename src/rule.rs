use serde_yaml::Value;

use crate::value_ref::ValueResolutionErr;

#[derive(Debug)]
pub enum RuleErrType<'a> {
    MissingRequired,
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


#[derive(Debug)]
pub struct RuleEvalSuccess<'a> {
    path: Vec<&'a Value>,
    result: bool,
}

impl<'a> RuleEvalSuccess<'a> {
    pub fn new(path: &[&'a Value], result: bool) -> RuleEvalSuccess<'a> {
        RuleEvalSuccess { path: path.to_vec(), result }
    }
}

#[derive(Debug)]
pub enum RuleEvalResult<'a> {
    Single(Result<RuleEvalSuccess<'a>, RuleEvalErr<'a>>),
    Multi(Vec<Result<RuleEvalSuccess<'a>, RuleEvalErr<'a>>>),
}

impl<'a> RuleEvalResult<'a> {
    pub fn err(path: &[&'a Value], err: RuleErrType<'a>) -> RuleEvalResult<'a> {
        RuleEvalResult::Single(Err(RuleEvalErr::new(path, err)))
    }

    pub fn suc(result: bool, path: &[&'a Value]) -> RuleEvalResult<'a> {
        RuleEvalResult::Single(Ok(RuleEvalSuccess::new(path, result)))
    }

    pub fn from(res: Result<RuleEvalSuccess<'a>, RuleEvalErr<'a>>) -> RuleEvalResult<'a> {
        RuleEvalResult::Single(res)
    }
}

impl<'a> IntoIterator for RuleEvalResult<'a> {
    type Item = Result<RuleEvalSuccess<'a>, RuleEvalErr<'a>>;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            RuleEvalResult::Single(res) => vec![res].into_iter(),
            RuleEvalResult::Multi(v) => v.into_iter(),
        }
    }
}