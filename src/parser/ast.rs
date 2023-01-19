use std::borrow::Cow;

#[derive(Debug, Default)]
pub struct ScriptInfo {
	pub script_name: String,

	pub extended_type: Option<Type>,
	pub is_conditional: bool,
}

#[non_exhaustive]
#[derive(Debug)]
pub struct Ast {
	pub script_info: ScriptInfo,
	pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
	Integer,
	Float,
	String,
	Boolean,
	None,

	Array(String), // You can't have an array of arrays: https://www.creationkit.com/index.php?title=Arrays_(Papyrus)#Declaring_Arrays
	Class(String),
}

impl Type {
	#[inline]
	pub fn frag(&self) -> &str {
		match self {
			Self::Integer => "integer",
			Self::Float => "float",
			Self::String => "string",
			Self::Boolean => "bool",
			Self::None => "none",

			Self::Array(t) => t,
			Self::Class(s) => s,
		}
	}
}

impl std::fmt::Display for Type {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Array(ty) => {
				ty.fmt(f);
				f.write_str("[]");
			}

			_ => f.write_str(self.frag())?,
		}
		Ok(())
	}
}

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
		functions: (Box<Self>, Option<Box<Self>>),
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

	CompoundAssignment {
		name: String,
		op: super::Rule,
		value: Expression,
	},

	/// Certain expressions (function calls) can be used as statements.
	Expression {
		expr: Expression,
	},

	Struct {
		name: String,
		fields: Vec<Field>,
	},

	/// Import ObjectReference
	Import {
		item: String,
	},
}

#[derive(Debug)]
pub enum Index {
	Dot(String),
	Bracket(Expression),
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum Expression {
	/// +
	Addition(Box<Self>, Box<Self>),

	/// -
	Subtraction(Box<Self>, Box<Self>),

	/// *
	Multiplication(Box<Self>, Box<Self>),

	/// /
	Division(Box<Self>, Box<Self>),

	/// >
	GreaterThan(Box<Self>, Box<Self>),

	/// <
	LessThan(Box<Self>, Box<Self>),

	/// >=
	GreaterThanOrEqual(Box<Self>, Box<Self>),

	/// <=
	LessThanOrEqual(Box<Self>, Box<Self>),

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

	/// None
	None,

	/// new int[5]
	Array(Type, Box<Self>),

	/// new test
	Struct(Type),
}

#[derive(Debug)]
pub struct Parameter(pub Type, pub String, pub Option<Expression>);

#[derive(Debug, Clone)]
pub enum Argument {
	Named(String, Expression),
	Anonymous(Expression),
}

#[derive(Debug)]
pub struct Field(pub Type, pub String, pub Option<Expression>);
