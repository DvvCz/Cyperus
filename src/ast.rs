struct ScriptInfo {}

#[non_exhaustive]
enum PapyrusAst {
	ScriptInfo(ScriptInfo),

	Statements(Vec<Statement>),
}

#[non_exhaustive]
enum Type {
	Bool,
	Float,
	Int,
	String,
	Var,

	ObjectReference
}

#[non_exhaustive]
enum Statement {
	/// Vector of conditions and statements.
	/// Condition is None in case of `else`.
	If(Vec<(Option<Expr>, Vec<Self>)>),
	While(Expr, Vec<Self>),

	Function(Type, String, Vec<(Type, String)>, Vec<Self>),
	Event(String, Vec<(Type, String)>, Vec<Self>),
}