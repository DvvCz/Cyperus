use crate::parser::ast::*;

mod error;
use error::{Error, Result};

// mod encode;
mod passes;

trait Pass<'pass, Userdata> {
	fn statement(_: &'pass Statement, _: &'pass mut Userdata) {}
	fn expression(_: &'pass Expression, _: &'pass mut Userdata) {}

	fn enter_scope(_: &'pass mut Userdata) {}
	fn exit_scope(_: &'pass mut Userdata) {}
}

trait AstWalk<'walk, Userdata, P: Pass<'walk, Userdata>> {
	fn walk(&'walk self, userdata: &'walk mut Userdata);
}

impl<'walk, Userdata, P: Pass<'walk, Userdata>, T: AstWalk<'walk, Userdata, P>> AstWalk<'walk, Userdata, P> for Vec<T> {
	fn walk(&'walk self, userdata: &'walk mut Userdata) {
		P::enter_scope(userdata);
		for x in self {
			x.walk(userdata);
		}
		P::exit_scope(userdata);
	}
}

impl<'walk, Userdata, P: Pass<'walk, Userdata>> AstWalk<'walk, Userdata, P> for Statement {
	fn walk(&'walk self, userdata: &'walk mut Userdata) {
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

trait FalliblePass<Userdata> {
	fn statement(stmt: &Statement, userdata: &mut Userdata) -> Result<()>;
	fn expression(expr: &Expression, userdata: &mut Userdata) -> Result<()>;

	fn enter_scope(_: &mut Userdata) {}
	fn exit_scope(_: &mut Userdata) {}
}

trait FallibleAstWalk<Userdata, P: FalliblePass<Userdata>> {
	fn walk(&self, userdata: &mut Userdata) -> Result<()>;
}

impl<Userdata, P: FalliblePass<Userdata>, T: FallibleAstWalk<Userdata, P>> FallibleAstWalk<Userdata, P> for Vec<T> {
	fn walk(&self, userdata: &mut Userdata) -> Result<()> {
		P::enter_scope(userdata);
		for elem in self {
			elem.walk(userdata)?;
		}
		P::exit_scope(userdata);
		Ok(())
	}
}

impl<Userdata, P: FalliblePass<Userdata>> FallibleAstWalk<Userdata, P> for Statement {
	fn walk(&self, userdata: &mut Userdata) -> Result<()> {
		match self {
			Statement::If {
				body,
				elifs,
				else_block,
				..
			} => {
				FallibleAstWalk::<Userdata, P>::walk(body, userdata)?;

				for elif in elifs {
					P::enter_scope(userdata);
					FallibleAstWalk::<Userdata, P>::walk(&elif.1, userdata)?;
					P::exit_scope(userdata);
				}

				if let Some(body) = else_block {
					FallibleAstWalk::<Userdata, P>::walk(body, userdata)?;
				}
			}

			Statement::Function { body, .. } => FallibleAstWalk::<Userdata, P>::walk(body, userdata)?,
			Statement::While { body, .. } => FallibleAstWalk::<Userdata, P>::walk(body, userdata)?,
			Statement::Event { body, .. } => FallibleAstWalk::<Userdata, P>::walk(body, userdata)?,
			Statement::Group { properties, .. } => FallibleAstWalk::<Userdata, P>::walk(properties, userdata)?,

			Statement::PropertyFull { functions, .. } => {
				FallibleAstWalk::<Userdata, P>::walk(functions.0.as_ref(), userdata)?;

				if let Some(body) = functions.1.as_ref() {
					FallibleAstWalk::<Userdata, P>::walk(body.as_ref(), userdata)?;
				}
			}

			_ => (),
		};

		P::statement(self, userdata)
	}
}


pub fn compile(ast: &Ast) -> Vec<u8> {
	use indexmap::IndexSet;
	use passes::{validate::{Validate, ValidationState}, collect_strings::CollectStrings, collect_objects::{CollectObjects, ObjectCollectionState}};

	let mut state = ValidationState::default();
	if let Err(why) = FallibleAstWalk::<_, Validate>::walk(&ast.statements, &mut state) {
		println!("Failed to validate {why}");
	}

	let mut strings = IndexSet::new();
	AstWalk::<_, CollectStrings>::walk(&ast.statements, &mut strings);

	let mut objects = ObjectCollectionState::new(strings);
	AstWalk::<_, CollectObjects>::walk(&ast.statements, &mut objects);

	vec![]
}