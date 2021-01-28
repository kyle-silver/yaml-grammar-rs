use constraint::Constraint;
use parse::{ParseErr, YamlParseResult};
use obj::{ObjConstr, ObjectConstraint};
use rule::{Rule, RuleEvalErr, RuleEvalSuccess};
use serde_yaml::{Mapping, Value};
use value_ref::ValueResolutionErr;

// public API
pub mod parse;
pub mod num;
pub mod str;
pub mod obj;
pub mod rule;
pub mod value_ref;
pub mod bubble;
pub mod constraint;

#[derive(Debug)]
pub enum Evaluation<'a> {
    GrammarParseErr(Vec<ParseErr<'a>>),
    ValueResolutionErr(Vec<ValueResolutionErr<'a>>),
    RuleEvalErr(Vec<RuleEvalErr<'a>>),
    Completed { ok: Vec<RuleEvalSuccess<'a>>, err: Vec<RuleEvalErr<'a>>},
}

pub fn yamlfmt<'a>(spec: &'a Mapping, input: &'a Value, name: &'a Value) -> Evaluation<'a> {
    // first, parse the spec
    let spec: Vec<_> = spec.iter().map(Constraint::from_spec).collect();
    let yaml_parse: YamlParseResult = spec.into();

    // if there are errors, return them
    let (constraints, err): (Vec<_>, _) = yaml_parse.into_iter().partition(Result::is_ok);
    if !err.is_empty() {
        return Evaluation::GrammarParseErr(err.into_iter().map(Result::unwrap_err).collect());
    }

    // if there are no errors, try value resolution
    let constraints: Vec<_> = constraints.into_iter().map(Result::unwrap).collect();
    let map = constraints.into_iter().map(|c| (c.field_name(), c)).collect();
    let objconstr = ObjectConstraint::new(name, ObjConstr::Fields(map), None);
    let constraint = Constraint::Obj(objconstr);
    let (rules, err): (Vec<_>, _) = Rule::new(constraint, &input).get().into_iter()
        .partition(Result::is_ok);
    if !err.is_empty() {
        return Evaluation::ValueResolutionErr(err.into_iter().map(Result::unwrap_err).collect());
    }

    // if all the rules are valid, evaluate them
    let rules: Vec<_> = rules.into_iter().map(Result::unwrap).collect();
    let (ok, err): (Vec<_>, _) = rules.into_iter()
        .map(|rule| rule.eval(input, &vec![]))
        .flatten()
        .partition(Result::is_ok);
    let ok = ok.into_iter().map(Result::unwrap).collect();
    let err = err.into_iter().map(Result::unwrap_err).collect();
    Evaluation::Completed { ok, err }

}
