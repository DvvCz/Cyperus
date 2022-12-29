use super::*;
use indexmap::IndexSet;

pub(crate) struct StringTable;

impl Pass<IndexSet<String>> for StringTable {
	fn statement(stmt: &Statement, userdata: &mut IndexSet<String>) {
		match stmt {
			Statement::If { cond, elifs, .. } => {
				Self::expression(cond, userdata);

				for elif in elifs {
					Self::expression(&elif.0, userdata);
				}
			},

			Statement::Declaration { ty, name } => {
				userdata.insert(ty.clone());
				userdata.insert(name.clone());
			},

			Statement::Definition { ty, name, .. } => {
				userdata.insert(ty.clone());
				userdata.insert(name.clone());
			},

			Statement::Assignment { name, .. } => {
				userdata.insert(name.clone());
			},

			Statement::While { cond, .. } => {
				Self::expression(cond, userdata);
			}

			Statement::Expression { expr } => Self::expression(expr, userdata),

			Statement::Function { return_type, name, parameters, .. } => {
				if let Some(ty) = return_type {
					userdata.insert(ty.clone());
				}

				for param in parameters {
					userdata.insert(param.0.clone());
					userdata.insert(param.1.clone());
					if let Some(val) = &param.2 {
						Self::expression(val, userdata);
					}
				}

				userdata.insert(name.clone());
			}

			Statement::NativeFunction { return_type, name, parameters } => {
				if let Some(ty) = return_type {
					userdata.insert(ty.clone());
				}

				for param in parameters {
					userdata.insert(param.0.clone());
					userdata.insert(param.1.clone());
					if let Some(val) = &param.2 {
						Self::expression(val, userdata);
					}
				}

				userdata.insert(name.clone());
			}

			Statement::Return { value } => if let Some(val) = value {
				Self::expression(val, userdata)
			},

			Statement::Event { name, parameters, .. } => {
				userdata.insert(name.clone());

				for param in parameters {
					userdata.insert(param.0.clone());
					userdata.insert(param.1.clone());
					if let Some(val) = &param.2 {
						Self::expression(val, userdata);
					}
				}
			},

			Statement::PropertyAuto { ty, name, value } => {
				userdata.insert(ty.clone());
				userdata.insert(name.clone());

				if let Some(val) = value {
					Self::expression(val, userdata);
				}
			},

			Statement::PropertyAutoConst { ty, name, value } => {
				userdata.insert(ty.clone());
				userdata.insert(name.clone());
				Self::expression(value, userdata);
			},

			Statement::PropertyFull { ty, name, .. } => {
				userdata.insert(ty.clone());
				userdata.insert(name.clone());
			}

			Statement::State { name, .. } => {
				userdata.insert(name.clone());
			},

			Statement::Group { name, .. } => {
				userdata.insert(name.clone());
			},

			Statement::Struct { name, fields } => {
				userdata.insert(name.clone());

				for field in fields {
					userdata.insert(field.0.clone());
					userdata.insert(field.1.clone());
					if let Some(val) = &field.2 {
						Self::expression(val, userdata);
					}
				}
			},

			Statement::Import { item } => {
				userdata.insert(item.clone());
			},

			Statement::CompoundAssignment { name, value, .. } => {
				userdata.insert(name.clone());
				Self::expression(value, userdata);
			}
		}
	}

	fn expression(expr: &Expression, userdata: &mut IndexSet<String>) {
		match expr {
			Expression::String(s) => { userdata.insert(s.clone()); },
			Expression::Ident(s) => { userdata.insert(s.clone()); },

			_ => ()
		}
	}
}