use crate::*;

#[derive(Clone)]
pub struct Module {
    pub values: EnvironmentValues,
}

impl Module {
    pub fn new(values: EnvironmentValues) -> Self {
        Module { values }
    }

    pub fn from(env: &EnvironmentRef) -> Self {
        Module::new(env.borrow().values.clone())
    }

    pub fn env(&self) -> Environment {
        Environment {
            values: self.values.clone(),
            parent: None,
        }
    }
}

primitive!(module for Module);

#[derive(Clone)]
pub struct ModuleBlock {
    pub statements: Vec<List>,
    pub location: Option<SourceLocation>,
}

primitive!(module_block for ModuleBlock);

pub(crate) fn setup(env: &mut Environment) {
    // Module ::= Text
    env.add_conformance_for_primitive(TraitID::text(), |_: Module, _, _| {
        Ok(Some(Value::of(Text {
            text: String::from("<module>"),
            location: None,
        })))
    });

    // Module ::= Function
    env.add_conformance_for_primitive(TraitID::function(), |module: Module, _, _| {
        Ok(Some(Value::of(Function::new(move |value, env, stack| {
            let name = value.get_primitive_or::<Name>("Expected a name", env, stack)?;

            name.resolve(&module.env().into_ref(), stack)
        }))))
    });

    // Module-Block ::= Text
    env.add_conformance_for_primitive(TraitID::text(), |_: ModuleBlock, _, _| {
        Ok(Some(Value::of(Text {
            text: String::from("<module block>"),
            location: None,
        })))
    });

    // Module-Block ::= Evaluate
    env.add_conformance_for_primitive(TraitID::evaluate(), |module_block: ModuleBlock, _, _| {
        Ok(Some(Value::of(EvaluateFn::new(move |env, stack| {
            let mut stack = stack.clone();
            if let Some(location) = &module_block.location {
                stack.queue_location(location);
            }

            // Modules capture their environment
            // let captured_env = env.child().into_ref();
            let captured_env = Environment::child_of(env).into_ref();

            for statement in &module_block.statements {
                let mut stack = stack.clone();
                if let Some(location) = &statement.location {
                    stack.queue_location(location);
                }

                // Evaluate each statement as a list
                let list = Value::of(statement.clone());
                list.evaluate(&captured_env, &stack)?;
            }

            Ok(Value::of(Module::from(&captured_env)))
        }))))
    });

    // Module-Block ::= Macro-Expand
    env.add_conformance_for_primitive(
        TraitID::macro_expand(),
        |module_block: ModuleBlock, _, _| {
            Ok(Some(Value::of(MacroExpandFn::new(
                move |parameter, replacement, env, stack| {
                    // Module blocks expand the same way as blocks

                    let block = Value::of(Block {
                        statements: module_block.statements.clone(),
                        location: module_block.location.clone(),
                    });

                    let expanded_block = block
                        .macro_expand(parameter, replacement, env, stack)?
                        .get_primitive::<Block>(env, stack)?;

                    Ok(Value::of(ModuleBlock {
                        statements: expanded_block.statements,
                        location: expanded_block.location,
                    }))
                },
            ))))
        },
    );
}