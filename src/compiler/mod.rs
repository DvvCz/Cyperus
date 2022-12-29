use crate::parser::ast::*;

// mod encode;
mod passes;

trait Pass<Userdata> {
	fn statement(stmt: &Statement, userdata: &mut Userdata) {}
	fn expression(expr: &Expression, userdata: &mut Userdata) {}

	fn enter_scope(userdata: &mut Userdata) {}
	fn exit_scope(userdata: &mut Userdata) {}
}

trait AstWalk<Userdata, P: Pass<Userdata>> {
	fn walk(&self, userdata: &mut Userdata);
}

impl<Userdata, P: Pass<Userdata>, T: AstWalk<Userdata, P>> AstWalk<Userdata, P> for Vec<T> {
	fn walk(&self, userdata: &mut Userdata) {
		P::enter_scope(userdata);
		self.iter().for_each(|x| x.walk(userdata));
		P::exit_scope(userdata);
	}
}

impl<Userdata, P: Pass<Userdata>> AstWalk<Userdata, P> for Statement {
	fn walk(&self, userdata: &mut Userdata) {
		match self {
			Statement::If {
				body,
				elifs,
				else_block,
				..
			} => {
				AstWalk::<Userdata, P>::walk(body, userdata);

				for elif in elifs {
					P::enter_scope(userdata);
					AstWalk::<Userdata, P>::walk(&elif.1, userdata);
					P::exit_scope(userdata);
				}

				if let Some(body) = else_block {
					AstWalk::<Userdata, P>::walk(body, userdata);
				}
			}

			Statement::Function { body, .. } => AstWalk::<Userdata, P>::walk(body, userdata),
			Statement::While { body, .. } => AstWalk::<Userdata, P>::walk(body, userdata),
			Statement::Event { body, .. } => AstWalk::<Userdata, P>::walk(body, userdata),
			Statement::Group { properties, .. } => AstWalk::<Userdata, P>::walk(properties, userdata),

			Statement::PropertyFull { functions, .. } => {
				AstWalk::<Userdata, P>::walk(functions.0.as_ref(), userdata);

				if let Some(body) = functions.1.as_ref() {
					AstWalk::<Userdata, P>::walk(body.as_ref(), userdata);
				}
			}

			_ => (),
		};

		P::statement(self, userdata);
	}
}

pub fn compile(ast: &Ast) -> Vec<u8> {
	use indexmap::IndexSet;
	use passes::{validate::{Validate, State as ValidationState}, string_table::StringTable};

	let mut state = ValidationState::default();
	AstWalk::<_, Validate>::walk(&ast.statements, &mut state);

	let mut strings = IndexSet::new();
	AstWalk::<_, StringTable>::walk(&ast.statements, &mut strings);

	vec![]
}