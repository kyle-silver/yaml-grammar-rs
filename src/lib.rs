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

pub fn parse_grammar(spec: &Mapping) -> YamlParseResult {
    let res: Vec<_> = spec.iter().map(Constraint::from_spec).collect();
    res.into()
}

pub fn into_constraint<'a>(constraints: Vec<Constraint<'a>>, name: &'a Value) -> ObjectConstraint<'a> {
    let map = constraints.into_iter().map(|c| (c.field_name(), c)).collect();
    ObjectConstraint::new(name, ObjConstr::Fields(map), None)
}

pub fn rules<'a>(spec: &'a Constraint, user_input: &'a Value) -> Vec<Result<Rule<'a>, ValueResolutionErr<'a>>> {
    Rule::new(spec, user_input).get().into_iter().collect()
}

pub fn evaluate<'a>(rules: &'a Vec<Rule<'a>>, user_input: &'a Value) -> Evaluation<'a> {
    let (ok, err): (Vec<_>, Vec<_>) = rules.iter()
        .map(|rule| rule.eval(user_input, &vec![]))
        .flatten()
        .partition(Result::is_ok);
    let ok: Vec<_> = ok.into_iter().map(Result::unwrap).collect();
    let err: Vec<_> = err.into_iter().map(Result::unwrap_err).collect();
    Evaluation::Completed { ok, err }
}
