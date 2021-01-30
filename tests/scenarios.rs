use serde_yaml::{Mapping, Number, Value};
use yaml_grammar::{Evaluation, rule::{RuleErrType, RuleEvalErr, RuleEvalSuccess}, valstr, yamlfmt};

mod utils;

#[test]
pub fn nested_syntactically_valid() {
    let spec: Mapping = utils::spec("nested-strings");
    let input: Value = utils::input("nested-strings", "input.yaml");
    let name = valstr!(".");
    let eval = yamlfmt(&spec, &input, &name);

    if let Evaluation::Completed { ok, err } = eval {
        // successes
        assert_eq!(3, ok.len());
        assert!(ok.contains(&RuleEvalSuccess::new(false, &valpath![".", "parent", "hello"])));
        assert!(ok.contains(&RuleEvalSuccess::new(true, &valpath![".", "parent", "world"])));
        assert!(ok.contains(&RuleEvalSuccess::new(true, &valpath![".", "parent", "nested", "foobar"])));
        // errors
        assert_eq!(1, err.len());
        assert!(err.contains(&RuleEvalErr::new(&valpath![".", "other"], RuleErrType::IncorrectType(&valnum!(7)))))
    } else {
        panic!("Result was not `Evaluation::Completed`")
    }
}

#[test]
pub fn missing_nested() {
    let spec: Mapping = utils::spec("nested-strings");
    let input: Value = utils::input("nested-strings", "invalid-01.yaml");
    let name = valstr!(".");
    let eval = yamlfmt(&spec, &input, &name);

    if let Evaluation::Completed { ok, err } = eval {
        // successes
        assert_eq!(2, ok.len());
        assert!(ok.contains(&RuleEvalSuccess::new(false, &valpath![".", "parent", "hello"])));
        assert!(ok.contains(&RuleEvalSuccess::new(true, &valpath![".", "parent", "world"])));
        // errors
        assert_eq!(2, err.len());
        assert!(err.contains(&RuleEvalErr::new(&valpath!["."], RuleErrType::KeyNotFound(&valstr!("other")))));
        assert!(err.contains(&RuleEvalErr::new(&valpath![".", "parent", "nested"], RuleErrType::KeyNotFound(&valstr!("foobar")))));
    } else {
        panic!("Result was not `Evaluation::Completed`")
    }
}

#[test]
pub fn compare_against_default() {
    let spec: Mapping = utils::spec("default-values");
    let input: Value = utils::input("default-values", "missing-optional.yaml");
    let name = valstr!(".");
    let eval = yamlfmt(&spec, &input, &name);
    println!("{:?}", eval);
}