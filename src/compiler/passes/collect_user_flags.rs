use super::*;

pub(crate) struct CollectUserFlags;

pub(crate) struct UserFlag {
	name: u16, // string table index
	flag: u8,
}

type Userdata = Vec<UserFlag>;
impl Pass<Userdata> for CollectUserFlags {
	fn statement(stmt: &Statement, userdata: &mut Userdata) {}

	fn expression(expr: &Expression, userdata: &mut Userdata) {}
}
