use crate::parser::ast::*;

mod passes;

trait Optimizer {
	fn statement(stmt: &mut Statement);
	fn expression(expr: &mut Expression);
}

trait AstWalk<O: Optimizer> {
	fn walk(&mut self);
}

impl<O: Optimizer, T: AstWalk<O>> AstWalk<O> for Vec<T> {
	fn walk(&mut self) {
		self.iter_mut().for_each(|x| x.walk());
	}
}

impl<O: Optimizer> AstWalk<O> for Statement {
	fn walk(&mut self) {
		match self {
			Statement::If {
				body,
				elifs,
				else_block,
				..
			} => {
				AstWalk::<O>::walk(body);

				for elif in elifs {
					AstWalk::<O>::walk(&mut elif.1);
				}

				if let Some(body) = else_block {
					AstWalk::<O>::walk(body);
				}
			}

			Statement::Function { body, .. } => AstWalk::<O>::walk(body),
			Statement::While { body, .. } => AstWalk::<O>::walk(body),
			Statement::Event { body, .. } => AstWalk::<O>::walk(body),

			Statement::PropertyFull { functions, .. } => {
				AstWalk::<O>::walk(functions.0.as_mut());

				if let Some(body) = functions.1.as_mut() {
					AstWalk::<O>::walk(body.as_mut());
				}
			}

			_ => (),
		};

		O::statement(self)
	}
}

pub fn optimize(ast: &mut Ast) {
	for stmt in &mut ast.statements {
		for _ in 0..5 {
			// 5 passes of constant folding
			AstWalk::<passes::ConstEval>::walk(stmt);
		}
	}
}
