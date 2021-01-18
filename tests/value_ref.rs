use std::fs;

use serde_yaml::{Mapping, Value};
use yaml_grammar_rs::{grammar::{Constraint, YamlParseResult}, str_constr::StrConstr};


#[test]
fn deep_path() {
    let raw = fs::read_to_string("tests/res/deep_path.yamlfmt").unwrap();
    let spec: Mapping = serde_yaml::from_str(&raw).unwrap();
    let raw = fs::read_to_string("tests/res/deep_path.yaml").unwrap();
    let user: Value = serde_yaml::from_str(&raw).unwrap();
    for (k,v) in spec {
        let res = Constraint::parse(&k, &v, &vec![]);
        println!("{:?} -> {:?}: {:?}", k, v, res);
        if let YamlParseResult::Single(Ok(Constraint::Str(s))) = res {
            if let StrConstr::Equals(vr) = s.constr {
                println!("{:?}", vr.resolve(&user))
            }
        }
    }
}