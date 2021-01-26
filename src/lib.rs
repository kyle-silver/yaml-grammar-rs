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
            "other: 7\n",
        );

        let spec: Mapping = serde_yaml::from_str(spec).unwrap();
        let input: Value = serde_yaml::from_str(input).unwrap();

        let yaml_res: Vec<_> = parse_grammar(&spec).into_iter()
            .map(Result::unwrap)
            .collect();      
        
        let name = valstr!(".");
        let obj_constr = into_constraint(yaml_res, &name);
        let obj_constr = Constraint::Obj(obj_constr);
        let rules = rules(&obj_constr, &input);
        let (ok, _): (Vec<_>, _) = rules.into_iter().partition(|r| r.is_ok());
        let ok = ok.into_iter().map(Result::unwrap).collect();
        println!("EVALUATION:");
        let eval = evaluate(&ok, &input);
        println!("FINAL EVAL:\n{:?}", eval);

    }
}
