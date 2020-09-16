mod groth16;

use super::VerificationError;
use crate::Engine;
use r1cs_core::{ConstraintSynthesizer, SynthesisError};
use rand::Rng;

pub trait ProvingSystem<E: Engine> {
    type Parameters;
    type Proof;
    type PreparedVerifyingKey;
    type VerifyingKey;

    fn generate_random_parameters<C, R>(circuit: C, rng: &mut R) -> Result<Self::Parameters, SynthesisError>
    where
        C: ConstraintSynthesizer<E::Fr>,
        R: Rng;

    fn create_random_proof<C, R>(
        circuit: C,
        params: &Self::Parameters,
        rng: &mut R
    ) -> Result<Self::Proof, SynthesisError>
    where
        C: ConstraintSynthesizer<E::Fr>,
        R: Rng;

    fn prepare_verifying_key(vk: &Self::VerifyingKey) -> Self::PreparedVerifyingKey;

    fn verify_proof(
        pvk: &Self::PreparedVerifyingKey,
        proof: &Self::Proof,
        public_inputs: &[E::Fr]
    ) -> Result<bool, VerificationError>;
}
