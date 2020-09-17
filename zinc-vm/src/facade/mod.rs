pub mod proving_systems;
use proving_systems::*;

use crate::constraint_systems::{DebugConstraintSystem, DuplicateRemovingCS};
use crate::core::VirtualMachine;
pub use crate::errors::{MalformedBytecode, Result, RuntimeError, TypeSizeError};
use crate::gadgets::utils::bigint_to_fr;
use crate::Engine;
use failure::Fail;
use num_bigint::BigInt;
use r1cs_core::{ConstraintSynthesizer, ConstraintSystem, Namespace, SynthesisError};
use r1cs_std::test_constraint_system::TestConstraintSystem;
use std::{fmt::Debug, marker::PhantomData};
use zinc_bytecode::{data::values::Value, program::Program};

struct VMCircuit<'a, E: Engine> {
    program: &'a Program,
    inputs: Option<&'a [BigInt]>,
    result: &'a mut Option<Result<Vec<Option<BigInt>>>>,
    engine: PhantomData<E>,
}

impl<E: Engine> ConstraintSynthesizer<E::Fr> for VMCircuit<'_, E> {
    fn generate_constraints<CS: ConstraintSystem<E::Fr>>(
        self,
        cs: &mut CS,
    ) -> std::result::Result<(), SynthesisError> {
        // let cs = LoggingConstraintSystem::new(cs.ns(|| "logging"));
        let cs = DuplicateRemovingCS::<E, Namespace<'_, E::Fr, CS::Root>>::new(
            cs.ns(|| "duplicates removing"),
        );
        let mut vm =
            VirtualMachine::<E, DuplicateRemovingCS<E, Namespace<'_, E::Fr, CS::Root>>>::new(
                cs, false,
            );
        *self.result = Some(vm.run(self.program, self.inputs, |_| {}, |_| Ok(())));
        Ok(())
    }
}

pub fn run<E: Engine>(program: &Program, inputs: &Value) -> Result<Value> {
    let cs = DebugConstraintSystem::<E>::default();
    let mut vm = VirtualMachine::<E, DebugConstraintSystem<E>>::new(cs, true);

    let inputs_flat = inputs.to_flat_values();

    let mut num_constraints = 0;
    let result = vm.run(
        program,
        Some(&inputs_flat),
        |cs| {
            let num = cs.num_constraints() - num_constraints;
            num_constraints += num;
            log::debug!("Constraints: {}", num);
        },
        |cs| {
            if !cs.is_satisfied() {
                return Err(RuntimeError::UnsatisfiedConstraint);
            }

            Ok(())
        },
    )?;

    let cs = vm.constraint_system();
    if !cs.is_satisfied() {
        return Err(RuntimeError::UnsatisfiedConstraint);
    }

    let output_flat = result
        .into_iter()
        .map(|v| v.expect("`run` always computes witness"))
        .collect::<Vec<_>>();

    let value = Value::from_flat_values(&program.output, &output_flat).ok_or_else(|| {
        TypeSizeError::Output {
            expected: 0,
            actual: 0,
        }
    })?;

    Ok(value)
}

pub fn debug<E: Engine>(program: &Program, inputs: &Value) -> Result<Value> {
    let cs = TestConstraintSystem::<E::Fr>::new();
    let mut vm = VirtualMachine::<E, TestConstraintSystem<E::Fr>>::new(cs, true);

    let inputs_flat = inputs.to_flat_values();

    let mut num_constraints = 0;
    let result = vm.run(
        program,
        Some(&inputs_flat),
        |cs| {
            let num = cs.num_constraints() - num_constraints;
            num_constraints += num;
            log::debug!("Constraints: {}", num);
        },
        |cs| {
            if !cs.is_satisfied() {
                return Err(RuntimeError::UnsatisfiedConstraint);
            }

            Ok(())
        },
    )?;

    let cs = vm.constraint_system();

    // FIXME
    // log::trace!("{}", cs.pretty_print());

    if !cs.is_satisfied() {
        log::error!("unsatisfied: {}", cs.which_is_unsatisfied().unwrap());
        return Err(RuntimeError::UnsatisfiedConstraint);
    }

    // FIXME
    // let unconstrained = cs.find_unconstrained();
    // if !unconstrained.is_empty() {
    //     log::error!("Unconstrained: {}", unconstrained);
    //     return Err(RuntimeError::InternalError(
    //         "Generated unconstrained variables".into(),
    //     ));
    // }

    let output_flat = result
        .into_iter()
        .map(|v| v.expect("`run` always computes witness"))
        .collect::<Vec<_>>();

    let value = Value::from_flat_values(&program.output, &output_flat).ok_or_else(|| {
        TypeSizeError::Output {
            expected: 0,
            actual: 0,
        }
    })?;

    Ok(value)
}

pub fn setup<PS: ProvingSystem<E>, E: Engine>(program: &Program) -> Result<PS::Parameters> {
    let rng = &mut rand::thread_rng();
    let mut result = None;
    let circuit: VMCircuit<'_, E> = VMCircuit {
        program,
        inputs: None,
        result: &mut result,
        engine: PhantomData,
    };

    let params = PS::generate_random_parameters(circuit, rng)?;

    match result.expect("vm should return either output or error") {
        Ok(_) => Ok(params),
        Err(error) => Err(error),
    }
}

pub fn prove<PS: ProvingSystem<E>, E: Engine>(
    program: &Program,
    params: &PS::Parameters,
    witness: &Value,
) -> Result<(Value, PS::Proof)> {
    let rng = &mut rand::thread_rng();

    let witness_flat = witness.to_flat_values();

    let (result, proof) = {
        let mut result = None;
        let circuit: VMCircuit<'_, E> = VMCircuit {
            program,
            inputs: Some(&witness_flat),
            result: &mut result,
            engine: PhantomData,
        };

        let proof =
            PS::create_random_proof(circuit, params, rng).map_err(RuntimeError::SynthesisError)?;

        (result, proof)
    };

    match result {
        None => Err(RuntimeError::InternalError(
            "circuit hasn't generate outputs".into(),
        )),
        Some(res) => match res {
            Ok(values) => {
                let output_flat: Vec<BigInt> = values
                    .into_iter()
                    .map(|v| v.expect("`prove` always computes witness"))
                    .collect();

                let value =
                    Value::from_flat_values(&program.output, &output_flat).ok_or_else(|| {
                        TypeSizeError::Output {
                            expected: 0,
                            actual: 0,
                        }
                    })?;

                Ok((value, proof))
            }
            Err(err) => Err(err),
        },
    }
}

#[derive(Debug, Fail)]
pub enum VerificationError {
    #[fail(display = "value overflow: value {} is not in the field", _0)]
    ValueOverflow(BigInt),

    #[fail(display = "failed to synthesize circuit: {}", _0)]
    SynthesisError(SynthesisError),
}

pub fn verify<PS: ProvingSystem<E>, E: Engine>(
    key: &PS::VerifyingKey,
    proof: &PS::Proof,
    public_input: &Value,
) -> std::result::Result<bool, VerificationError> {
    let public_input_flat = public_input
        .to_flat_values()
        .into_iter()
        .map(|value| {
            bigint_to_fr::<E::Fr>(&value).ok_or_else(|| VerificationError::ValueOverflow(value))
        })
        .collect::<std::result::Result<Vec<E::Fr>, VerificationError>>()?;

    let pvk = PS::prepare_verifying_key(&key);
    let success = PS::verify_proof(&pvk, proof, public_input_flat.as_slice())?;

    Ok(success)
}
