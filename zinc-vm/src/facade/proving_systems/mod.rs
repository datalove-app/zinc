mod groth16;
pub use self::groth16::Groth16;

use super::VerificationError;
use crate::Engine;
use algebra::{CanonicalDeserialize, CanonicalSerialize};
use r1cs_core::{ConstraintSynthesizer, SynthesisError};
use rand::Rng;
use std::io;

pub trait ProvingSystem<E: Engine> {
    type Parameters: Parameters<VerifyingKey = Self::VerifyingKey>;
    type Proof: Proof;
    type PreparedVerifyingKey;
    type VerifyingKey: VerifyingKey;

    fn generate_random_parameters<C, R>(
        circuit: C,
        rng: &mut R,
    ) -> Result<Self::Parameters, SynthesisError>
    where
        C: ConstraintSynthesizer<E::Fr>,
        R: Rng;

    fn create_random_proof<C, R>(
        circuit: C,
        params: &Self::Parameters,
        rng: &mut R,
    ) -> Result<Self::Proof, SynthesisError>
    where
        C: ConstraintSynthesizer<E::Fr>,
        R: Rng;

    fn prepare_verifying_key(vk: &Self::VerifyingKey) -> Self::PreparedVerifyingKey;

    fn verify_proof(
        pvk: &Self::PreparedVerifyingKey,
        proof: &Self::Proof,
        public_inputs: &[E::Fr],
    ) -> Result<bool, VerificationError>;
}

/// Full public (prover and verifier) parameters for a zkSNARK.
pub trait Parameters: CanonicalSerialize + CanonicalDeserialize {
    type VerifyingKey: VerifyingKey;

    fn read<R: io::Read>(reader: R) -> io::Result<Self>;

    fn write<W: io::Write>(&self, writer: W) -> io::Result<()>;

    fn read_verifying_key<R: io::Read>(reader: R) -> io::Result<Self::VerifyingKey>;

    fn write_verifying_key<W: io::Write>(&self, writer: W) -> io::Result<()>;
}

/// A proof in the a SNARK.
pub trait Proof: CanonicalSerialize + CanonicalDeserialize {
    fn read<R: io::Read>(reader: R) -> io::Result<Self>;

    fn write<W: io::Write>(&self, writer: W) -> io::Result<()>;
}

/// A verification key for a zkSNARK.
pub trait VerifyingKey: CanonicalSerialize + CanonicalDeserialize {
    fn read<R: io::Read>(reader: R) -> io::Result<Self>;

    fn write<W: io::Write>(&self, writer: W) -> io::Result<()>;
}
