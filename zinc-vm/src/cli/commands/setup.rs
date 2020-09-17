use crate::{Error, IoToError};
use algebra::Bn254;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;
use zinc_bytecode::program::Program;
use zinc_vm::proving_systems::*;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "setup",
    about = "Generates a pair of proving and verifying keys"
)]
pub struct SetupCommand {
    #[structopt(short = "c", long = "circuit", help = "Circuit's bytecode file")]
    pub circuit_path: PathBuf,

    #[structopt(short = "p", long = "proving-key", help = "Params file to write")]
    pub proving_key_path: PathBuf,

    #[structopt(short = "v", long = "verifying-key", help = "Params file to write")]
    pub verifying_key_path: PathBuf,

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

impl SetupCommand {
    pub fn execute(&self) -> Result<(), Error> {
        let bytes =
            fs::read(&self.circuit_path).error_with_path(|| self.circuit_path.to_string_lossy())?;
        let program = Program::from_bytes(bytes.as_slice()).map_err(Error::ProgramDecoding)?;

        let vk_hex = self.inner(&program)?;
        fs::write(&self.verifying_key_path, vk_hex)
            .error_with_path(|| self.verifying_key_path.to_string_lossy())?;

        Ok(())
    }

    fn inner(&self, program: &Program) -> Result<String, Error> {
        let pkey_file = fs::File::create(&self.proving_key_path)
            .error_with_path(|| self.proving_key_path.to_string_lossy())?;

        let params = match (self.system.as_str(), self.engine.as_str()) {
            ("groth16", "bn254") => zinc_vm::setup::<Groth16, Bn254>(program)?,
            _ => todo!(),
        };

        params
            .write(pkey_file)
            .error_with_path(|| self.proving_key_path.to_string_lossy())?;

        let mut vk_bytes = Vec::new();
        params.write_verifying_key(&mut vk_bytes).expect("writing to vec");

        Ok(hex::encode(vk_bytes) + "\n")
    }
}
