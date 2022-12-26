use super::Rule;
use once_cell::sync::Lazy;
use pest::pratt_parser::PrattParser;

pub static PRATT_PARSER: Lazy<PrattParser<Rule>> = Lazy::new(|| {
	use pest::pratt_parser::{Assoc::*, Op};

	PrattParser::new()
		.op(Op::infix(Rule::op_eq, Left)
			| Op::infix(Rule::op_neq, Left)
			| Op::infix(Rule::op_geq, Left)
			| Op::infix(Rule::op_leq, Left)
			| Op::infix(Rule::op_gt, Left)
			| Op::infix(Rule::op_lt, Left))
		.op(Op::infix(Rule::op_add, Left) | Op::infix(Rule::op_sub, Left))
		.op(Op::infix(Rule::op_mul, Left) | Op::infix(Rule::op_div, Left))
		.op(Op::prefix(Rule::not) | Op::prefix(Rule::neg))
		.op(Op::postfix(Rule::cast) | Op::postfix(Rule::type_check))
});
