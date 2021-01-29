use serde_yaml::{Mapping, Number, Sequence, Value};

use crate::{constraint::Constraint, parse::PEType};

#[macro_export]
macro_rules! lit {
    ($val:expr) => {
        ValueRef::Literal(&String::from($val))
    };
}

/// These errors occur in the case where a constraint references a field's value,
/// but that field is not supplied by the user; in which case, we check to see if
/// the yamlfmt provides a default value which can be substituted during value
/// resolution. 
/// The paths here (represented as vectors of values) are residual, meaning that
/// they are not the complete path. The paths are only from the point of error
/// onwards. The complete path is present in the yamlfmt, and if the caller receives
/// any of these errors, then they necessarily have access to the absolute path.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DefaultFetchErr<'a> {
    IncorrectType {
        residual_path: Vec<&'a Value>,
        constr: Constraint<'a>
    },
    KeyNotFound(Vec<&'a Value>),
    ConstraintIsAny(Vec<&'a Value>),
    PathIsTooShort(Vec<&'a Value>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ValueResolutionErr<'a> {
    TooShort,
    TooLong,
    NotFound(Vec<&'a Value>),
    NonTerminalType(&'a Value),
    IncorrectType(&'a Value),
    Unimplemented,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueRef<'a, T> {
    Literal(&'a T),
    AbsolutePath(Vec<&'a Value>)
}

impl<'a, T> ValueRef<'a, T> {
    pub fn abs_path(path: &'a [Value]) -> Result<ValueRef<'a, T>, PEType<'a>> {
        let res = path.iter()
            .map(|val| match val {
                Value::Bool(_) => Ok(val),
                Value::Number(_) => Ok(val),
                Value::String(_) => Ok(val),
                _ => Err(PEType::InvalidAbsolutePath(val)),
            })
            .collect();
        match res {
            Ok(abs_path) => Ok(ValueRef::AbsolutePath(abs_path)),
            Err(e) => Err(e)
        }
    }

    fn resolve_with(&self, root: &'a Value, to_type: fn(&'a Value) -> Option<&'a T>) -> Result<&'a T, ValueResolutionErr<'a>> {
        match self {
            ValueRef::Literal(literal) => Ok(*literal),
            ValueRef::AbsolutePath(abs_path) => {
                let mut iter = abs_path.iter().peekable();
                let mut curr = root;
                while let Some(next) = iter.next() {
                    if let Value::Mapping(m) = curr {
                        if let Some(val) = m.get(next) {
                            curr = val;
                        } else {
                            return Err(ValueResolutionErr::NotFound(abs_path.clone()));
                        }
                    } else {
                        return if iter.peek().is_none() {
                            Err(ValueResolutionErr::TooShort)
                        } else {
                            Err(ValueResolutionErr::TooLong)
                        };
                    }
                }
                match to_type(curr) {
                    Some(val) => Ok(val),
                    None => Err(ValueResolutionErr::IncorrectType(curr))
                }
            }
        }  
    }
}

impl<'a> ValueRef<'a, String> {
    pub fn resolve(&self, root: &'a Value) -> Result<&'a String, ValueResolutionErr<'a>> {
        self.resolve_with(root, |v| match v {
            Value::String(s) => Some(s),
            _ => None,
        })
    }
}

impl<'a> ValueRef<'a, Number> {
    pub fn resolve(&self, root: &'a Value) -> Result<&'a Number, ValueResolutionErr<'a>> {
        self.resolve_with(root, |v| match v {
            Value::Number(n) => Some(n),
            _ => None,
        })
    }
}

impl<'a> ValueRef<'a, bool> {
    pub fn resolve(&self, root: &'a Value) -> Result<&'a bool, ValueResolutionErr<'a>> {
        self.resolve_with(root, |v| match v {
            Value::Bool(b) => Some(b),
            _ => None,
        })
    }
}

impl<'a> ValueRef<'a, Mapping> {
    pub fn resolve(&self, root: &'a Value) -> Result<&'a Mapping, ValueResolutionErr<'a>> {
        self.resolve_with(root, |v| match v {
            Value::Mapping(m) => Some(m),
            _ => None,
        })
    }
}

impl<'a> ValueRef<'a, Sequence> {
    pub fn resolve(&self, root: &'a Value) -> Result<&'a Sequence, ValueResolutionErr<'a>> {
        self.resolve_with(root, |v| match v {
            Value::Sequence(seq) => Some(seq),
            _ => None,
        })
    }
}