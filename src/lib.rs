use bubble::Bubble;
use constraint::Constraint;
use parse::YamlParseResult;
use obj::{ObjConstr, ObjectConstraint, ObjectRule};
use rule::{Rule, RuleErrType, RuleEvalErr, RuleEvalResult, ValueResolutionResult};
use serde_yaml::{Mapping, Value};

// public API
pub mod parse;
pub mod num;
pub mod str;
pub mod obj;
pub mod rule;
pub mod value_ref;
pub mod bubble;
pub mod constraint;

pub fn parse_grammar(spec: &Mapping) -> YamlParseResult {
    let res: Vec<_> = spec.iter()
        .map(|(k, v)| Constraint::parse(k, v, &vec![]))
        .collect();
    res.into()
}

pub fn into_rule<'a>(constraints: Vec<Constraint<'a>>, name: &'a Value) -> ObjectConstraint<'a> {
    let map = constraints.into_iter().map(|c| (c.field_name(), c)).collect();
    ObjectConstraint::new(name, ObjConstr::Fields(map), None)
}

pub fn evaluate<'a>(spec: &'a Constraint, user_input: &'a Value) {
    let (ok, err): (Vec<_>, Vec<_>) = Rule::new(spec, user_input).get()
        .into_iter()
        .partition(Result::is_ok);
    if !err.is_empty() {
        let err_type: RuleErrType = RuleErrType::Resolution(Bubble::Multi(err));
        // return RuleEvalErr::new(&vec![], err_type).into();
        println!("{:?}", err_type);
        return;
    }
    let rules: Vec<_> = ok.into_iter().map(Result::unwrap).collect();
    let results: Vec<_> = rules.iter()
        .map(|rule| rule.eval(user_input, &vec![]))
        .collect();
    println!("FINAL EVAL");
    println!("{:?}", results);

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn dry_run() {
        let spec = concat!(
            "parent:\n",
            "  type: object\n",
            "  fields:\n",
            "    hello:\n",
            "      type: string\n",
            "      eq: [parent, world]\n",
            "    world: string\n",
            "    nested:\n",
            "      type: object\n",
            "      fields:\n",
            "        foobar: string\n",
            "other: string\n"
        );

        let input = concat!(
            "parent:\n",
            "  hello: foo\n",
            "  world: bar\n",
            "  nested:\n",
            "    foobar: fizzbuzz\n",
            "other: something else\n",
        );

        let spec: Mapping = serde_yaml::from_str(spec).unwrap();
        let input: Value = serde_yaml::from_str(input).unwrap();

        let yaml_res: Vec<_> = parse_grammar(&spec).into_iter()
            .map(Result::unwrap)
            .collect();      
        
        let name = valstr!("");
        let rule = into_rule(yaml_res, &name);
        println!("EVALUATION:");
        evaluate(&Constraint::Obj(rule), &input);

    }
}
