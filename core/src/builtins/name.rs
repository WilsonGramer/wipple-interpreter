use crate::*;
use std::{collections::HashMap, rc::Rc};

#[derive(Clone)]
pub struct Name {
    pub name: String,
    pub location: Option<SourceLocation>,
}

fundamental_primitive!(name for Name);

#[derive(Clone)]
pub struct AssignFn(pub Rc<dyn Fn(&Value, &EnvironmentRef, &Stack) -> Result<()>>);

impl AssignFn {
    pub fn new(assign: impl Fn(&Value, &EnvironmentRef, &Stack) -> Result<()> + 'static) -> Self {
        AssignFn(Rc::new(assign))
    }
}

fundamental_primitive!(assign for AssignFn);

#[derive(Clone, Copy)]
pub struct Computed;

fundamental_primitive!(computed for Computed);

pub type Variables = HashMap<String, Value>;

fundamental_env_key!(pub variables for Variables {
    EnvironmentKey::new(
        UseFn::new(|parent: &Variables, new| {
            parent.clone().into_iter().chain(new.clone()).collect()
        }),
        true,
    )
});

impl Environment {
    pub fn get_variable(&mut self, name: &str) -> Option<&Value> {
        self.variables().get(name)
    }

    pub fn set_variable(&mut self, name: &str, value: Value) {
        self.variables().insert(String::from(name), value);
    }
}

impl Name {
    pub fn resolve(&self, env: &EnvironmentRef, stack: &Stack) -> Result {
        self.resolve_in(env, env, stack)
    }

    pub fn resolve_in(
        &self,
        resolve_env: &EnvironmentRef,
        compute_env: &EnvironmentRef,
        stack: &Stack,
    ) -> Result {
        let variable = self.resolve_without_computing(resolve_env, stack)?;

        if variable.has_trait(TraitID::computed(), compute_env, &stack)? {
            variable.evaluate(compute_env, &stack)
        } else {
            Ok(variable)
        }
    }

    pub fn resolve_without_computing(&self, env: &EnvironmentRef, stack: &Stack) -> Result {
        let stack = stack.add(|| format!("Resolving variable '{}'", self.name));

        self.resolve_without_computing_if_present(env)
            .ok_or_else(|| {
                ReturnState::Error(Error::new("Name does not refer to a variable", &stack))
            })
    }

    pub fn resolve_without_computing_if_present(&self, env: &EnvironmentRef) -> Option<Value> {
        fn get(name: &Name, env: &EnvironmentRef) -> Option<Value> {
            let variable = env.borrow_mut().variables().get(&name.name).cloned();
            if let Some(variable) = variable {
                return Some(variable);
            }

            let parent = env.borrow_mut().parent.clone();
            parent.and_then(|parent| get(name, &parent))
        }

        get(self, env)
    }
}

pub(crate) fn setup(env: &mut Environment) {
    // Name : trait
    env.set_variable(
        "Name",
        Value::of(TraitConstructor {
            id: TraitID::name(),
            validation: Validation::for_primitive::<Name>(),
        }),
    );

    env.add_primitive_conformance(|name: Name| {
        AssignFn::new(move |value, env, stack| {
            let value = value.evaluate(env, stack)?;
            env.borrow_mut().set_variable(&name.name, value);
            Ok(())
        })
    });

    env.add_primitive_conformance(|name: Name| {
        EvaluateFn::new(move |env, stack| name.resolve(env, stack))
    });

    env.add_primitive_conformance(|name: Name| {
        DefineMacroParameterFn::new(move |value, env, stack| {
            let parameter = MacroParameter(name.name.clone());
            let replacement = value.evaluate(env, stack)?;

            Ok((parameter, replacement))
        })
    });

    env.add_primitive_conformance(|name: Name| {
        MacroExpandFn::new(move |parameter, replacement, _, _| {
            Ok(if name.name == parameter.0 {
                replacement.clone()
            } else {
                Value::of(name.clone())
            })
        })
    });

    env.add_primitive_conformance(|name: Name| Text {
        text: name.name,
        location: None,
    });
}
