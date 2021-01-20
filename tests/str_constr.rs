use serde_yaml::{Mapping, Value};
use str_constr::StrConstr;
use yaml_grammar_rs::{grammar::{PEType, ParseErr}, str_constr};
use yaml_grammar_rs::value_ref::ValueRef;

macro_rules! valstr {
    ($val:expr) => {
        Value::String(String::from($val))
    };
}

macro_rules! lit {
    ($val:expr) => {
        ValueRef::Literal(&String::from($val))
    };
}

#[test]
fn str_eq_valid() {
    let raw = concat!(
        "type: string\n",
        "eq: hello",
    );
    let map: Mapping = serde_yaml::from_str(raw).unwrap();
    let name = valstr!("f");
    let acutal = str_constr::build(&name, &map, &vec![]);
    if let Ok(string_constraint) = acutal {
        assert_eq!(string_constraint.field_name, &valstr!("f"));
        assert_eq!(string_constraint.constr, StrConstr::Equals(lit!("hello")));
        assert_eq!(string_constraint.default, None);
    } else {
        panic!("didn't parse valid input")
    }
}

#[test]
fn str_eq_invalid() {
    let raw = concat!(
        "type: string\n",
        "eq: 7",
    );
    let map: Mapping = serde_yaml::from_str(raw).unwrap();
    let name = valstr!("f");
    let acutal = str_constr::build(&name, &map, &vec![]);
    if let Err(pe) = acutal {
        assert!(matches!(pe.err, PEType::IncorrectType(Value::Number(_))));
        assert_eq!(pe.path, Vec::<&Value>::new());
    } else {
        panic!()
    }
}