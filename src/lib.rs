#[macro_use]
pub mod fundamentals;
pub mod builtins;

pub use builtins::*;
pub use fundamentals::*;

use num_rational::BigRational;
use std::rc::Rc;

pub fn init(env: &mut fundamentals::Environment) {
    builtins::init(env);

    // TODO: TEMPORARY (implement in Wipple code)

    // 'new' function

    fn new(trait_constructor: TraitConstructor, value: Value, env: &mut Environment) -> Result {
        let value = match trait_constructor.validation.validate(&value, env)?.unwrap() {
            ValidationResult::Valid(value) => value,
            ValidationResult::Invalid => {
                return Err(ProgramError::new(
                    "Cannot use this value to represent this trait",
                ))
            }
        };

        Ok(Value::new(Trait::new(trait_constructor.id, move |_| {
            Ok(value.clone())
        })))
    }

    env.variables.insert(
        String::from("new"),
        Value::new(Trait::function(Function::new(|input, env| {
            let trait_constructor = input.get_trait(TraitID::trait_constructor, env)?;

            Ok(Value::new(Trait::function(Function::new(
                move |input, env| new(trait_constructor.clone(), input, env),
            ))))
        }))),
    );

    // 'do' function

    env.variables.insert(
        String::from("do"),
        Value::new(Trait::function(Function::new(|input, env| {
            let mut inner_env = env.clone();

            input.evaluate(&mut inner_env)
        }))),
    );

    // Assignment operator (::)

    fn group(list: Vec<Value>) -> Value {
        if list.len() == 1 {
            list[0].clone()
        } else {
            Value::new(Trait::list(List(list)))
        }
    };

    fn assign(
        left: Vec<Value>,
        right: impl Fn(&mut Environment) -> Result + 'static,
        env: &mut Environment,
    ) -> Result {
        let assign = group(left)
            .get_trait_if_present(TraitID::assign, env)?
            .ok_or_else(|| {
                ProgramError::new(
                    "Cannot assign to this value because it does not have the Assign trait",
                )
            })?;

        let right = right(env)?.evaluate(env)?;

        assign.0(right, env)?;

        Ok(Value::empty())
    };

    let assignment_precedence_group = env.precedence_group(
        Associativity::Right,
        PrecedenceGroupComparison::<VariadicPrecedenceGroup>::highest(),
    );

    let assignment_operator = VariadicOperator::collect(|left, right, env| {
        assign(left, move |_| Ok(group(right.clone())), env)
    });

    env.add_variadic_operator(&assignment_operator, &assignment_precedence_group);
    env.variables.insert(
        String::from(":"),
        Value::new(Trait::operator(Operator::Variadic(assignment_operator))),
    );

    // Trait operator (::)

    let trait_operator = VariadicOperator::collect(|left, right, env| {
        assign(
            left,
            move |env| {
                // TODO: Auto-derive a value for the trait using the conformance
                // if only the trait is provided
                if right.len() != 2 {
                    return Err(ProgramError::new(
                        "Expected a trait and a value for the trait",
                    ));
                }

                let trait_constructor = right[0]
                    .evaluate(env)?
                    .get_trait(TraitID::trait_constructor, env)?;

                let value = new(trait_constructor, right[1].evaluate(env)?, env)?;

                Ok(value)
            },
            env,
        )
    });

    env.add_variadic_operator(&trait_operator, &assignment_precedence_group);
    env.variables.insert(
        String::from("::"),
        Value::new(Trait::operator(Operator::Variadic(trait_operator.clone()))),
    );

    // Macro operator (=>)

    let function_precedence_group = env.precedence_group(
        Associativity::Right,
        PrecedenceGroupComparison::<VariadicPrecedenceGroup>::lower_than(
            assignment_precedence_group,
        ),
    );

    let macro_operator = VariadicOperator::collect(|left, right, env| {
        let define_parameter = group(left)
            .get_trait_if_present(TraitID::macro_parameter, env)?
            .ok_or_else(|| {
                ProgramError::new("Macro parameter must have the Macro-Parameter trait")
            })?;

        Ok(Value::new(Trait::r#macro(Macro {
            define_parameter,
            value_to_expand: group(right),
        })))
    });

    env.add_variadic_operator(&macro_operator, &function_precedence_group);
    env.variables.insert(
        String::from("=>"),
        Value::new(Trait::operator(Operator::Variadic(macro_operator.clone()))),
    );

    // Closures

    #[derive(Clone)]
    struct DefineClosureParameterFn(Rc<dyn Fn(Value, &mut Environment) -> Result<()>>);

    let closure_parameter_trait_id =
        TraitID::<DefineClosureParameterFn>::builtin("Closure-Parameter");

    // Name ::= Closure-Parameter
    env.add_conformance(Conformance::new(
        closure_parameter_trait_id.clone(),
        TraitID::name.validation(),
        |name, _| {
            let name = name.clone();

            Ok(DefineClosureParameterFn(Rc::new(move |input, env| {
                env.variables.insert(name.0.clone(), input);

                Ok(())
            })))
        },
    ));

    // TODO: Closure trait

    let closure_operator = VariadicOperator::collect(move |left, right, env| {
        let define_parameter = group(left)
            .get_trait_if_present(closure_parameter_trait_id.clone(), env)?
            .ok_or_else(|| {
                ProgramError::new("Closure parameter must have the Closure-Parameter trait")
            })?;

        let return_value = group(right);

        let closure_env = env.clone();

        Ok(Value::new(Trait::function(Function::new(
            move |input, _| {
                let mut closure_env = closure_env.clone();

                define_parameter.0(input, &mut closure_env)?;

                return_value.evaluate(&mut closure_env)
            },
        ))))
    });

    env.add_variadic_operator(&closure_operator, &function_precedence_group);
    env.variables.insert(
        String::from("->"),
        Value::new(Trait::operator(Operator::Variadic(closure_operator))),
    );

    // Math (temporary)

    // TODO: Implement using traits
    fn math(
        operation: impl Fn(BigRational, BigRational) -> BigRational + 'static,
    ) -> BinaryOperator {
        BinaryOperator::collect(move |left, right, env| {
            let left = left.evaluate(env)?.get_trait(TraitID::number, env)?;
            let right = right.evaluate(env)?.get_trait(TraitID::number, env)?;

            let result = Number(operation(left.0, right.0));

            Ok(Value::new(Trait::number(result)))
        })
    }

    let addition_precedence_group = env.precedence_group(
        Associativity::Left,
        PrecedenceGroupComparison::<BinaryPrecedenceGroup>::lowest(),
    );

    let addition_operator = math(std::ops::Add::add);
    env.add_binary_operator(&addition_operator, &addition_precedence_group);
    env.variables.insert(
        String::from("+"),
        Value::new(Trait::operator(Operator::Binary(addition_operator))),
    );

    let subtraction_operator = math(std::ops::Sub::sub);
    env.add_binary_operator(&subtraction_operator, &addition_precedence_group);
    env.variables.insert(
        String::from("-"),
        Value::new(Trait::operator(Operator::Binary(subtraction_operator))),
    );

    let multiplication_precedence_group = env.precedence_group(
        Associativity::Left,
        PrecedenceGroupComparison::<BinaryPrecedenceGroup>::higher_than(addition_precedence_group),
    );

    let multiplication_operator = math(std::ops::Mul::mul);
    env.add_binary_operator(&multiplication_operator, &multiplication_precedence_group);
    env.variables.insert(
        String::from("*"),
        Value::new(Trait::operator(Operator::Binary(multiplication_operator))),
    );

    let division_operator = math(std::ops::Div::div);
    env.add_binary_operator(&division_operator, &multiplication_precedence_group);
    env.variables.insert(
        String::from("/"),
        Value::new(Trait::operator(Operator::Binary(division_operator))),
    );
}

#[cfg(test)]
mod test {
    use crate::ProgramError;

    #[test]
    fn test_env() -> Result<(), ProgramError> {
        use crate::*;

        let mut env = Environment::default();
        init(&mut env);

        let block = Value::new(Trait::block(Block(Vec::new())));

        let result = block
            .evaluate(&mut env)?
            .get_trait(TraitID::text, &mut env)?;

        println!("{}", result.0);

        Ok(())
    }
}