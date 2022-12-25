use crate::{ast, error::Result};
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

/*#[non_exhaustive]
#[derive(Debug)]
pub enum Type {
	Bool,
	Float,
	Int,
	String,
	Var,

	ObjectReference,

	Other(String)
}*/

pub type Type = String;

#[non_exhaustive]
#[derive(Debug)]
pub enum Statement {
	/// Vector of conditions and statements.
	/// Condition is None in case of `else`.
	If(Vec<(Option<Expression>, Vec<Self>)>),
	While {
		cond: Expression,
		body: Vec<Self>,
	},

	Function {
		return_type: Type,
		name: String,
		parameters: Vec<(Type, String)>,
		body: Vec<Self>,
	},
	Event {
		name: String,
		parameters: Vec<(Type, String)>,
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

	/// Postfix Operations
	Cast,
	Call,
	Index
}

#[non_exhaustive]
#[derive(Debug)]
pub enum Expression {
	BinaryOperation(Box<Self>, Op, Box<Self>),
	UnaryOperation(Op, Box<Self>),

	Ident(String),
	LiteralBool(bool),
	LiteralString(String),
	LiteralInteger(i64),
	LiteralFloat(f64),
}

pub fn parse_expr(pairs: &mut Pairs<Rule>) -> Expression {
	use once_cell::sync::Lazy;
	static PrattParser: Lazy<PrattParser<Rule>> = Lazy::new(|| {
		use pest::pratt_parser::{Assoc::*, Op};

		PrattParser::new()
			.op(Op::infix(Rule::eq, Left)
				| Op::infix(Rule::neq, Left)
				| Op::infix(Rule::geq, Left)
				| Op::infix(Rule::leq, Left)
				| Op::infix(Rule::gt, Left)
				| Op::infix(Rule::lt, Left))
			.op(Op::infix(Rule::add, Left) | Op::infix(Rule::sub, Left))
			.op(Op::infix(Rule::mul, Left) | Op::infix(Rule::div, Left))
			.op(Op::prefix(Rule::not) | Op::prefix(Rule::neg))
	});

	PrattParser
		.map_primary(|prim| match prim.as_rule() {
			Rule::float => Expression::LiteralFloat(prim.as_str().parse().unwrap()),
			Rule::integer => Expression::LiteralInteger(prim.as_str().parse().unwrap()),
			Rule::ident => Expression::Ident(prim.as_str().to_owned()),
			unknown => todo!("expr: {unknown:#?}"),
		})
		.map_infix(|lhs, op, rhs| {
			let op = match op.as_rule() {
				Rule::add => Op::Add,
				Rule::sub => Op::Sub,
				Rule::mul => Op::Mul,
				Rule::div => Op::Div,
				Rule::eq => Op::Eq,
				Rule::neq => Op::Neq,
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
		.parse(pairs)
}

fn parse_body(pairs: &mut Pairs<Rule>) -> Result<Vec<Statement>> {
	pairs.next().unwrap().into_inner().map(parse_stmt).collect()
}

pub fn parse_stmt(stmt: Pair<Rule>) -> Result<Statement> {
	let (rule, mut item) = (stmt.as_rule(), stmt.into_inner());

	println!("parsing stmt");

	let stmt = match rule {
		Rule::r#if => {
			let cond = parse_expr(&mut item.next().unwrap().into_inner());
			println!("if {:#?}", cond);

			if let Some(stmts) = item.next() {
				let stmts = stmts.into_inner();
				println!("stmts: {:#?}", stmts.as_str());
			}

			todo!()
		}

		Rule::full_property => {
			let type_ = item.next().unwrap().as_str().to_owned();
			let name = item.next().unwrap().as_str().to_owned();

			let expr1 = item.next().map(|x| parse_expr(&mut item));
			let expr2 = item.next().map(|x| parse_expr(&mut item));

			Statement::PropertyFull {
				ty: type_,
				name,
				setter: expr1,
				getter: expr2,
			}
		}

		Rule::auto_property => {
			let type_ = item.next().unwrap().as_str().to_owned();
			let name = item.next().unwrap().as_str().to_owned();

			let expr = item.next().map(|x| parse_expr(&mut item));
			Statement::PropertyAuto {
				ty: type_,
				name,
				value: expr,
			}
		}

		Rule::auto_conditional_property => {
			let type_ = item.next().unwrap().as_str().to_owned();
			let name = item.next().unwrap().as_str().to_owned();

			match type_.to_ascii_lowercase().as_str() {
				"int" | "float" | "bool" => (),
				_ => panic!("Bad type for auto conditional property."),
			}

			Statement::PropertyAutoConditional { ty: type_, name }
		}

		Rule::const_property => {
			let type_ = item.next().unwrap().as_str().to_owned();
			let name = item.next().unwrap().as_str().to_owned();

			let expr = parse_expr(&mut item.next().unwrap().into_inner());
			Statement::PropertyAutoConst {
				ty: type_,
				name,
				value: expr,
			}
		}

		Rule::auto_state => {
			let name = item.next().unwrap().as_str().to_owned();
			let body = parse_body(&mut item)?;

			Statement::State {
				auto: true,
				name,
				body
			}
		}

		Rule::normal_state => {
			let name = item.next().unwrap().as_str().to_owned();

			let body = parse_body(&mut item)?;

			Statement::State {
				auto: false,
				name,
				body
			}
		},

		Rule::function => todo!("function"),
		Rule::r#return => todo!("return"),
		Rule::definition => todo!("definition"),
		Rule::event => {
			let name = item.next().unwrap().as_str().to_owned();

			fn parse_params(pairs: &mut Pairs<Rule>) -> Result<Vec<(Type, String)>> {
				let params = pairs
					.next()
					.unwrap()
					.into_inner()
					.map(|param| {
						let mut data = param.into_inner();
						( data.next().unwrap().as_str().to_owned(), data.next().unwrap().as_str().to_owned()  )
					})
					.collect::<Vec<_>>();

				Ok(params)
			}

			let parameters = parse_params(&mut item)?;
			let body = parse_body(&mut item)?;

			Statement::Event { name, parameters, body }
		},
		Rule::r#while => todo!("while_"),
		Rule::assignment => todo!("assignment"),
		Rule::compound_assignment => todo!("compound_assignment"),
		Rule::declaration => todo!("declaration"),

		unknown => unreachable!("{unknown:#?}"),
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
						script_info.script_name = pairs.next().unwrap().as_str().to_owned();
						script_info.extended_type = pairs.next().unwrap().as_str().to_owned();
						script_info.is_conditional = pairs.next().is_some();
					}
					_ => unreachable!(),
				}
			}

			Rule::body => {
				statements.extend( parse_body(&mut pairs)?);
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
