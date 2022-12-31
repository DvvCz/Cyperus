use super::*;
use std::collections::{HashMap, HashSet};

pub(crate) struct Validate;

#[derive(Debug, Default)]
pub(crate) struct Scope {
	variables: HashMap<String, Type>
}

#[derive(Debug, Default)]
pub(crate) struct ValidationState {
	functions: HashSet<String>,
	scopes: Vec<Scope>
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

		fn resolve<'a>(var: &'a String, userdata: &'a mut Userdata) -> Option<&'a Type> {
			userdata.scopes.iter().rev().find_map(|x| x.variables.get(var))
		}

		match stmt {
			Statement::Declaration { ty, name } => {
				if scope.variables.get(name).is_some() {
					return Err(Error::Validation(format!("Declaring already existing variable {name}")));
				} else {
					scope.variables.insert(name.clone(), ty.clone());
				}
			},

			Statement::Definition { ty, name, value } => {
				if scope.variables.get(name).is_some() {
					return Err(Error::Validation(format!("Defining already existing variable {name}")));
				} else {
					scope.variables.insert(name.clone(), ty.clone());
				}
			},

			Statement::Assignment { name, indexes, value } => {
				match resolve(name, userdata) {
					Some(ty) => {
						// Todo: Resolve what type "value" is and compare.
					},
					None => return Err(Error::Validation(format!("Assigning to undeclared variable {name}")))
				}
			}

			_ => ()
		}

		Ok(())
	}

	fn expression(expr: &Expression, userdata: &mut Userdata) -> Result<()> {
		Ok(())
	}
}