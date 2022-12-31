use super::*;
use hashbrown::{HashMap, HashSet};
use indexmap::IndexSet;

pub(crate) struct CollectObjects;

#[derive(Debug)]
pub(crate) enum VariableValue {
	Null,
	Identifier(u16),
	String(u16),
	Integer(i32),
	Float(f32),
	Boolean(u8)
}

#[derive(Debug)]
pub(crate) struct Variable {
	name: u16,
	type_name: u16,
	user_flags: u32,

	val: VariableValue
}

#[derive(Debug)]
pub(crate) struct Instruction {
	op: u8,
	arguments: Vec<VariableValue>
}

#[derive(Debug)]
pub(crate) enum FunctionFlags {
	Global = 0x01,
	Native = 0x02
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
	instructions: Vec<Instruction>
}

#[derive(Debug)]
enum PropertyValue {
	Read(Function),
	Write(Function),
	ReadWrite(Function, Function),
	AutoVar(u16)
}

#[derive(Debug)]
pub(crate) struct Property {
	name: u16,
	type_name: u16,
	doc_string: u16,
	user_flags: u32,
	flags: u8,

	val: PropertyValue
}

#[derive(Debug)]
pub(crate) struct State {
	name: u16,
	functions: Vec<Function>
}

#[derive(Debug)]
pub(crate) struct ObjectData {
	parent_class_name: u16,
	doc_string: u16,
	user_flags: u32,
	auto_state_name: u16,

	variables: Vec<Variable>,
	properties: Vec<Property>,
	states: Vec<State>
}

#[derive(Debug)]
pub(crate) struct Object {
	name: u16, // string table index
	size: u32,
	data: Vec<ObjectData>
}

#[derive(Debug, Default)]
pub(crate) struct Scope {
	variables: HashMap<String, Type>
}

#[derive(Debug, Default)]
pub(crate) struct ObjectCollectionState {
	strings: IndexSet<String>,

	scopes: Vec<Scope>,
	objects: Vec<Object>
}

impl<'pass> ObjectCollectionState {
	pub(crate) fn new(strings: IndexSet<String>) -> Self {
		Self {
			strings,
			..Default::default()
		}
	}

	fn resolve_var(&'pass self, var: &'pass String) -> Option<&Type> {
		self.scopes.iter().rev().find_map(|x| x.variables.get(var))
	}

	// Prob want to replace this with string interning too. Or SmolStr?
	fn solve_expr_type(&'pass mut self, expr: &'pass Expression) -> Option<&'pass Type> {
		match expr {
			Expression::Ident(i) => self.resolve_var(i),
			Expression::String(_) => Some(&Type::STRING),
			Expression::Integer(_) => Some(&Type::INT),
			Expression::Float(_) => Some(&Type::FLOAT),
			Expression::Bool(_) => Some(&Type::BOOL),
			Expression::None => Some(&Type::NONE),
			Expression::Array(ty, ..) => Some(&Type::new(ty.frag().clone(), true)), // todo: add is_array to it.

			// lhs and rhs must match, verified in the validate pass.
			Expression::Addition(lhs, ..) => self.solve_expr_type(lhs),
			Expression::Subtraction(lhs, ..) => self.solve_expr_type(lhs),
			Expression::Multiplication(lhs, ..) => self.solve_expr_type(lhs),
			Expression::Division(lhs, ..) => self.solve_expr_type(lhs),
			// Expression::Modulus(lhs, ..) => Some(&TYPE_INTEGER),

			Expression::Not(_) => Some(&Type::BOOL),
			Expression::Negate(un) => self.solve_expr_type(un),

			Expression::Equal(..) => Some(&Type::BOOL),
			Expression::NotEqual(..) => Some(&Type::BOOL),
			Expression::GreaterThan(..) => Some(&Type::BOOL),
			Expression::LessThan(..) => Some(&Type::BOOL),
			Expression::GreaterThanOrEqual(..) => Some(&Type::BOOL),
			Expression::LessThanOrEqual(..) => Some(&Type::BOOL),

			Expression::And(..) => Some(&Type::BOOL),
			Expression::Or(..) => Some(&Type::BOOL),

			Expression::Cast(_, to) => Some(to),
			Expression::Is(..) => Some(&Type::BOOL),

			tricky => todo!("tricky {tricky:?}")
		}
	}
}

type Userdata = ObjectCollectionState;
impl<'pass> Pass<'pass, Userdata> for CollectObjects {
	fn enter_scope(userdata: &'pass mut Userdata) {
		userdata.scopes.push(Scope::default());
	}

	fn exit_scope(userdata: &'pass mut Userdata) {
		userdata.scopes.pop();
	}

	fn statement(stmt: &'pass Statement, userdata: &'pass mut Userdata) {
		let scope = userdata.scopes.last_mut().unwrap();
		match stmt {
			Statement::Definition { ty, name, value } => {
				// scope.variables.insert(name.clone(), userdata.strings.get(&ty.frag().to_lowercase()).unwrap().clone());
			}
			Statement::Expression { expr } => Self::expression(expr, userdata),
			_ => ()
		}
	}

	fn expression(expr: &'pass Expression, userdata: &'pass mut Userdata) {
		match expr {
			Expression::DotIndex(lhs, ind) => {
				match userdata.solve_expr_type(lhs) {
					Some(ty) if ty.is_array() && ind == "Length" => {
						println!("Getting length of an array");
					},
					None => panic!("Unable to resolve type ?? {lhs:?}"),
					_ => (),
				}
			}

			_ => ()
		}
	}
}