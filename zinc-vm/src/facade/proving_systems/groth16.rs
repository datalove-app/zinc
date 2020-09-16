use super::{ProvingSystem, VerificationError};
use crate::Engine;
use groth16;
use r1cs_core::{ConstraintSynthesizer, SynthesisError};
use rand::Rng;

pub struct Groth16;

impl<E: Engine> ProvingSystem<E> for Groth16 {
    type Parameters = groth16::Parameters<E>;
    type Proof = groth16::Proof<E>;
    type PreparedVerifyingKey = groth16::PreparedVerifyingKey<E>;
    type VerifyingKey = groth16::VerifyingKey<E>;

    fn generate_random_parameters<C, R>(circuit: C, rng: &mut R) -> Result<Self::Parameters, SynthesisError>
    where
        C: ConstraintSynthesizer<E::Fr>,
        R: Rng
    {
        groth16::generate_random_parameters(circuit, rng)
    }

    fn create_random_proof<C, R>(
        circuit: C,
        params: &Self::Parameters,
        rng: &mut R
    ) -> Result<Self::Proof, SynthesisError>
    where
        C: ConstraintSynthesizer<E::Fr>,
        R: Rng
    {
        groth16::create_random_proof(circuit, params, rng)
    }

    fn prepare_verifying_key(vk: &Self::VerifyingKey) -> Self::PreparedVerifyingKey {
        groth16::prepare_verifying_key(vk)
    }

    fn verify_proof(
        pvk: &Self::PreparedVerifyingKey,
        proof: &Self::Proof,
        public_inputs: &[E::Fr]
    ) -> Result<bool, VerificationError> {
        groth16::verify_proof(pvk, proof, public_inputs)
            .map_err(VerificationError::SynthesisError)
    }
}
