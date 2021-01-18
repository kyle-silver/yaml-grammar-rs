use std::fs;
use grammar::Constraint;
use serde_yaml::Mapping;
use yaml_grammar_rs::grammar;

#[test]
fn basic_str() {
    let raw = fs::read_to_string("tests/res/str.yaml").unwrap();
    let spec: Mapping = serde_yaml::from_str(&raw).unwrap();
    for (k,v) in spec {
        println!("{:?}", Constraint::parse(&k, &v, &vec![]));
    }
    
}