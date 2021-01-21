use std::collections::HashMap;

use serde_yaml::{Mapping, Value};

use crate::grammar::Constraint;

#[derive(Debug, PartialEq, Eq)]
pub enum ObjConstr<'a> {
    Fields(HashMap<&'a Value, Constraint<'a>>),
    Any,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ObjectConstraint<'a> {
    field_name: &'a Value,
    constr: ObjConstr<'a>,
    default: Option<&'a Mapping>,
}

impl<'a> ObjectConstraint<'a> {
    pub fn default(field_name: &Value) -> ObjectConstraint {
        ObjectConstraint { field_name, constr: ObjConstr::Any, default: None }
    }

    fn new(field_name: &'a Value, constr: ObjConstr<'a>, default: Option<&'a Mapping>) -> ObjectConstraint<'a> {
        ObjectConstraint { field_name, constr, default }
    }
}