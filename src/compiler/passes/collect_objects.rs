use super::*;
use indexmap::IndexSet;
use std::{
	borrow::Cow,
	collections::{HashMap, HashSet},
};

pub(crate) struct CollectObjects;

#[derive(Debug)]
pub(crate) enum VariableValue {
	Null,
	Identifier(u16),
	String(u16),
	Integer(i32),
	Float(f32),
	Boolean(u8),
}

#[derive(Debug)]
pub(crate) struct Variable {
	name: u16,
	type_name: u16,
	user_flags: u32,

	val: VariableValue,
}

#[derive(Debug)]
pub(crate) struct Instruction {
	op: u8,
	arguments: Vec<VariableValue>,
}

#[derive(Debug)]
pub(crate) enum FunctionFlags {
	Global = 0x01,
	Native = 0x02,
}

#[derive(Debug)]
pub(crate) struct Function {
	name: Option<u16>,
	return_type: u16,
	doc_string: u16,
	user_flags: u32,
	flags: FunctionFlags,
	params: Vec<(u16, u16)>,
	locals: Vec<(u16, u16)>,
	instructions: Vec<Instruction>,
}

#[derive(Debug)]
enum PropertyValue {
	Read(Function),
	Write(Function),
	ReadWrite(Function, Function),
	AutoVar(u16),
}

#[derive(Debug)]
pub(crate) struct Property {
	name: u16,
	type_name: u16,
	doc_string: u16,
	user_flags: u32,
	flags: u8,

	val: PropertyValue,
}

#[derive(Debug)]
pub(crate) struct State {
	name: u16,
	functions: Vec<Function>,
}

#[derive(Debug)]
pub(crate) struct ObjectData {
	parent_class_name: u16,
	doc_string: u16,
	user_flags: u32,
	auto_state_name: u16,

	variables: Vec<Variable>,
	properties: Vec<Property>,
	states: Vec<State>,
}

#[derive(Debug)]
pub(crate) struct Object {
	name: u16, // string table index
	size: u32,
	data: Vec<ObjectData>,
}

#[derive(Debug, Default)]
pub(crate) struct Scope {
	variables: HashMap<String, Type>,
}

#[derive(Debug, Default)]
pub(crate) struct ObjectCollectionState {
	strings: IndexSet<String>,

	scopes: Vec<Scope>,
	objects: Vec<Object>,
}

impl ObjectCollectionState {
	pub(crate) fn new(strings: IndexSet<String>) -> Self {
		Self {
			strings,
			..Default::default()
		}
	}

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

type Userdata = ObjectCollectionState;
impl Pass<Userdata> for CollectObjects {
	fn enter_scope(userdata: &mut Userdata) {
		userdata.scopes.push(Scope::default());
	}

	fn exit_scope(userdata: &mut Userdata) {
		userdata.scopes.pop();
	}

	fn statement(stmt: &Statement, userdata: &mut Userdata) {
		let scope = userdata.scopes.last_mut().unwrap();
		match stmt {
			Statement::Definition { ty, name, value } => {
				// scope.variables.insert(name.clone(), userdata.strings.get(&ty.frag().to_lowercase()).unwrap().clone());
			}
			Statement::Expression { expr } => Self::expression(expr, userdata),
			_ => (),
		}
	}

	fn expression(expr: &Expression, userdata: &mut Userdata) {
		match expr {
			Expression::DotIndex(lhs, ind) => match userdata.solve_expr_type(lhs) {
				Some(Type::Array(_)) if ind == "Length" => {
					println!("Getting length of an array");
				}
				None => panic!("Unable to resolve type ?? {lhs:?}"),
				_ => (),
			},

			_ => (),
		}
	}
}
