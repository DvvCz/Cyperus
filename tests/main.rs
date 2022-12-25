#[macro_use]
extern crate pest_derive;

#[derive(Parser)]
#[grammar = "papyrus.pest"]
struct PapyrusParser;

fn should_parse(rule: Rule, source: impl AsRef<str>) {
	use pest::Parser;
	if let Err(why) = PapyrusParser::parse(rule, source.as_ref()) {
		panic!("Failed to parse: {why}");
	}
}

#[test]
fn test_script_name() {
	for case in [
		"Scriptname Test Hidden",
		"Scriptname Test Extends Another",
		"ScriptName Test Extends Another Conditional",
	] {
		should_parse(Rule::script_name, case);
	}
}

#[test]
fn test_group() {
	for case in ["Group MyGroup
			{A group containing properties}
			int Property FirstProperty auto
			float Property SecondProperty auto
		EndGroup"]
	{
		should_parse(Rule::group, case);
	}
}

#[test]
fn test_property() {
	for case in [
		"int property myInt
			int function get()
				return myInt_Var
			endFunction

			function set(int value)
				myInt_Var = value
			endFunction
		endProperty",
		"int property ReadOnly
			int function get()
				return myVar
			endFunction
		endProperty",
		"int property myInt = 5 auto",
		"int property myConstProperty auto const",
		"int property myConstProperty auto const mandatory",
		"int property myReadOnlyInt = 20 autoReadOnly",
		"int property myVar auto conditional",
	] {
		should_parse(Rule::property, case);
	}
}

#[test]
fn test_state() {
	for case in ["State MyState EndState", "Auto State StartHere EndState"] {
		should_parse(Rule::state, case);
	}
}

#[test]
fn test_event() {
	for case in [
		"Event OnHit(ObjectReference akAggressor, Form akWeapon, Projectile akProjectile) EndEvent",
		"Event Test() EndEvent",
	] {
		should_parse(Rule::event, case);
	}
}

#[test]
fn test_if() {
	for case in [
		"if true endif",
		"if false elseif true endif",
		"if false int x = 1 else int y = 2 endif",
	] {
		should_parse(Rule::r#if, case);
	}
}

#[test]
fn test_while() {
	for case in [
		"while true endwhile",
		"while 2 + 7 endwhile",
		"while true int x = 5 endwhile",
	] {
		should_parse(Rule::r#while, case);
	}
}

#[test]
fn test_function() {
	for case in [
		"bool Function LoadCharacter(Actor a, Race b, string c = \"test\") native global",
		"int Function GetVersion(int bar = 55) global return 2 EndFunction",
		"int Function GetScriptVersion() return 2 EndFunction",
		"Function DoNothing() return EndFunction",
	] {
		should_parse(Rule::function, case);
	}
}

#[test]
fn test_compound_assignment() {
	for case in [
		"foo += 55",
		"foo -= 55",
		"foo *= 55",
		"foo /= 55",
		"foo %= 55",
	] {
		should_parse(Rule::compound_assignment, case);
	}
}

#[test]
fn test_definition() {
	for case in [
		"int x = 5",
		"float y = 55.6 const",
		"float[] z = new float[5]",
		"MyScript[] x = new MyScript[5 * count]",
	] {
		should_parse(Rule::definition, case);
	}
}

#[test]
fn test_declaration() {
	for case in [
		"int x",
		"float y",
		"float[] z",
		"Race test",
		"Actor thatguy",
	] {
		should_parse(Rule::declaration, case);
	}
}

#[test]
fn test_expression() {
	for case in [
		"xyz as Actor",
		"(foo is Actor) + 2",
		"1 + 2 - 3 * (4 / 5)",
		"5 >= 2",
		"1 > 2",
		"1 < 3",
		"4 <= 6",
		"4 == 2",
		"!(True && False)",
		"2 != True",
		"-x",
		"MyFunction().MyProperty",
		"(MyVariable as MyObject).MyFunction()[0]",
		"new float[5]",
		"new int[x + 2]", // Note accepting a runtime expression here is specific to Fallout 4.
	] {
		should_parse(Rule::expression, case);
	}
}

#[test]
fn test_number() {
	for case in [
		"0xFFFF",
		"44.24572334",
		"0.0",
		"5555555555555555555555555",
		"1.2f",
	] {
		should_parse(Rule::number, case);
	}
}

#[test]
fn test_string() {
	for case in [
		r#""test""#,     // "test"
		r#""test\\\"""#, // "test\\\""
		r#""""#,         // ""
	] {
		should_parse(Rule::string, case);
	}
}

#[test]
fn test_ident() {
	for case in ["hello", "HELLO", "TES_42T2"] {
		should_parse(Rule::ident, case);
	}
}
