use super::ast::Argument;
use super::{Error, Expression, PestNode, PestWalker, Result, Rule};

use once_cell::sync::Lazy;
use pest::iterators::Pair;
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
		.op(binary!(op_and, op_or))
		.op(binary!(op_eq, op_neq, op_geq, op_leq, op_gt, op_lt))
		.op(binary!(op_add, op_sub))
		.op(binary!(op_mul, op_div))
		.op(unary!(not, neg))
		.op(postfix!(cast, type_check, call, dot_index, bracket_index))
});

pub(crate) trait ParseExpression: PestNode {
	fn argument(self) -> Result<Argument>;
	fn arguments(self) -> Result<Vec<Argument>>;
	fn expression(self) -> Result<Expression>;
}

impl<'a> ParseExpression for Pair<'a, Rule> {
	fn arguments(self) -> Result<Vec<Argument>> {
		self.into_inner()
			.map(Self::argument)
			.collect::<Result<Vec<_>>>()
	}

	fn argument(self) -> Result<Argument> {
		// Parses a single argument passed to a function.
		let mut inner = self.into_inner();
		match inner.next() {
			Some(pair) if pair.as_rule() == Rule::ident => Ok(Argument::Named(
				pair.ident(),
				inner.expect_rule(Rule::expression)?.expression()?,
			)),
			Some(pair) => Ok(Argument::Anonymous(pair.expression()?)),
			None => Err(Error::UnexpectedEOI(Rule::expression)),
		}
	}

	fn expression(self) -> Result<Expression> {
		// Todo: Make these functions fallible instead of panicking
		fn primary(prim: Pair<Rule>) -> Expression {
			match prim.as_rule() {
				Rule::ident => Expression::Ident(prim.ident()),
				Rule::hexadecimal => Expression::Integer(
					i64::from_str_radix(prim.as_str().trim_start_matches("0x"), 16).unwrap(),
				),
				Rule::decimal => Expression::Float(prim.as_str().parse().unwrap()),
				Rule::integer => Expression::Integer(prim.as_str().parse().unwrap()),
				Rule::string => Expression::String(prim.as_str().to_owned()),
				Rule::boolean => Expression::Bool(prim.as_str().to_lowercase() == "true"),
				Rule::new_array => {
					let mut inner = prim.into_inner();
					Expression::Array(
						inner.expect_rule(Rule::r#type).unwrap().ty(),
						Box::new(
							inner
								.expect_rule(Rule::expression)
								.unwrap()
								.expression()
								.unwrap(),
						),
					)
				}
				Rule::new_struct => Expression::Struct(prim.into_inner().next().unwrap().ty()),
				Rule::none => Expression::None,
				Rule::expression => prim.expression().unwrap(), // for grouped expressions: "(" ~ expression ~ ")"
				unknown => todo!("expr: {unknown:#?} at {:?}", prim.as_str()),
			}
		}

		fn infix(lhs: Expression, op: Pair<Rule>, rhs: Expression) -> Expression {
			match op.as_rule() {
				Rule::op_add => Expression::Addition(Box::new(lhs), Box::new(rhs)),
				Rule::op_sub => Expression::Subtraction(Box::new(lhs), Box::new(rhs)),
				Rule::op_mul => Expression::Multiplication(Box::new(lhs), Box::new(rhs)),
				Rule::op_div => Expression::Division(Box::new(lhs), Box::new(rhs)),

				Rule::op_gt => Expression::GreaterThan(Box::new(lhs), Box::new(rhs)),
				Rule::op_lt => Expression::LessThan(Box::new(lhs), Box::new(rhs)),
				Rule::op_geq => Expression::GreaterThanOrEqual(Box::new(lhs), Box::new(rhs)),
				Rule::op_leq => Expression::LessThanOrEqual(Box::new(lhs), Box::new(rhs)),
				Rule::op_eq => Expression::Equal(Box::new(lhs), Box::new(rhs)),
				Rule::op_neq => Expression::NotEqual(Box::new(lhs), Box::new(rhs)),

				Rule::op_and => Expression::And(Box::new(lhs), Box::new(rhs)),
				Rule::op_or => Expression::Or(Box::new(lhs), Box::new(rhs)),
				rule => unreachable!("Expected infix operation, found {:?}", rule),
			}
		}

		fn prefix(op: Pair<Rule>, rhs: Expression) -> Expression {
			match op.as_rule() {
				Rule::neg => Expression::Negate(Box::new(rhs)),
				Rule::not => Expression::Not(Box::new(rhs)),
				rule => unreachable!("Expected prefix operation, found {:?}", rule),
			}
		}

		fn postfix(lhs: Expression, op: Pair<Rule>) -> Expression {
			match op.as_rule() {
				Rule::cast => Expression::Cast(Box::new(lhs), op.into_inner().next().unwrap().ty()),
				Rule::type_check => {
					Expression::Is(Box::new(lhs), op.into_inner().next().unwrap().ty())
				}
				Rule::call => Expression::Call(
					Box::new(lhs),
					op.into_inner().next().unwrap().arguments().unwrap(),
				),
				Rule::dot_index => {
					Expression::DotIndex(Box::new(lhs), op.into_inner().next().unwrap().ident())
				}
				Rule::bracket_index => Expression::BracketIndex(
					Box::new(lhs),
					Box::new(op.into_inner().next().unwrap().expression().unwrap()),
				),
				rule => unreachable!("Expected postfix operation, found {:?}", rule),
			}
		}

		let expr = PRATT_PARSER
			.map_primary(primary)
			.map_infix(infix)
			.map_prefix(prefix)
			.map_postfix(postfix)
			.parse(self.into_inner());

		Ok(expr)
	}
}
