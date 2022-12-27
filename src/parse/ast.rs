#[derive(Debug, Default)]
pub struct ScriptInfo {
	pub script_name: String,

	pub extended_type: Option<String>,
	pub is_conditional: bool,
}

#[non_exhaustive]
#[derive(Debug)]
pub struct Ast {
	pub script_info: ScriptInfo,
	pub statements: Vec<Statement>,
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
		functions: (Expression, Option<Expression>),
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

	Declaration {
		ty: Type,
		name: String,
	},

	Group {
		name: String,
		properties: Vec<Self>,
	},

	Assignment {
		name: String,
		indexes: Vec<Index>,
		value: Expression,
	},

	/// Certain expressions (function calls) can be used as statements.
	Expression {
		expr: Expression,
	},
}

#[derive(Debug)]
pub enum Index {
	Dot(String),
	Bracket(Expression),
}

#[non_exhaustive]
#[derive(Debug)]
pub enum Expression {
	/// +
	Addition(Box<Self>, Box<Self>),

	/// -
	Subtraction(Box<Self>, Box<Self>),

	/// *
	Multiplication(Box<Self>, Box<Self>),

	/// /
	Division(Box<Self>, Box<Self>),

	/// ==
	Equal(Box<Self>, Box<Self>),

	/// !=
	NotEqual(Box<Self>, Box<Self>),

	/// &&
	And(Box<Self>, Box<Self>),

	/// ||
	Or(Box<Self>, Box<Self>),

	/// !
	Not(Box<Self>),

	/// -
	Negate(Box<Self>),

	/// 2414 as int
	Cast(Box<Self>, Type),

	/// Foo is int
	Is(Box<Self>, Type),

	/// foo.bar
	DotIndex(Box<Self>, String),

	/// foo[0]
	BracketIndex(Box<Self>, Box<Self>),

	/// foo(bar.qux, "baz", 123)
	Call(Box<Self>, Vec<Argument>),

	/// Hello
	Ident(String),

	/// True or false
	Bool(bool),

	/// "String"
	String(String),

	/// 2
	Integer(i64),

	/// 0.4f or 0.2
	Float(f64),

	/// new int[5]
	Array(Type),
}

#[derive(Debug)]
pub struct Parameter(pub Type, pub String, pub Option<Expression>);

#[derive(Debug)]
pub enum Argument {
	Named(String, Expression),
	Anonymous(Expression)
}