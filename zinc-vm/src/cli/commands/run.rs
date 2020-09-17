use crate::{Error, IoToError};
use algebra::Bn254;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;
use zinc_bytecode::data::values::Value;
use zinc_bytecode::program::Program;

#[derive(Debug, StructOpt)]
#[structopt(name = "run", about = "Executes circuit and prints program's output")]
pub struct RunCommand {
    #[structopt(short = "c", long = "circuit", help = "Circuit's bytecode file")]
    pub circuit_path: PathBuf,

    #[structopt(short = "i", long = "input", help = "Program's input file")]
    pub input_path: PathBuf,

    #[structopt(short = "o", long = "output", help = "Program's output file")]
    pub output_path: PathBuf,

    #[structopt(
        short = "e",
        long = "engine",
        help = "Pairing engine",
        default_value = "bn254"
    )]
    pub engine: String,
}

impl RunCommand {
    pub fn execute(&self) -> Result<(), Error> {
        let bytes =
            fs::read(&self.circuit_path).error_with_path(|| self.circuit_path.to_string_lossy())?;
        let program = Program::from_bytes(bytes.as_slice()).map_err(Error::ProgramDecoding)?;

        let input_text = fs::read_to_string(&self.input_path)
            .error_with_path(|| self.input_path.to_string_lossy())?;
        let json = serde_json::from_str(&input_text)?;
        let input = Value::from_typed_json(&json, &program.input)?;

        let output = self.inner(&program, &input)?;

        let output_json = serde_json::to_string_pretty(&output.to_json())? + "\n";
        fs::write(&self.output_path, &output_json)
            .error_with_path(|| self.output_path.to_string_lossy())?;

        print!("{}", output_json);

        Ok(())
    }

    fn inner(&self, program: &Program, input: &Value) -> Result<Value, Error> {
        match self.engine.as_str() {
            "bn254" => Ok(zinc_vm::run::<Bn254>(program, input)?),
            _ => todo!(),
        }
    }
}
