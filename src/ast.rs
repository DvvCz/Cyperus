use crate::{
	ast,
	error::{Error, Result},
};
use pest::{
	iterators::{Pair, Pairs},
	pratt_parser::PrattParser,
	Parser,
};

#[derive(Parser)]
#[grammar = "papyrus.pest"]
pub struct PapyrusParser;

#[derive(Debug, Default)]
struct ScriptInfo {
	script_name: String,

	extended_type: String,
	is_conditional: bool,
}

#[non_exhaustive]
#[derive(Debug)]
pub struct PapyrusAst {
	script_info: ScriptInfo,
	statements: Vec<Statement>,
}

pub type Type = String;

#[non_exhaustive]
#[derive(Debug)]
pub enum Statement {
	/// Vector of conditions and statements.
	/// Condition is None in case of `else`.
	If {
		cond: Expression,
		body: Vec<Self>,

		elifs: Vec<(Expression, Vec<Self>)>,

		else_block: Option<Vec<Self>>,
	},

	While {
		cond: Expression,
		body: Vec<Self>,
	},

	Function {
		return_type: Option<Type>,
		name: String,
		parameters: Vec<Parameter>,
		body: Vec<Self>,
	},

	NativeFunction {
		return_type: Option<Type>,
		name: String,
		parameters: Vec<Parameter>,
	},

	Return {
		value: Option<Expression>,
	},

	Event {
		name: String,
		parameters: Vec<Parameter>,
		body: Vec<Self>,
	},

	PropertyFull {
		ty: Type,
		name: String,
		setter: Option<Expression>,
		getter: Option<Expression>,
	},
	PropertyAuto {
		ty: Type,
		name: String,
		value: Option<Expression>,
	},
	PropertyAutoConst {
		ty: Type,
		name: String,
		value: Expression,
	},
	PropertyAutoConditional {
		ty: Type,
		name: String,
	},

	State {
		auto: bool,
		name: String,
		body: Vec<Self>,
	},

	Definition {
		ty: Type,
		name: String,
		value: Expression,
	},
}

#[non_exhaustive]
#[derive(Debug)]
pub enum Op {
	/// +
	Add,
	/// -
	Sub,
	/// *
	Mul,
	/// /
	Div,

	/// ==
	Eq,

	/// !=
	Neq,

	/// Unary Operations

	/// -
	Neg,
	/// !
	Not,
}

#[non_exhaustive]
#[derive(Debug)]
pub enum Expression {
	/// X + 4 * 2
	BinaryOperation(Box<Self>, Op, Box<Self>),

	/// !True
	UnaryOperation(Op, Box<Self>),

	/// 2414 as int
	Cast(Box<Self>, Type),

	/// Foo is int
	Is(Box<Self>, Type),

	/// Hello
	Ident(String),

	/// True or false
	LiteralBool(bool),

	/// "String"
	LiteralString(String),

	/// 2
	LiteralInteger(i64),

	/// 0.4f or 0.2
	LiteralFloat(f64),

	/// new int[5]
	LiteralArray(Type),
}

#[inline(always)]
fn next_inner<'a>(pairs: &'a mut Pairs<Rule>) -> Pairs<'a, Rule> {
	pairs.next().unwrap().into_inner()
}

trait PestHelper<'a> {
	fn parse_ident(&mut self) -> Result<String>;
	fn parse_optional_ident(&mut self) -> Option<String>;
	fn parse_type(&mut self) -> Result<Type>;

	fn parse_body(&mut self) -> Result<Vec<Statement>>;
	fn parse_params(&mut self) -> Result<Vec<Parameter>>;

	fn parse_optional_expr(&mut self) -> Option<Expression>;
	fn parse_expr(&mut self) -> Result<Expression>;
}

#[derive(Debug)]
pub struct Parameter(Type, String, Option<Expression>);

impl<'a> PestHelper<'a> for Pairs<'a, Rule> {
	fn parse_ident(&mut self) -> Result<String> {
		Ok(self
			.next()
			.ok_or(Error::Expected("identifier", "end of input"))?
			.as_str()
			.to_owned())
	}

	fn parse_optional_ident(&mut self) -> Option<String> {
		self.next().map(|i| i.as_str().to_owned())
	}

	fn parse_type(&mut self) -> Result<Type> {
		Ok(self
			.next()
			.ok_or(Error::Expected("type", "end of input"))?
			.as_str()
			.to_owned())
	}

	fn parse_body(&mut self) -> Result<Vec<Statement>> {
		next_inner(self).map(parse_stmt).collect()
	}

	fn parse_params(&mut self) -> Result<Vec<Parameter>> {
		fn parse_param(param: Pair<Rule>) -> Result<Parameter> {
			let mut inner = param.into_inner();
			Ok(Parameter(inner.parse_type()?, inner.parse_ident()?, inner.parse_optional_expr()))
		}

		next_inner(self).map(parse_param).collect()
	}

	fn parse_optional_expr(&mut self) -> Option<Expression> {
		if self.peek().is_some() {
			self.parse_expr().ok()
		} else {
			None
		}
	}

	fn parse_expr(&mut self) -> Result<Expression> {
		use once_cell::sync::Lazy;
		static PRATT_PARSER: Lazy<PrattParser<Rule>> = Lazy::new(|| {
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

		Ok(PRATT_PARSER
			.map_primary(|prim| match prim.as_rule() {
				Rule::hexadecimal => Expression::LiteralInteger(prim.as_str().parse().unwrap()),
				Rule::decimal => Expression::LiteralFloat(prim.as_str().parse().unwrap()),
				Rule::integer => Expression::LiteralInteger(prim.as_str().parse().unwrap()),
				Rule::ident => Expression::Ident(prim.as_str().to_owned()),
				Rule::string => Expression::LiteralString(prim.as_str().to_owned()),
				Rule::boolean => Expression::LiteralBool(prim.as_str().to_lowercase() == "true"),
				Rule::array => Expression::LiteralArray(prim.as_str().to_owned()),
				unknown => todo!("expr: {unknown:#?}"),
			})
			.map_infix(|lhs, op, rhs| {
				let op = match op.as_rule() {
					Rule::op_add => Op::Add,
					Rule::op_sub => Op::Sub,
					Rule::op_mul => Op::Mul,
					Rule::op_div => Op::Div,
					Rule::op_eq => Op::Eq,
					Rule::op_neq => Op::Neq,
					rule => unreachable!("parse_expr expected infix operation, found {:?}", rule),
				};

				Expression::BinaryOperation(Box::new(lhs), op, Box::new(rhs))
			})
			.map_prefix(|op, rhs| {
				let op = match op.as_rule() {
					Rule::neg => Op::Neg,
					Rule::not => Op::Not,
					rule => unreachable!("parse_expr expected unary operation, found {:?}", rule),
				};

				Expression::UnaryOperation(op, Box::new(rhs))
			})
			.map_postfix(|lhs, op| match op.as_rule() {
				Rule::cast => Expression::Cast(Box::new(lhs), op.into_inner().as_str().to_owned()),
				Rule::type_check => {
					Expression::Is(Box::new(lhs), op.into_inner().as_str().to_owned())
				}
				rule => unreachable!("parse_expr expected postfix operation, found {:?}", rule),
			})
			.parse(next_inner(self)))
	}
}

pub fn parse_stmt(stmt: Pair<Rule>) -> Result<Statement> {
	let (rule, mut item) = (stmt.as_rule(), stmt.into_inner());

	let stmt = match rule {
		Rule::r#if => {
			let cond = item.parse_expr()?;
			let body = item.parse_body()?;

			let mut elifs = vec![];
			for elif in item.next().unwrap().into_inner() {
				let mut inner = elif.into_inner();
				elifs.push((inner.parse_expr()?, inner.parse_body()?));
			}

			let else_block = item
				.next()
				.map(|x| x.into_inner().parse_body().ok())
				.flatten();

			Statement::If {
				cond,
				body,

				elifs,

				else_block,
			}
		}

		Rule::full_property => Statement::PropertyFull {
			ty: item.parse_type()?,
			name: item.parse_ident()?,
			setter: item.parse_optional_expr(),
			getter: item.parse_optional_expr(),
		},

		Rule::auto_property => Statement::PropertyAuto {
			ty: item.parse_type()?,
			name: item.parse_ident()?,
			value: item.parse_optional_expr(),
		},

		Rule::const_property => Statement::PropertyAutoConst {
			ty: item.parse_type()?,
			name: item.parse_ident()?,
			value: item.parse_expr()?,
		},

		Rule::auto_state => Statement::State {
			auto: true,
			name: item.parse_ident()?,
			body: item.parse_body()?,
		},

		Rule::normal_state => Statement::State {
			auto: false,
			name: item.parse_ident()?,
			body: item.parse_body()?,
		},

		Rule::native_function => Statement::NativeFunction {
			return_type: item.parse_optional_ident(),
			name: item.parse_ident()?,
			parameters: item.parse_params()?,
		},

		Rule::global_function => Statement::Function {
			return_type: item.parse_optional_ident(),
			name: item.parse_ident()?,
			parameters: item.parse_params()?,
			body: item.parse_body()?
		},

		Rule::method_function => Statement::Function {
			return_type: item.parse_optional_ident(),
			name: item.parse_ident()?,
			parameters: item.parse_params()?,
			body: item.parse_body()?
		},

		Rule::r#return => Statement::Return {
			value: item.parse_optional_expr()
		},

		Rule::definition => Statement::Definition {
			ty: item.parse_type()?,
			name: item.parse_ident()?,
			value: item.parse_expr()?,
		},

		Rule::event => Statement::Event {
			name: item.parse_ident()?,
			parameters: item.parse_params()?,
			body: item.parse_body()?,
		},

		Rule::r#while => todo!("while_"),
		Rule::assignment => todo!("assignment"),
		Rule::compound_assignment => todo!("compound_assignment"),
		Rule::declaration => todo!("declaration"),

		unknown => todo!("Rule::{unknown:?}"),
	};

	Ok(stmt)
}

pub fn parse_module(source: impl AsRef<str>) -> Result<PapyrusAst> {
	let source = source.as_ref();
	let pairs = ast::PapyrusParser::parse(ast::Rule::module, source)?;

	let mut statements = vec![];
	let mut script_info = ScriptInfo::default();

	for item in pairs.into_iter() {
		match item.as_rule() {
			Rule::heading => {
				let item = item.into_inner().next().unwrap();
				match item.as_rule() {
					Rule::script_name => {
						let mut pairs = item.into_inner();
						script_info.script_name = pairs.parse_ident()?;
						script_info.extended_type = pairs.parse_type()?;
						script_info.is_conditional = pairs.next().is_some();
					}
					_ => unreachable!(),
				}
			}

			Rule::body => {
				for pair in item.into_inner() {
					statements.push(parse_stmt(pair)?);
				}
			}

			Rule::EOI => (),

			unknown => todo!("parse_module {unknown:#?} {}", item.as_str()),
		}
	}

	Ok(PapyrusAst {
		script_info,
		statements,
	})
}
