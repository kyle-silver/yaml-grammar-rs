use serde_yaml::{Mapping, Number, Value};
use yaml_grammar::{Evaluation, rule::{RuleErrType, RuleEvalErr, RuleEvalSuccess}, valstr, yamlfmt};

mod utils;

#[test]
pub fn syntactically_valid() {
    let spec: Mapping = utils::spec("nested-strings");
    let input: Value = utils::input("nested-strings");
    println!("{:?}", spec);
    println!("{:?}", input);

    let name = valstr!(".");
    let eval = yamlfmt(&spec, &input, &name);

    if let Evaluation::Completed { ok, err } = eval {
        assert_eq!(3, ok.len());
        assert!(ok.contains(&RuleEvalSuccess::new(false, &valpath![".", "parent", "hello"])));
        assert!(ok.contains(&RuleEvalSuccess::new(true, &valpath![".", "parent", "world"])));
        assert!(ok.contains(&RuleEvalSuccess::new(true, &valpath![".", "parent", "nested", "foobar"])));

        assert_eq!(1, err.len());
        assert!(err.contains(&RuleEvalErr::new(&valpath![".", "other"], RuleErrType::IncorrectType(&valnum!(7)))))
    } else {
        panic!("Result was not `Evaluation::Completed`")
    }
}