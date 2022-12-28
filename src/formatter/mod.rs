// Formatter for papyrus scripts.
use crate::parser::ast::*;

pub trait Format {
	fn format(self) -> String;
}

impl Format for Expression {
	fn format(self) -> String {
		match self {
			Self::Struct(ty) => format!("new {ty}"),
			Self::Array(ty, expr) => format!("new {ty}[{}]", expr.format()),
			Self::Bool(val) => val.to_string(),
			Self::Integer(i) => i.to_string(),
			Self::Float(f) => f.to_string(),
			Self::String(s) => s,
			Self::Ident(i) => i,
			Self::None => String::from("None"),

			Self::Is(expr, ty) => format!("{} is {ty}", expr.format()),
			Self::Cast(expr, ty) => format!("{} as {ty}", expr.format()),
			Self::DotIndex(expr, index) => format!("{}.{index}", expr.format()),
			Self::BracketIndex(expr, index) => format!("{}[{}]", expr.format(), index.format()),
			Self::Call(expr, args) => format!("{}({})", expr.format(), args.into_iter().map(Format::format).collect::<Vec<_>>().join(", ")),

			Self::And(lhs, rhs) => format!("{} && {}", lhs.format(), rhs.format()),
			Self::Or(lhs, rhs) => format!("{} || {}", lhs.format(), rhs.format()),

			Self::Not(expr) => format!("!{}", expr.format()),
			Self::Negate(expr) => format!("-{}", expr.format()),

			Self::Addition(lhs, rhs) => format!("{} + {}", lhs.format(), rhs.format()),
			Self::Subtraction(lhs, rhs) => format!("{} - {}", lhs.format(), rhs.format()),
			Self::Multiplication(lhs, rhs) => format!("{} * {}", lhs.format(), rhs.format()),
			Self::Division(lhs, rhs) => format!("{} / {}", lhs.format(), rhs.format()),

			Self::Equal(lhs, rhs) => format!("{} == {}", lhs.format(), rhs.format()),
			Self::NotEqual(lhs, rhs) => format!("{} != {}", lhs.format(), rhs.format()),

			Self::GreaterThan(lhs, rhs) => format!("{} > {}", lhs.format(), rhs.format()),
			Self::LessThan(lhs, rhs) => format!("{} < {}", lhs.format(), rhs.format()),
			Self::GreaterThanOrEqual(lhs, rhs) => format!("{} >= {}", lhs.format(), rhs.format()),
			Self::LessThanOrEqual(lhs, rhs) => format!("{} <= {}", lhs.format(), rhs.format()),
		}
	}
}

impl<T: Format> Format for Vec<T> {
	fn format(self) -> String {
		if self.is_empty() {
			String::new()
		} else {
			self.into_iter().map(|x| x.format().replace('\n', "\n\t")).collect::<Vec<String>>().join("\n\t")
		}
	}
}

impl Format for Argument {
	fn format(self) -> String {
		match self {
			Self::Anonymous(expr) => expr.format(),
			Self::Named(name, expr) => format!("{} = {}", name, expr.format())
		}
	}
}

impl Format for Parameter {
	fn format(self) -> String {
		match self.2 {
			Some(value) => format!("{} {} = {}", self.0, self.1, value.format()),
			None => format!("{} {}", self.0, self.1)
		}
	}
}

impl Format for Index {
	fn format(self) -> String {
		match self {
			Self::Dot(i) => format!(".{i}"),
			Self::Bracket(i) => format!("[{}]", i.format()),
		}
	}
}

impl Format for crate::parser::Rule {
	fn format(self) -> String {
		match self {
			Self::op_add => String::from("+"),
			Self::op_sub => String::from("-"),
			Self::op_mul => String::from("*"),
			Self::op_div => String::from("/"),
			_ => unreachable!()
		}
	}
}

impl Format for Field {
	fn format(self) -> String {
		match self.2 {
			Some(value) => format!("{} {} = {}", self.0, self.1, value.format()),
			None => format!("{} {}", self.0, self.1)
		}
	}
}

impl Format for Statement {
	fn format(self) -> String {
		match self {
			Statement::If { cond, body, elifs, else_block } => {
				let formatted_elifs = if elifs.is_empty() { String::new() } else { elifs.into_iter().map(|x| format!("elseif\n\t{}\n{}", x.0.format(), x.1.format())).collect() };

				format!("if {}\n\t{}\n{}{}endif", cond.format(), body.format(), formatted_elifs, else_block.map(|x| format!("else\n\t{}\n", x.format())).unwrap_or_default())
			},

			Statement::While { cond, body } => format!("while {}\n\t{}\nendwhile", cond.format(), body.format()),
			Statement::Assignment { name, indexes, value } => {
				if indexes.is_empty() {
					format!("{name} = {}", value.format())
				} else {
					format!("{name}{} = {}", indexes.format(), value.format())
				}
			},

			Statement::Function { return_type, name, parameters, body } => match return_type {
				Some(ret) => format!("{ret} function {name}({})\n\t{}\nendfunction", parameters.into_iter().map(Format::format).collect::<Vec<_>>().join(", "), body.format()),
				None => format!("function {name}({})\n\t{}\nendfunction", parameters.into_iter().map(Format::format).collect::<Vec<_>>().join(", "), body.format()),
			},

			Statement::NativeFunction { return_type, name, parameters } => match return_type {
				Some(ret) => format!("{ret} function {name}({}) native", parameters.into_iter().map(Format::format).collect::<Vec<_>>().join(", ")),
				None => format!("function {name}({}) native", parameters.into_iter().map(Format::format).collect::<Vec<_>>().join(", "))
			},

			Statement::Return { value } => format!("return {}", value.map(Format::format).unwrap_or_default()),
			Statement::Event { name, parameters, body } => format!("event {name}({}) {} endevent", parameters.into_iter().map(Format::format).collect::<Vec<_>>().join(", "), body.format()),

			Statement::PropertyFull { ty, name, functions } => format!("{ty} property {name} {} {} endproperty", functions.0.format(), functions.1.map(|x| x.format()).unwrap_or_default()),
			Statement::PropertyAuto { ty, name, value } => format!("{ty} property {name} = {} auto", value.map(Format::format).unwrap_or_default()),
			Statement::PropertyAutoConst { ty, name, value } => format!("{ty} property {name} = {} AutoReadOnly", value.format()),

			Statement::State { auto, name, body } => {
				if auto {
					format!("state {}\n\t{}\nendstate", name, body.format())
				} else {
					format!("auto state {}\n\t{}\nendstate", name, body.format())
				}
			},

			Statement::Definition { ty, name, value } => format!("{ty} {name} = {}", value.format()),
			Statement::Declaration { ty, name } => format!("{ty} {name}"),
			Statement::Group { name, properties } => format!("group {name} {} endgroup", properties.format()),
			Statement::CompoundAssignment { name, op, value } => format!("{name} {}= {}", op.format(), value.format()),
			Statement::Struct { name, fields } => format!("struct {}\n\t{}\nendstruct", name, fields.format()),
			Statement::Import { item } => format!("import {item}"),

			Statement::Expression { expr } => expr.format(),
		}
	}
}

impl Format for Ast {
	fn format(self) -> String {
		self.statements.into_iter().map(Format::format).collect::<Vec<String>>().join("\n\n")
	}
}