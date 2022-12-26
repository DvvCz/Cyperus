use super::Rule;
use once_cell::sync::Lazy;
use pest::pratt_parser::PrattParser;

pub static PRATT_PARSER: Lazy<PrattParser<Rule>> = Lazy::new(|| {
	use pest::pratt_parser::{Assoc::*, Op};

	macro_rules! binary {
		($rule:ident) => { Op::infix(Rule::$rule, Left) };
		($rule:ident, $( $r:ident ),*) => { binary!($rule) | binary!( $( $r ),* ) };
	}

	macro_rules! unary {
		($rule:ident) => { Op::prefix(Rule::$rule) };
		($rule:ident, $( $r:ident ),*) => { unary!($rule) | unary!( $( $r ),* ) };
	}

	macro_rules! postfix {
		($rule:ident) => { Op::postfix(Rule::$rule) };
		($rule:ident, $( $r:ident ),*) => { postfix!($rule) | postfix!( $( $r ),* ) };
	}

	PrattParser::new()
		.op(binary!(op_eq, op_neq, op_geq, op_leq, op_gt, op_lt))
		.op(binary!(op_add, op_sub))
		.op(binary!(op_mul, op_div))
		.op(unary!(not, neg))
		.op(postfix!(cast, type_check, call, dot_index, bracket_index))
});
