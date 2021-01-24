use serde_yaml::{Mapping, Number, Value};

use crate::value_ref::ValueRef;

#[derive(Debug)]
pub enum NumConstr<'a> {
    Allowed(Vec<ValueRef<'a, Number>>),
    Disallowed(Vec<ValueRef<'a, Number>>),
    Range { min: ValueRef<'a, Number>, max: ValueRef<'a, Number>, },
    Equals(ValueRef<'a, Number>),
    NotEquals(ValueRef<'a, Number>),
    GreaterThan(ValueRef<'a, Number>),
    GreaterThanEq(ValueRef<'a, Number>),
    LessThan(ValueRef<'a, Number>),
    LessThanEq(ValueRef<'a, Number>),
    Any,
}

#[derive(Debug)]
pub struct NumberConstraint<'a> {
    pub field_name: &'a Value,
    pub constr: NumConstr<'a>,
    pub default: Option<&'a Number>
}

impl<'a> NumberConstraint<'a> {
    pub fn default(field_name: &Value) -> NumberConstraint {
        NumberConstraint { field_name, constr: NumConstr::Any, default: None }
    }

    fn new(field_name: &'a Value, constr: NumConstr<'a>, default: Option<&'a Number>) -> NumberConstraint<'a> {
        NumberConstraint { field_name, constr, default }
    }
}

#[derive(Debug)]
struct NumConstrBuilder<'a, 'b> {
    field_name: &'a Value,
    map: &'a Mapping,
    path: &'b [&'a Value],
    default: Option<&'a Number>
}