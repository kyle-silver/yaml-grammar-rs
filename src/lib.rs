use std::clone;

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
    Success(Vec<RuleEvalSuccess<'a>>),
    Failure(Vec<RuleEvalErr<'a>>),
    MixedSuccess { ok: Vec<RuleEvalSuccess<'a>>, err: Vec<RuleEvalErr<'a>> },
}

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

pub fn evaluate<'a>(spec: &'a Constraint, user_input: &'a Value) -> Evaluation<'a> {
    let (ok, err): (Vec<_>, Vec<_>) = Rule::new(spec, user_input).get()
        .into_iter()
        .partition(Result::is_ok);
    if !err.is_empty() {
        return Evaluation::ValueResolutionErr(err.into_iter().map(Result::unwrap_err).collect());
    }
    let rules: Vec<_> = ok.into_iter().map(Result::unwrap).collect();
    let (ok, err): (Vec<_>, Vec<_>) = rules.iter()
        .map(|rule| rule.eval(user_input, &vec![]))
        .flatten()
        .partition(Result::is_ok);
    let ok: Vec<_> = ok.into_iter().map(Result::unwrap).collect();
    let err: Vec<_> = err.into_iter().map(Result::unwrap_err).collect();

    if err.is_empty() {
        Evaluation::Success(ok)
    } else if ok.is_empty() {
        Evaluation::Failure(err)
    } else {
        Evaluation::MixedSuccess { ok, err }
    }
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
