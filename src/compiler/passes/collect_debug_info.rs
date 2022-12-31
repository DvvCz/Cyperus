use super::*;
use indexmap::IndexSet;

pub(crate) struct CollectDebugInfo;

#[repr(u8)]
pub(crate) enum FunctionType {
	Unknown = 0,
	X,
	Y,
	Z
}

pub(crate) struct DebugFunction {
	object_name: u16,
	state_name: u16,
	function_name: u16,
	function_type: FunctionType,

	// instruction => line in source
	lines: Vec<u16>
}

type Userdata = Vec<DebugFunction>;
impl Pass<'_, Userdata> for CollectDebugInfo {
	fn statement(stmt: &Statement, userdata: &mut Userdata) {
	}

	fn expression(expr: &Expression, userdata: &mut Userdata) {
	}
}