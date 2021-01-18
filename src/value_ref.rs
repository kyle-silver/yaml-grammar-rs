use serde_yaml::{Mapping, Number, Sequence, Value};

use crate::grammar::PEType;

#[derive(Debug)]
pub enum PathResolutionErr<'a> {
    TooShort,
    TooLong,
    NotFound,
    NonTerminalType(&'a Value),
    IncorrectType(&'a Value),
}

#[derive(Debug)]
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

    fn resolve_with(&self, root: &'a Value, to_type: fn(&'a Value) -> Option<&'a T>) -> Result<&'a T, PathResolutionErr<'a>> {
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
                            return Err(PathResolutionErr::NotFound);
                        }
                    } else {
                        return if iter.peek().is_none() {
                            Err(PathResolutionErr::TooShort)
                        } else {
                            Err(PathResolutionErr::TooLong)
                        };
                    }
                }
                match to_type(curr) {
                    Some(val) => Ok(val),
                    None => Err(PathResolutionErr::IncorrectType(curr))
                }
            }
        }  
    }
}

impl<'a> ValueRef<'a, String> {
    pub fn resolve(&self, root: &'a Value) -> Result<&'a String, PathResolutionErr<'a>> {
        self.resolve_with(root, |v| match v {
            Value::String(s) => Some(s),
            _ => None,
        })
    }
}

impl<'a> ValueRef<'a, Number> {
    pub fn resolve(&self, root: &'a Value) -> Result<&'a Number, PathResolutionErr<'a>> {
        self.resolve_with(root, |v| match v {
            Value::Number(n) => Some(n),
            _ => None,
        })
    }
}

impl<'a> ValueRef<'a, bool> {
    pub fn resolve(&self, root: &'a Value) -> Result<&'a bool, PathResolutionErr<'a>> {
        self.resolve_with(root, |v| match v {
            Value::Bool(b) => Some(b),
            _ => None,
        })
    }
}

impl<'a> ValueRef<'a, Mapping> {
    pub fn resolve(&self, root: &'a Value) -> Result<&'a Mapping, PathResolutionErr<'a>> {
        self.resolve_with(root, |v| match v {
            Value::Mapping(m) => Some(m),
            _ => None,
        })
    }
}

impl<'a> ValueRef<'a, Sequence> {
    pub fn resolve(&self, root: &'a Value) -> Result<&'a Sequence, PathResolutionErr<'a>> {
        self.resolve_with(root, |v| match v {
            Value::Sequence(seq) => Some(seq),
            _ => None,
        })
    }
}