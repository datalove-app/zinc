use crate::{Error, IoToError};
use algebra::Bn254;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;
use zinc_bytecode::data::values::Value;
use zinc_bytecode::Program;
use zinc_vm::proving_systems::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "verify", about = "Verifies the proof using verifying key")]
pub struct VerifyCommand {
    #[structopt(short = "c", long = "circuit", help = "Compiled circuit program file")]
    pub circuit_path: PathBuf,

    #[structopt(
        short = "k",
        long = "verifying-key",
        help = "Path to verifying key file"
    )]
    pub key_path: PathBuf,

    #[structopt(
        short = "d",
        long = "public-data",
        help = "Path to public data JSON file"
    )]
    pub public_data_path: PathBuf,

    #[structopt(
        short = "s",
        long = "system",
        help = "Proving system",
        default_value = "groth16"
    )]
    pub system: String,

    #[structopt(
        short = "e",
        long = "engine",
        help = "Pairing engine",
        default_value = "bn254"
    )]
    pub engine: String,
}

impl VerifyCommand {
    pub fn execute(&self) -> Result<(), Error> {
        // Read program
        let bytes =
            fs::read(&self.circuit_path).error_with_path(|| self.circuit_path.to_string_lossy())?;
        let program = Program::from_bytes(bytes.as_slice()).map_err(Error::ProgramDecoding)?;

        // Read public input
        let output_text = fs::read_to_string(&self.public_data_path)
            .error_with_path(|| self.public_data_path.to_string_lossy())?;
        let output_value = serde_json::from_str(output_text.as_str())?;
        let output_struct = Value::from_typed_json(&output_value, &program.output)?;

        // Verify
        let verified = self.inner(&output_struct)?;

        if verified {
            println!("{}", "✔  Verified".bold().green());
        } else {
            println!("{}", "✘  Failed".bold().red());
            exit(1);
        }

        Ok(())
    }

    fn inner(&self, public_output: &Value) -> Result<bool, Error> {
        // Read proof
        let proof_bytes = read_hex(std::io::stdin(), "<stdin>", "proof")?;

        // Read verification key
        let key_file =
            fs::File::open(&self.key_path).error_with_path(|| self.key_path.to_string_lossy())?;
        let key_bytes = read_hex(
            key_file,
            &self.key_path.to_string_lossy(),
            "verification key",
        )?;

        match (self.system.as_str(), self.engine.as_str()) {
            ("groth16", "bn254") => {
                let proof = <Groth16 as ProvingSystem<Bn254>>::Proof::read(proof_bytes.as_slice())
                    .error_with_path(|| "<proof data>")?;
                let key = <Groth16 as ProvingSystem<Bn254>>::VerifyingKey::read(key_bytes.as_slice())
                    .error_with_path(|| self.key_path.to_string_lossy())?;
                Ok(zinc_vm::verify::<Groth16, Bn254>(&key, &proof, public_output)?)
            }
            _ => todo!(),
        }
    }
}

fn read_hex<R: std::io::Read>(
    mut reader: R,
    path_hint: &str,
    context_hint: &str,
) -> Result<Vec<u8>, Error> {
    let mut hex = String::new();
    reader
        .read_to_string(&mut hex)
        .error_with_path(|| path_hint)?;

    let bytes = hex::decode(hex.trim()).map_err(|error| Error::HexDecoding {
        context: context_hint.into(),
        error,
    })?;

    Ok(bytes)
}
