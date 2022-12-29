use super::*;

pub struct ConstEval;

impl Optimizer for ConstEval {
	fn statement(stmt: &mut Statement) {
		match stmt {
			Statement::If { cond, .. } => Self::expression(cond),
			Statement::While { cond, .. } => Self::expression(cond),

			Statement::Definition { value, .. } => Self::expression(value),
			Statement::Assignment { value, .. } => Self::expression(value),
			Statement::CompoundAssignment { value, .. } => Self::expression(value),

			Statement::Expression { expr } => Self::expression(expr),
			_ => (),
		}
	}

	fn expression(expr: &mut Expression) {
		match expr {
			Expression::Addition(lhs, rhs) => match lhs.as_mut() {
				Expression::Integer(lhs) => match rhs.as_mut() {
					Expression::Integer(rhs) => {
						*expr = Expression::Integer(*lhs + *rhs);
					}
					_ => Self::expression(rhs),
				},

				Expression::Float(lhs) => match rhs.as_mut() {
					Expression::Float(rhs) => {
						*expr = Expression::Float(*lhs + *rhs);
					}
					_ => Self::expression(rhs),
				},

				_ => Self::expression(lhs),
			},

			Expression::Subtraction(lhs, rhs) => match lhs.as_mut() {
				Expression::Integer(lhs) => match rhs.as_mut() {
					Expression::Integer(rhs) => {
						*expr = Expression::Integer(*lhs - *rhs);
					}
					_ => Self::expression(rhs),
				},

				Expression::Float(lhs) => match rhs.as_mut() {
					Expression::Float(rhs) => {
						*expr = Expression::Float(*lhs - *rhs);
					}
					_ => Self::expression(rhs),
				},

				_ => Self::expression(lhs),
			},

			Expression::Multiplication(lhs, rhs) => match lhs.as_mut() {
				Expression::Integer(lhs) => match rhs.as_mut() {
					Expression::Integer(rhs) => {
						*expr = Expression::Integer(*lhs * *rhs);
					}
					_ => Self::expression(rhs),
				},

				Expression::Float(lhs) => match rhs.as_mut() {
					Expression::Float(rhs) => {
						*expr = Expression::Float(*lhs * *rhs);
					}
					_ => Self::expression(rhs),
				},

				_ => Self::expression(lhs),
			},

			Expression::Division(lhs, rhs) => match lhs.as_mut() {
				Expression::Integer(lhs) => match rhs.as_mut() {
					Expression::Integer(rhs) => {
						*expr = Expression::Integer(*lhs / *rhs);
					}
					_ => Self::expression(rhs),
				},

				Expression::Float(lhs) => match rhs.as_mut() {
					Expression::Float(rhs) => {
						*expr = Expression::Float(*lhs / *rhs);
					}
					_ => Self::expression(rhs),
				},

				_ => Self::expression(lhs),
			},

			Expression::And(lhs, rhs) => match lhs.as_mut() {
				Expression::Bool(lhs) => match rhs.as_mut() {
					Expression::Bool(rhs) => {
						*expr = Expression::Bool(*lhs && *rhs);
					}
					_ => Self::expression(rhs),
				},
				_ => Self::expression(lhs),
			},

			Expression::Or(lhs, rhs) => match lhs.as_mut() {
				Expression::Bool(lhs) => match rhs.as_mut() {
					Expression::Bool(rhs) => {
						*expr = Expression::Bool(*lhs || *rhs);
					}
					_ => Self::expression(rhs),
				},

				_ => {
					Self::expression(lhs);
					Self::expression(rhs);
				}
			},

			Expression::Equal(lhs, rhs) => match lhs.as_mut() {
				Expression::Integer(lhs) => match rhs.as_mut() {
					Expression::Integer(rhs) => {
						*expr = Expression::Bool(*lhs == *rhs);
					}
					_ => Self::expression(rhs),
				},

				Expression::Bool(lhs) => match rhs.as_mut() {
					Expression::Bool(rhs) => {
						*expr = Expression::Bool(*lhs == *rhs);
					}
					_ => Self::expression(rhs),
				},

				Expression::String(lhs) => match rhs.as_mut() {
					Expression::String(rhs) => {
						*expr = Expression::Bool(*lhs == *rhs);
					}
					_ => Self::expression(rhs),
				},

				_ => {
					Self::expression(lhs);
					Self::expression(rhs);
				}
			},

			Expression::Not(unary) => match unary.as_mut() {
				Expression::Bool(unary) => {
					*expr = Expression::Bool(!*unary);
				}
				_ => Self::expression(unary),
			},

			Expression::Negate(unary) => match unary.as_mut() {
				Expression::Negate(val) => {
					*expr = *val.clone();
				}
				_ => Self::expression(unary),
			},

			_ => (),
		}
	}
}
