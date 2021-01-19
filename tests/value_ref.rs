use std::fs;

use serde_yaml::{Mapping, Value};
use yaml_grammar_rs::{grammar::{Constraint, YamlParseResult}, str_constr::StrConstr, str_rule::StringRule};


#[test]
fn deep_path() {
    let raw = fs::read_to_string("tests/res/deep_path.yamlfmt").unwrap();
    let spec: Mapping = serde_yaml::from_str(&raw).unwrap();
    let raw = fs::read_to_string("tests/res/deep_path.yaml").unwrap();
    let user: Value = serde_yaml::from_str(&raw).unwrap();
    for (k,v) in spec {
        let res = Constraint::parse(&k, &v, &vec![]);
        println!("{:?} -> {:?}: {:?}", k, v, res);
        if let YamlParseResult::Single(Ok(Constraint::Str(s))) = &res {
            let rule = StringRule::new(s, &user);
            println!("rule resolution: {:?}", rule);
            if let Ok(sr) = rule {
                if let Value::Mapping(m) = &user {
                    if let Some(value) = m.get(&k) {
                        println!("{:?}", sr.eval(value, &vec![]));
                    }
                }
            }
        }
    }
}