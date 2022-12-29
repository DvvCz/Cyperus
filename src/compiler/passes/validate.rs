use super::*;
use std::collections::{HashMap, HashSet};

pub(crate) struct Validate;

#[derive(Debug, Default)]
pub(crate) struct Scope {
	variables: HashMap<String, String>
}

#[derive(Debug, Default)]
pub(crate) struct State {
	functions: HashSet<String>,
	scopes: Vec<Scope>
}

impl Pass<State> for Validate {
	#[inline(always)]
	fn enter_scope(userdata: &mut State) {
		userdata.scopes.push(Scope::default());
	}

	#[inline(always)]
	fn exit_scope(userdata: &mut State) {
		userdata.scopes.pop();
	}

	fn statement(stmt: &Statement, userdata: &mut State) {
		let scope = userdata.scopes.last_mut().unwrap();

		fn resolve<'a>(var: &'a String, userdata: &'a mut State) -> Option<&'a String> {
			userdata.scopes.iter().rev().find_map(|x| x.variables.get(var))
		}

		match stmt {
			Statement::Declaration { ty, name } => {
				if scope.variables.get(name).is_some() {
					panic!("variable {name} declared multiple times");
				} else {
					scope.variables.insert(name.clone(), ty.clone());
				}
			},

			Statement::Definition { ty, name, value } => {
				if scope.variables.get(name).is_some() {
					panic!("variable {name} declared multiple times");
				} else {
					scope.variables.insert(name.clone(), ty.clone());
				}
			},

			Statement::Assignment { name, indexes, value } => {
				match resolve(name, userdata) {
					Some(ty) => {
						// Todo: Resolve what type "value" is and compare.
					},
					None => panic!("variable {name} not declared")
				}
			}

			_ => ()
		}
	}

	fn expression(expr: &Expression, userdata: &mut State) {

	}
}