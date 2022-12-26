mod expression;
mod ast;
mod error;

pub use error::{Error, Result};
use pest::{Parser, iterators::{Pair, Pairs}};
use ast::{Type, Index, Statement, Expression, Parameter, ScriptInfo, Ast};

#[derive(Parser)]
#[grammar = "papyrus.pest"]
pub struct PestParser;

#[inline(always)]
fn next_inner<'a>(pairs: &'a mut Pairs<Rule>) -> Pairs<'a, Rule> {
	pairs.next().unwrap().into_inner()
}

trait PestWalker {
	fn expect_rule(&mut self, rule: Rule) -> Result<Pair<Rule>>;
	fn opt_rule(&mut self, rule: Rule) -> Option<Pair<Rule>>;
}

/// All of these functions assume they are on a node with the correct matching [Rule].
/// Should only use these after using [PestWalker::expect_rule].
trait PestNode<'a> {
	fn ident(self) -> String;
	fn ty(self) -> String;
	fn param(self) -> Result<Parameter>;

	fn statement(self) -> Result<Statement>;
	fn expression(self) -> Result<Expression>;

	fn body(self) -> Result<Vec<Statement>>;
}

impl<'a> PestNode<'a> for Pair<'a, Rule> {
	#[inline(always)]
	fn ident(self) -> String {
		self.as_str().to_owned()
	}

	#[inline(always)]
	fn ty(self) -> String {
		self.as_str().to_owned()
	}

	fn param(self) -> Result<Parameter> {
		let mut inner = self.into_inner();
		Ok(Parameter(
			inner.expect_rule(Rule::ident)?.ident(),
			inner.expect_rule(Rule::r#type)?.ty(),
			inner.opt_rule(Rule::expression).map(|e| e.expression().ok()).flatten(),
		))
	}

	fn statement(self) -> Result<Statement> {
		let (rule, mut inner) = (self.as_rule(), self.into_inner());
		let out = match rule {
			Rule::r#if => {
				let cond = inner.expect_rule(Rule::expression)?.expression()?;
				let body = inner.expect_rule(Rule::body)?.body()?;

				let mut elifs = vec![];
				for elif in inner.next().unwrap().into_inner() {
					let mut inner = elif.into_inner();
					elifs.push((inner.expect_rule(Rule::expression)?.expression()?, inner.expect_rule(Rule::body)?.body()?));
				}

				Statement::If {
					cond,
					body,

					elifs,
					else_block: inner.opt_rule(Rule::body).map(|x| x.body().ok()).flatten()
				}
			},

			Rule::full_property => Statement::PropertyFull {
				ty: inner.expect_rule(Rule::r#type)?.ty(),
				name: inner.expect_rule(Rule::ident)?.ident(),
				functions: (inner.expect_rule(Rule::expression)?.expression()?, inner.opt_rule(Rule::expression).map(|e| e.expression().ok()).flatten()),
			},

			Rule::auto_property => Statement::PropertyAuto {
				ty: inner.expect_rule(Rule::r#type)?.ty(),
				name: inner.expect_rule(Rule::ident)?.ident(),
				value: inner.opt_rule(Rule::expression).map(|e| e.expression().ok()).flatten(),
			},

			Rule::const_property => Statement::PropertyAutoConst {
				ty: inner.expect_rule(Rule::r#type)?.ty(),
				name: inner.expect_rule(Rule::ident)?.ident(),
				value: inner.expect_rule(Rule::expression)?.expression()?,
			},

			Rule::auto_state => Statement::State {
				auto: true,
				name: inner.expect_rule(Rule::ident)?.ident(),
				body: inner.expect_rule(Rule::body)?.body()?,
			},

			Rule::normal_state => Statement::State {
				auto: false,
				name: inner.expect_rule(Rule::ident)?.ident(),
				body: inner.expect_rule(Rule::body)?.body()?,
			},

			Rule::native_function => Statement::NativeFunction {
				return_type: inner.opt_rule(Rule::ident).map(PestNode::ident),
				name: inner.expect_rule(Rule::ident)?.ident(),
				parameters: next_inner(&mut inner).map(PestNode::param).collect::<Result<Vec<_>>>()?,
			},

			Rule::global_function => Statement::Function {
				return_type: inner.opt_rule(Rule::ident).map(PestNode::ident),
				name: inner.expect_rule(Rule::ident)?.ident(),
				parameters: next_inner(&mut inner).map(PestNode::param).collect::<Result<Vec<_>>>()?,
				body: inner.expect_rule(Rule::body)?.body()?,
			},

			Rule::method_function => Statement::Function {
				return_type: inner.opt_rule(Rule::ident).map(PestNode::ident),
				name: inner.expect_rule(Rule::ident)?.ident(),
				parameters: next_inner(&mut inner).map(PestNode::param).collect::<Result<Vec<_>>>()?,
				body: inner.expect_rule(Rule::body)?.body()?,
			},

			Rule::r#return => Statement::Return {
				value: inner.opt_rule(Rule::expression).map(|e| e.expression().ok()).flatten(),
			},

			Rule::definition => Statement::Definition {
				ty: inner.expect_rule(Rule::r#type)?.ty(),
				name: inner.expect_rule(Rule::ident)?.ident(),
				value: inner.expect_rule(Rule::expression)?.expression()?,
			},

			Rule::event => Statement::Event {
				name: inner.expect_rule(Rule::ident)?.ident(),
				parameters: next_inner(&mut inner).map(PestNode::param).collect::<Result<Vec<_>>>()?,
				body: inner.expect_rule(Rule::body)?.body()?,
			},

			Rule::r#while => Statement::While {
				cond: inner.expect_rule(Rule::expression)?.expression()?,
				body: inner.expect_rule(Rule::body)?.body()?,
			},

			Rule::assignment => {
				let name = inner.expect_rule(Rule::ident)?.ident();

				let mut indexes = vec![];
				while let Some(p) = inner.peek() {
					match p.as_rule() {
						Rule::bracket_index => indexes.push(Index::Bracket(p.into_inner().expect_rule(Rule::expression)?.expression()?)),
						Rule::dot_index => indexes.push(Index::Dot(p.into_inner().expect_rule(Rule::ident)?.ident())),
						_ => break
					}
					inner.next();
				}

				Statement::Assignment {
					name,
					indexes,
					value: inner.expect_rule(Rule::expression)?.expression()?
				}
			},

			Rule::group => Statement::Group {
				name: inner.expect_rule(Rule::ident)?.ident(),
				properties: inner.expect_rule(Rule::body)?.body()?,
			},

			_ => todo!("Unimplemented: {rule:?}")
		};

		Ok(out)
	}

	fn expression(self) -> Result<Expression> {
		// Todo: Make these functions fallible instead of panicking
		fn primary(prim: Pair<Rule>) -> Expression {
			match prim.as_rule() {
				Rule::hexadecimal => Expression::LiteralInteger(prim.as_str().parse().unwrap()),
				Rule::decimal => Expression::LiteralFloat(prim.as_str().parse().unwrap()),
				Rule::integer => Expression::LiteralInteger(prim.as_str().parse().unwrap()),
				Rule::ident => Expression::Ident(prim.as_str().to_owned()),
				Rule::string => Expression::LiteralString(prim.as_str().to_owned()),
				Rule::boolean => Expression::LiteralBool(prim.as_str().to_lowercase() == "true"),
				Rule::array => Expression::LiteralArray(prim.as_str().to_owned()),
				unknown => todo!("expr: {unknown:#?}"),
			}
		}

		fn infix(lhs: Expression, op: Pair<Rule>, rhs: Expression) -> Expression {
			match op.as_rule() {
				Rule::op_add => Expression::Addition(Box::new(lhs), Box::new(rhs)),
				Rule::op_sub => Expression::Subtraction(Box::new(lhs), Box::new(rhs)),
				Rule::op_mul => Expression::Multiplication(Box::new(lhs), Box::new(rhs)),
				Rule::op_div => Expression::Division(Box::new(lhs), Box::new(rhs)),
				Rule::op_eq => Expression::Equal(Box::new(lhs), Box::new(rhs)),
				Rule::op_neq => Expression::NotEqual(Box::new(lhs), Box::new(rhs)),
				rule => unreachable!("Expected infix operation, found {:?}", rule),
			}
		}

		fn prefix(op: Pair<Rule>, rhs: Expression) -> Expression {
			match op.as_rule() {
				Rule::neg => Expression::Negate(Box::new(rhs)),
				Rule::not => Expression::Not(Box::new(rhs)),
				rule => unreachable!("Expected prefix operation, found {:?}", rule)
			}
		}

		fn postfix(lhs: Expression, op: Pair<Rule>) -> Expression {
			match op.as_rule() {
				Rule::cast => Expression::Cast(Box::new(lhs), op.into_inner().as_str().to_owned()),
				Rule::type_check => Expression::Is(Box::new(lhs), op.into_inner().as_str().to_owned()),
				rule => unreachable!("Expected postfix operation, found {:?}", rule)
			}
		}

		let expr = expression::PRATT_PARSER
			.map_primary(primary)
			.map_infix(infix)
			.map_prefix(prefix)
			.map_postfix(postfix)
			.parse(self.into_inner());

		Ok(expr)
	}

	fn body(self) -> Result<Vec<Statement>> {
		self.into_inner().map(Self::statement).collect()
	}
}

impl<'a> PestWalker for Pairs<'a, Rule> {
	fn expect_rule(&mut self, expecting: Rule) -> Result<Pair<Rule>> {
		match self.peek() {
			Some(pair) => {
				let got = pair.as_rule();
				if got == expecting {
					self.next();
					Ok(pair)
				} else {
					Err(Error::Expected(expecting, got))
				}
			},
			None => Err(Error::Expected(expecting, Rule::EOI))
		}
	}

	fn opt_rule(&mut self, expecting: Rule) -> Option<Pair<Rule>> {
		match self.peek() {
			Some(pair) if pair.as_rule() == expecting => {
				self.next();
				Some(pair)
			}
			_ => None
		}
	}
}

pub fn parse_module(source: impl AsRef<str>) -> Result<Ast> {
	let source = source.as_ref();
	let pairs = PestParser::parse(Rule::module, source)?;

	let mut statements = vec![];
	let mut script_info = ScriptInfo::default();

	for item in pairs.into_iter() {
		match item.as_rule() {
			Rule::heading => {
				let item = item.into_inner().next().unwrap();
				match item.as_rule() {
					Rule::script_name => {
						let mut inner = item.into_inner();
						script_info.script_name = inner.expect_rule(Rule::ident)?.ident();
						script_info.extended_type = inner.opt_rule(Rule::r#type).map(PestNode::ty);
						script_info.is_conditional = inner.peek().is_some();
					}
					_ => unreachable!(),
				}
			}

			Rule::body => {
				for pair in item.into_inner() {
					statements.push(pair.statement()?);
				}
			}

			Rule::EOI => (),

			unknown => todo!("parse_module {unknown:#?} {}", item.as_str()),
		}
	}

	Ok(Ast {
		script_info,
		statements,
	})
}