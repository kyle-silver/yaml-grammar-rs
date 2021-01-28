use std::collections::HashMap;

use parse::YamlParseResult;
use lazy_static::lazy_static;
use serde_yaml::{Mapping, Value};

use crate::{bubble::Bubble, constraint::Constraint, parse::{self, PEType, ParseErr}, rule::{Rule, RuleErrType, RuleEvalErr, RuleEvalResult, RuleEvalSuccess, ValueResolutionResult}};
use crate::valstr;

#[derive(Debug, PartialEq, Eq)]
pub enum ObjConstr<'a> {
    Fields(HashMap<&'a Value, Constraint<'a>>),
    Any,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ObjectConstraint<'a> {
    pub field_name: &'a Value,
    pub constr: ObjConstr<'a>,
    default: Option<&'a Mapping>,
}

impl<'a> ObjectConstraint<'a> {
    pub fn default(field_name: &Value) -> ObjectConstraint {
        ObjectConstraint { field_name, constr: ObjConstr::Any, default: None }
    }

    pub fn new(field_name: &'a Value, constr: ObjConstr<'a>, default: Option<&'a Mapping>) -> ObjectConstraint<'a> {
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

    fn path(&self) -> Vec<&'a Value> {
        let mut path = self.path.to_vec();
        path.push(self.field_name);
        path
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
                .partition(|b| b.all(Result::is_ok));
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
            ParseErr::new(&self.path(), PEType::IncorrectType(fields)).into()
        }
    }
}

pub fn build<'a>(field_name: &'a Value, config: &'a Mapping, path: &[&'a Value]) -> YamlParseResult<'a> {
    match ObjectConstraintBuilder::new(field_name, config, path) {
        Ok(builder) => builder.from_mapping(),
        Err(e) => e.into()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjRule<'a> {
    Fields(HashMap<&'a Value, Rule<'a>>),
    Any,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectRule<'a> {
    pub field_name: &'a Value,
    rule: ObjRule<'a>,
    default: Option<&'a Mapping>,
}

impl<'a> ObjectRule<'a> {
    pub fn new(constraint: ObjectConstraint<'a>, root: &'a Value) -> ValueResolutionResult<'a> {
        match constraint.constr {
            ObjConstr::Fields(constraints) => {
                let (ok, err): (Vec<_>, Vec<_>) = constraints.into_iter()
                    .map(|(_, c)| Rule::new(c, root))
                    .partition(|b| b.all(Result::is_ok));
                if err.is_empty() {
                    let map = ok.into_iter()
                        .flatten()
                        .map(Result::unwrap)
                        .map(|r| (r.field_name(), r))
                        .collect();
                    let rule = ObjRule::Fields(map);
                    let object_rule = ObjectRule { field_name: constraint.field_name, rule, default: constraint.default };
                    Bubble::Single(Ok(Rule::Obj(object_rule)))
                } else {
                    Bubble::Multi(err.into_iter().flatten().collect())
                }
            }
            ObjConstr::Any => {
                let object_rule = ObjectRule { field_name: constraint.field_name, rule: ObjRule::Any, default: constraint.default };
                Bubble::Single(Ok(Rule::Obj(object_rule)))
            },
        }
    }

    pub fn eval(self, value: &'a Value, path: &[&'a Value]) -> RuleEvalResult<'a> {
        // this doesn't work for the very top level of rules
        // that evaluation is treated as a special case and done in a separate loop
        if let Value::Mapping(mapping) = value {
            match self.rule {
                ObjRule::Fields(rules) => {
                    let results: Vec<_> = rules.into_iter()
                        .map(|(key, rule)| ObjectRule::subrule(key, rule, mapping, path))
                        .collect();
                    results.into()
                }
                ObjRule::Any => {
                    RuleEvalSuccess::new(true, path).into()
                }
            }
        } else {
            RuleEvalErr::new(path, RuleErrType::IncorrectType(value)).into()
        }
    }

    pub fn subrule(key: &'a Value, rule: Rule<'a>, input: &'a Mapping, path: &[&'a Value]) -> RuleEvalResult<'a> {
        let keys: Vec<_> = input.iter().map(|(k,_)| k).collect();
        println!("Key: {:?}, Mapping: {:?}", key, keys);
        if let Some(value) = input.get(key) {
            rule.eval(value, path)
        } else {
            RuleEvalErr::new(path, RuleErrType::KeyNotFound(key)).into()
        }
    }
}

#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::*;
    use crate::str::StringConstraint;
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
            panic!("parse of valid input failed");
        }
    }

    #[test]
    fn obj_constr_invalid() {
        let raw = concat!(
            "type: object\n",
            "fields:\n",
            "  hello: stringerino\n",
            "  world: string\n",
        );

        let config: Mapping = serde_yaml::from_str(raw).unwrap();
        let name = valstr!("parent");
        let results = build(&name, &config, &vec![]).get();
        assert_eq!(results.len(), 1);

        let pe = results.into_iter().next()
            .expect("expected one error")
            .expect_err("First entry should be an error");
        let (p1, p2) = (valstr!("parent"), valstr!("hello"));
        let expected = ParseErr::new(&vec![&p1, &p2], PEType::UnknownType("stringerino"));
        assert_eq!(expected, pe);
    }

    #[test]
    fn obj_constr_multiple_invalid() {
        let raw = concat!(
            "type: object\n",
            "fields:\n",
            "  hello: stringerino\n",
            "  world:\n",
            "    type: string\n",
            "    regex: ^\\d{{{{$",
        );

        let config: Mapping = serde_yaml::from_str(raw).unwrap();
        let name = valstr!("parent");
        let results = build(&name, &config, &vec![]).get();
        assert_eq!(results.len(), 2);

        let (p1, p2) = (valstr!("parent"), valstr!("hello"));
        let hello_type_error = ParseErr::new(&vec![&p1, &p2], PEType::UnknownType("stringerino"));
        assert!(results.contains(&Err(hello_type_error)));

        let (p1, p2) = (valstr!("parent"), valstr!("world"));
        let world_regex_error = ParseErr::new(&vec![&p1, &p2], Regex::new("^\\d{{{{$").unwrap_err().into());
        assert!(results.contains(&Err(world_regex_error)));
    }

    #[test]
    fn obj_empty_fields() {
        let raw = concat!(
            "type: object\n",
            "fields:\n",
        );
        
        let config: Mapping = serde_yaml::from_str(raw).unwrap();
        let name = valstr!("parent");
        let results = build(&name, &config, &vec![]).get();
        assert_eq!(results.len(), 1);

        let pe = results.into_iter().next()
            .expect("expected one error")
            .expect_err("First entry should be an error");
        let expected = ParseErr::new(&vec![&name], PEType::IncorrectType(&Value::Null));
        assert_eq!(expected, pe);
    }

    #[test]
    fn resolution_test() {
        let spec = concat!(
            "type: object\n",
            "fields:\n",
            "  hello:\n",
            "    type: string\n",
            "    eq: [parent, world]\n",
            "  world: string\n",
            "  nested:\n",
            "    type: object\n",
            "    fields:\n",
            "      foobar: string\n"
        );

        let input = concat!(
            "parent:\n",
            "  hello: foo\n",
            "  world: bar\n",
            "  nested:\n",
            "    foobar: fizzbuzz\n"
        );

        let config: Mapping = serde_yaml::from_str(spec).unwrap();
        let name = valstr!("parent");
        let constraints: Vec<Constraint> = build(&name, &config, &vec![]).get().into_iter()
            .map(Result::unwrap)
            .collect();

        let input: Value = serde_yaml::from_str(input).unwrap();
        println!("{:?}", input);
        for constraint in constraints {
            let results = Rule::new(constraint, &input);
            println!("results: {:?}", results);
            for res in results.get() {
                if let Ok(rule) = res {
                    println!("{:?}", rule.eval(&input.get(valstr!("parent")).unwrap(), &vec![]));
                }
            }
        } 
        
        
    }
}