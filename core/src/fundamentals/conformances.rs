use crate::*;
use std::rc::Rc;

pub type Conformances = Vec<Conformance>;

fundamental_env_key!(pub conformances for Conformances {
    EnvironmentKey::new(
        UseFn::new(|parent: &Conformances, new| {
            parent.clone().into_iter().chain(new.clone()).collect()
        }),
        true,
    )
});

#[derive(Clone)]
pub struct Conformance {
    pub derived_trait_id: TraitID,

    #[allow(clippy::type_complexity)]
    pub derive_trait_value: Rc<dyn Fn(&Value, &EnvironmentRef, &Stack) -> Result<Option<Value>>>,
}

impl Environment {
    pub fn add_conformance(
        &mut self,
        derived_trait_id: TraitID,
        derive_trait_value: impl Fn(&Value, &EnvironmentRef, &Stack) -> Result<Option<Value>> + 'static,
    ) {
        self.conformances().push(Conformance {
            derived_trait_id,
            derive_trait_value: Rc::new(derive_trait_value),
        })
    }

    pub fn add_primitive_conformance<A: Primitive, B: Primitive>(
        &mut self,
        derive_trait_value: impl Fn(A) -> B + 'static,
    ) {
        self.add_conformance(TraitID::new_primitive::<B>(), move |value, env, stack| {
            let a = match value.get_primitive_if_present::<A>(env, stack)? {
                Some(primitive) => primitive,
                None => return Ok(None),
            };

            let b = derive_trait_value(a);

            Ok(Some(Value::of(b)))
        });
    }
}
