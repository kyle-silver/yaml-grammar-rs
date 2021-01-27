use serde_yaml::{Mapping, Number, Value};
use yaml_grammar::{Evaluation, constraint::Constraint, evaluate, into_constraint, parse_grammar, rule::{RuleErrType, RuleEvalErr, RuleEvalSuccess}, rules, valstr};

mod utils;

#[test]
pub fn syntactically_valid() {
    let spec: Mapping = utils::spec("nested-strings");
    let input: Value = utils::input("nested-strings");
    println!("{:?}", spec);
    println!("{:?}", input);

    let yaml_res: Vec<_> = parse_grammar(&spec).into_iter()
            .map(Result::unwrap)
            .collect();      
        
    let name = valstr!(".");
    let obj_constr = into_constraint(yaml_res, &name);
    let obj_constr = Constraint::Obj(obj_constr);
    let rules = rules(&obj_constr, &input);
    let (ok, _): (Vec<_>, _) = rules.into_iter().partition(|r| r.is_ok());
    let ok = ok.into_iter().map(Result::unwrap).collect();
    let eval = evaluate(&ok, &input);

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