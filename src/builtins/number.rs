use crate::*;

#[derive(Clone)]
pub struct Number {
    pub number: f64,
    pub location: Option<SourceLocation>,
}

primitive!(number for Number);

pub(crate) fn setup(env: &mut Environment) {
    // Number ::= Text
    env.add_conformance_for_primitive(TraitID::text(), |number: Number, _, _| {
        Ok(Some(Value::of(Text {
            text: number.number.to_string(),
            location: None,
        })))
    });
}