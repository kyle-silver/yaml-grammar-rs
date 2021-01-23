use std::collections::HashMap;

use grammar::YamlParseResult;
use lazy_static::lazy_static;
use serde_yaml::{Mapping, Value};

use crate::{grammar::{self, Constraint, PEType, ParseErr}, value_ref::ValueRef};
use crate::valstr;

#[derive(Debug, PartialEq, Eq)]
pub enum ObjConstr<'a> {
    Fields(HashMap<&'a Value, Constraint<'a>>),
    Any,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ObjectConstraint<'a> {
    pub field_name: &'a Value,
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

    pub fn add(&mut self, field_name: &'a Value, constraint: Constraint<'a>) {
        if let ObjConstr::Fields(map) = &mut self.constr {
            map.insert(&field_name, constraint);
        }
    }
}

impl<'a> From<ObjectConstraint<'a>> for YamlParseResult<'a> {
    fn from(c: ObjectConstraint<'a>) -> Self {
        YamlParseResult::Single(Ok(Constraint::Obj(c)))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ObjectConstraintBuilder<'a, 'b> {
    field_name: &'a Value,
    config: &'a Mapping,
    path: &'b [&'a Value],
    default: Option<&'a Mapping>
}

impl<'a, 'b> ObjectConstraintBuilder<'a, 'b> {
    fn new(field_name: &'a Value, config: &'a Mapping, path: &'b [&'a Value]) -> Result<Self, ParseErr<'a>> {
        let default = Self::field_default(config, path)?;
        Ok(Self { field_name, config, path, default })
    }

    fn field_default(config: &'a Mapping, path: &'b [&'a Value]) -> Result<Option<&'a Mapping>, ParseErr<'a>> {
        lazy_static! {
            static ref DEFAULT: Value = valstr!("default");
        }
        if let Some(val) = config.get(&DEFAULT) {
            match val {
                Value::Mapping(m) => Ok(Some(m)),
                _ => Err(ParseErr::new(path, PEType::InvalidDefault(val)))
            }
        } else {
            Ok(None)
        }
    }

    fn from_mapping(&self) -> YamlParseResult<'a> {
        lazy_static! {
            static ref FIELDS: Value = valstr!("fields");
        }
        if let Some(val) = self.config.get(&FIELDS) {
            return self.fields(val);
        }
        let default = ObjectConstraint::default(self.field_name);
        Constraint::Obj(default).into()
    }

    fn fields(&self, fields: &'a Value) -> YamlParseResult<'a> {
        if let Value::Mapping(f) = fields {
            let mut path = self.path.to_vec();
            path.push(self.field_name);
            let (ok, err): (Vec<_>, Vec<_>) = f.iter()
                .map(|(k, v)| Constraint::parse(k, v, &path))
                .partition(YamlParseResult::all_ok);
            // see if it was parsed without errors
            if err.is_empty() {
                let map = ok.into_iter()
                    .flatten()
                    .map(Result::unwrap)
                    .map(|c| (c.field_name(), c))
                    .collect();
                let constr = ObjConstr::Fields(map);
                ObjectConstraint::new(self.field_name, constr, self.default).into()
            } else {
                YamlParseResult::Multi(err.into_iter().flatten().collect())
            }
        } else {
            ParseErr::new(self.path, PEType::IncorrectType(fields)).into()
        }
    }
}

impl<'a> ValueRef<'a, Mapping> {
    fn new(value: &'a Value) -> Result<ValueRef<'a, Mapping>, PEType<'a>> {
        match value {
            Value::Mapping(literal) => Ok(ValueRef::Literal(literal)),
            Value::Sequence(path) => ValueRef::abs_path(path),
            _ => Err(PEType::IncorrectType(value)),
        }
    }
}

pub fn build<'a>(field_name: &'a Value, config: &'a Mapping, path: &[&'a Value]) -> YamlParseResult<'a> {
    match ObjectConstraintBuilder::new(field_name, config, path) {
        Ok(builder) => builder.from_mapping(),
        Err(e) => e.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lit, str_constr::StringConstraint};
    use crate::valstr;

    #[test]
    fn obj_constr_valid() {
        // yaml
        let raw = concat!(
            "type: object\n",
            "fields:\n",
            "  hello: string\n",
            "  world: string\n",
            "  nested:\n",
            "    type: object\n",
            "    fields:\n",
            "      foobar: string"
        );

        // expected structure in code
        // inner object constraint
        let name = valstr!("nested");
        let mut nested = ObjectConstraint::new(&name, ObjConstr::Fields(HashMap::new()), None);
        let name = valstr!("foobar");
        nested.add(&name, Constraint::Str(StringConstraint::default(&name)));
        // outer object constraint
        let name = valstr!("f");
        let mut expected = ObjectConstraint::new(&name, ObjConstr::Fields(HashMap::new()), None);
        let name = valstr!("nested");
        expected.add(&name, Constraint::Obj(nested));
        let name = valstr!("hello");
        expected.add(&name, Constraint::Str(StringConstraint::default(&name)));
        let name = valstr!("world");
        expected.add(&name, Constraint::Str(StringConstraint::default(&name)));

        // parse yaml and validate
        let config: Mapping = serde_yaml::from_str(raw).unwrap();
        let name = valstr!("f");
        let res = build(&name, &config, &vec![]);
        if let YamlParseResult::Single(Ok(Constraint::Obj(obj))) = res {
            assert_eq!(&String::from("f"), obj.field_name);
            assert_eq!(expected, obj);
            assert_eq!(None, obj.default);
        } else {
            panic!();
        }
    }
}