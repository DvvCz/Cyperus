use crate::parse::ast::Index;

use super::{
	ast::Parameter, expression::ParseExpression, PestNode, PestWalker, Result, Rule, Statement,
};
use pest::iterators::Pair;

pub(crate) trait ParseStatement: ParseExpression {
	fn statement(self) -> Result<Statement>;
	fn body(self) -> Result<Vec<Statement>>;

	fn param(self) -> Result<Parameter>;
	fn params(self) -> Result<Vec<Parameter>>;
}

impl<'a> ParseStatement for Pair<'a, Rule> {
	fn body(self) -> Result<Vec<Statement>> {
		self.into_inner().map(Self::statement).collect()
	}

	fn param(self) -> Result<Parameter> {
		let mut inner = self.into_inner();
		Ok(Parameter(
			inner.expect_rule(Rule::r#type)?.ty(),
			inner.expect_rule(Rule::ident)?.ident(),
			inner
				.opt_rule(Rule::expression)
				.and_then(|e| e.expression().ok()),
		))
	}

	fn params(self) -> Result<Vec<Parameter>> {
		self.into_inner()
			.map(Self::param)
			.collect::<Result<Vec<_>>>()
	}

	fn statement(self) -> Result<Statement> {
		let inner = self.into_inner().next().unwrap();
		let (rule, mut inner) = (inner.as_rule(), inner.into_inner());

		let out = match rule {
			Rule::r#if => {
				let cond = inner.expect_rule(Rule::expression)?.expression()?;
				let body = inner.opt_rule(Rule::body).and_then(|b| b.body().ok()).unwrap_or(vec![]);

				let mut elifs = vec![];
				while let Some(elif) = inner.peek() {
					if elif.as_rule() == Rule::r#elseif {
						inner.next();
						let mut inner = elif.into_inner();
						elifs.push((
							inner.expect_rule(Rule::expression)?.expression()?,
							inner.expect_rule(Rule::body)?.body()?,
						));
					} else {
						break;
					}
				}

				Statement::If {
					cond,
					body,

					elifs,
					else_block: inner.opt_rule(Rule::body).and_then(|x| x.body().ok()),
				}
			},

			Rule::r#while => Statement::While {
				cond: inner.expect_rule(Rule::expression)?.expression()?,
				body: inner.expect_rule(Rule::body)?.body()?,
			},

			Rule::full_property => Statement::PropertyFull {
				ty: inner.expect_rule(Rule::r#type)?.ty(),
				name: inner.expect_rule(Rule::ident)?.ident(),
				functions: (
					inner.expect_rule(Rule::expression)?.expression()?,
					inner
						.opt_rule(Rule::expression)
						.and_then(|e| e.expression().ok()),
				),
			},

			Rule::auto_property => Statement::PropertyAuto {
				ty: inner.expect_rule(Rule::r#type)?.ty(),
				name: inner.expect_rule(Rule::ident)?.ident(),
				value: inner
					.opt_rule(Rule::expression)
					.and_then(|e| e.expression().ok()),
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
				return_type: inner.opt_rule(Rule::r#type).map(PestNode::ty),
				name: inner.expect_rule(Rule::ident)?.ident(),
				parameters: inner.expect_rule(Rule::parameters)?.params()?,
			},

			Rule::global_function => Statement::Function {
				return_type: inner.opt_rule(Rule::r#type).map(PestNode::ty),
				name: inner.expect_rule(Rule::ident)?.ident(),
				parameters: inner.expect_rule(Rule::parameters)?.params()?,
				body: inner.expect_rule(Rule::body)?.body()?,
			},

			Rule::method_function => Statement::Function {
				return_type: inner.opt_rule(Rule::r#type).map(PestNode::ty),
				name: inner.expect_rule(Rule::ident)?.ident(),
				parameters: inner.expect_rule(Rule::parameters)?.params()?,
				body: inner.expect_rule(Rule::body)?.body()?,
			},

			Rule::r#return => Statement::Return {
				value: inner
					.opt_rule(Rule::expression)
					.and_then(|e| e.expression().ok()),
			},

			Rule::definition => Statement::Definition {
				ty: inner.expect_rule(Rule::r#type)?.ty(),
				name: inner.expect_rule(Rule::ident)?.ident(),
				value: inner.expect_rule(Rule::expression)?.expression()?,
			},

			Rule::event => Statement::Event {
				name: inner.expect_rule(Rule::ident)?.ident(),
				parameters: inner.expect_rule(Rule::parameters)?.params()?,
				body: inner.expect_rule(Rule::body)?.body()?,
			},

			Rule::assignment => {
				let name = inner.expect_rule(Rule::ident)?.ident();

				let mut indexes = vec![];
				while let Some(p) = inner.peek() {
					match p.as_rule() {
						Rule::bracket_index => indexes.push(Index::Bracket(
							p.into_inner().expect_rule(Rule::expression)?.expression()?,
						)),
						Rule::dot_index => indexes
							.push(Index::Dot(p.into_inner().expect_rule(Rule::ident)?.ident())),
						_ => break,
					}
					inner.next();
				}

				Statement::Assignment {
					name,
					indexes,
					value: inner.expect_rule(Rule::expression)?.expression()?,
				}
			}

			Rule::group => Statement::Group {
				name: inner.expect_rule(Rule::ident)?.ident(),
				properties: inner.expect_rule(Rule::body)?.body()?,
			},

			Rule::expression => Statement::Expression {
				expr: inner.expect_rule(Rule::expression)?.expression()?,
			},

			Rule::declaration => Statement::Declaration {
				ty: inner.expect_rule(Rule::r#type)?.ty(),
				name: inner.expect_rule(Rule::ident)?.ident(),
			},

			_ => todo!("{rule:?}"),
		};

		Ok(out)
	}
}
