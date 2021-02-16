use crate::builtins::*;
use crate::fundamentals::*;
use std::rc::Rc;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Name(pub String);

simple_trait! {
    name: name,
    type: Name,
    label: "Name",
}

#[derive(Clone)]
pub struct AssignFn(pub Rc<dyn Fn(Value, &mut Environment, &ProgramStack) -> Result<()>>);

impl AssignFn {
    pub fn new(
        assign: impl Fn(Value, &mut Environment, &ProgramStack) -> Result<()> + 'static,
    ) -> AssignFn {
        AssignFn(Rc::new(assign))
    }
}

simple_trait! {
    name: assign,
    type: AssignFn,
    label: "Assign",
}

#[derive(Debug, Clone)]
pub struct Computed;

simple_trait! {
    name: computed,
    type: Computed,
    label: "Computed",
}

pub(crate) fn init(env: &mut Environment) {
    // Name : trait
    env.variables.insert(
        String::from("Name"),
        Value::new(Trait::trait_constructor(TraitConstructor::new(
            TraitID::name,
            TraitID::name.validation(),
        ))),
    );

    // Name ::= Assign
    env.add_conformance(Conformance::new(
        TraitID::assign,
        TraitID::name.validation(),
        |name, _, _| {
            let name = name.clone();

            Ok(AssignFn::new(move |value, env, stack| {
                let value = value.evaluate(env, stack)?;
                env.variables.insert(name.0.clone(), value);
                Ok(())
            }))
        },
    ));

    // Name ::= Evaluate
    env.add_conformance(Conformance::new(
        TraitID::evaluate,
        TraitID::name.validation(),
        |name, _, _| {
            let name = name.clone();

            Ok(EvaluateFn::new(move |env, stack| {
                let stack = stack.add(&format!("Resolving variable '{}'", name.0));

                let variable = match env.variables.get(&name.0) {
                    Some(variable) => variable.clone(),
                    None => {
                        return Err(ProgramError::new(
                            "Name does not refer to a variable",
                            &stack,
                        ))
                    }
                };

                if variable.has_trait(TraitID::computed, env, &stack)? {
                    variable.evaluate(env, &stack)
                } else {
                    Ok(variable)
                }
            }))
        },
    ));

    // Name ::= Macro-Parameter
    env.add_conformance(Conformance::new(
        TraitID::macro_parameter,
        TraitID::name.validation(),
        |name, _, _| {
            let name = name.clone();

            Ok(DefineMacroParameterFn::new(move |input, env, stack| {
                let parameter = MacroParameter(name.clone());
                let replacement = input.evaluate(env, stack)?;

                Ok((parameter, replacement))
            }))
        },
    ));

    // Name ::= Macro-Expand
    env.add_conformance(Conformance::new(
        TraitID::macro_expand,
        TraitID::name.validation(),
        |name, _, _| {
            let name = name.clone();

            Ok(MacroExpandFn::new(move |parameter, replacement, _, _| {
                Ok(if name == parameter.0 {
                    replacement
                } else {
                    Value::new(Trait::name(name.clone()))
                })
            }))
        },
    ));

    // Name ::= Text
    env.add_conformance(Conformance::new(
        TraitID::text,
        TraitID::name.validation(),
        |name, _, _| Ok(Text(name.0.clone())),
    ));
}
