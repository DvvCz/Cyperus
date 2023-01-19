use super::*;
use std::collections::{HashMap, HashSet};

pub(crate) struct Validate;

#[derive(Debug, Default)]
pub(crate) struct Scope {
	variables: HashMap<String, Type>,
}

#[derive(Debug, Default)]
pub(crate) struct ValidationState {
	functions: HashSet<String>,
	scopes: Vec<Scope>,
}

impl ValidationState {
	fn resolve_var(&self, var: &String) -> Option<&Type> {
		self.scopes.iter().rev().find_map(|x| x.variables.get(var))
	}

	// Prob want to replace this with string interning too. Or SmolStr?
	fn solve_expr_type<'a>(&'a self, expr: &'a Expression) -> Option<&'a Type> {
		match expr {
			Expression::Ident(i) => self.resolve_var(i),
			Expression::String(_) => Some(&Type::String),
			Expression::Integer(_) => Some(&Type::Integer),
			Expression::Float(_) => Some(&Type::Float),
			Expression::Bool(_) => Some(&Type::Boolean),
			Expression::None => Some(&Type::None),
			Expression::Array(ty, ..) => Some(ty),

			// lhs and rhs must match, verified in the validate pass.
			Expression::Addition(lhs, ..) => self.solve_expr_type(lhs),
			Expression::Subtraction(lhs, ..) => self.solve_expr_type(lhs),
			Expression::Multiplication(lhs, ..) => self.solve_expr_type(lhs),
			Expression::Division(lhs, ..) => self.solve_expr_type(lhs),
			// Expression::Modulus(lhs, ..) => Some(&TYPE_INTEGER),
			Expression::Not(_) => Some(&Type::Boolean),
			Expression::Negate(un) => self.solve_expr_type(un),

			Expression::Equal(..) => Some(&Type::Boolean),
			Expression::NotEqual(..) => Some(&Type::Boolean),
			Expression::GreaterThan(..) => Some(&Type::Boolean),
			Expression::LessThan(..) => Some(&Type::Boolean),
			Expression::GreaterThanOrEqual(..) => Some(&Type::Boolean),
			Expression::LessThanOrEqual(..) => Some(&Type::Boolean),

			Expression::And(..) => Some(&Type::Boolean),
			Expression::Or(..) => Some(&Type::Boolean),

			Expression::Cast(_, to) => Some(to),
			Expression::Is(..) => Some(&Type::Boolean),

			tricky => todo!("tricky {tricky:?}"),
		}
	}
}

type Userdata = ValidationState;
impl FalliblePass<Userdata> for Validate {
	#[inline(always)]
	fn enter_scope(userdata: &mut Userdata) {
		userdata.scopes.push(Scope::default());
	}

	#[inline(always)]
	fn exit_scope(userdata: &mut Userdata) {
		userdata.scopes.pop();
	}

	fn statement(stmt: &Statement, userdata: &mut Userdata) -> Result<()> {
		let scope = userdata.scopes.last_mut().unwrap();

		fn resolve<'state>(
			var: &'state String,
			userdata: &'state mut Userdata,
		) -> Option<&'state Type> {
			userdata
				.scopes
				.iter()
				.rev()
				.find_map(|x| x.variables.get(var))
		}

		match stmt {
			Statement::Declaration { ty, name } => {
				if scope.variables.get(name).is_some() {
					return Err(Error::Validation(format!(
						"Declaring already existing variable {name}"
					)));
				} else {
					scope.variables.insert(name.clone(), ty.clone());
				}
			}

			Statement::Definition { ty, name, value } => {
				if scope.variables.get(name).is_some() {
					return Err(Error::Validation(format!(
						"Defining already existing variable {name}"
					)));
				} else {
					scope.variables.insert(name.clone(), ty.clone());
				}
			}

			Statement::Assignment {
				name,
				indexes,
				value,
			} => {
				match resolve(name, userdata).cloned() {
					Some(ref expected) => {
						match userdata.solve_expr_type(value) {
							Some(t) if t != expected => {},
							Some(unknown) => return Err(Error::Validation(format!(
								"Assigning type {unknown} to variable of type {expected}"
							))),
							_ => unimplemented!("Failed to solve type for value")
						}
					}
					None => {
						return Err(Error::Validation(format!(
							"Assigning to undeclared variable {name}"
						)))
					}
				}
			}

			_ => (),
		}

		Ok(())
	}

	fn expression(expr: &Expression, userdata: &mut Userdata) -> Result<()> {
		Ok(())
	}
}
