//!
//! The Zinc compiler library.
//!

mod bytecode;
mod error;
mod generator;
mod lexical;
mod semantic;
mod syntax;

pub use self::bytecode::Bytecode;
pub use self::error::Error;
pub use self::semantic::EntryPointAnalyzer;
pub use self::semantic::ModuleAnalyzer;
pub use self::semantic::Scope;
pub use self::syntax::Parser;
pub use self::syntax::Tree;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::RwLock;

use lazy_static::lazy_static;

use crate::generator::Representation;

use self::error::file::Error as FileError;

pub const BASE_BINARY: usize = 2;
pub const BASE_OCTAL: usize = 8;
pub const BASE_DECIMAL: usize = 10;
pub const BASE_HEXADECIMAL: usize = 16;

pub const BITLENGTH_BOOLEAN: usize = 1;
pub const BITLENGTH_BYTE: usize = 8;
pub const BITLENGTH_INDEX: usize = 64;
pub const BITLENGTH_MAX_INT: usize = 248;
pub const BITLENGTH_FIELD: usize = 254;

pub const SHA256_HASH_SIZE_BITS: usize = 256;
pub const PEDERSEN_HASH_INPUT_LIMIT_BITS: usize = 512;
pub const SCHNORR_MESSAGE_LIMIT_BYTES: usize = 31;
pub const SCHNORR_MESSAGE_LIMIT_BITS: usize = SCHNORR_MESSAGE_LIMIT_BYTES * BITLENGTH_BYTE;

pub static PANIC_LAST_SHARED_REFERENCE: &str = "There are no other references at this point";
pub static PANIC_MUTEX_SYNC: &str = "Mutexes never panic";
pub static PANIC_FILE_INDEX: &str = "File record always exists";
pub static PANIC_BUILDER_REQUIRES_VALUE: &str = "The builder requires a value: ";

lazy_static! {
    static ref FILE_INDEX: RwLock<HashMap<usize, PathBuf>> = RwLock::new(HashMap::new());
}

#[allow(clippy::implicit_hasher)]
pub fn compile_entry(
    path: PathBuf,
    dependencies: HashMap<String, Rc<RefCell<Scope>>>,
) -> Result<Representation, String> {
    let code = read(&path)?;
    let lines = code.lines().collect::<Vec<&str>>();

    let next_file_id = FILE_INDEX.read().expect(PANIC_MUTEX_SYNC).len();
    FILE_INDEX
        .write()
        .expect(PANIC_MUTEX_SYNC)
        .insert(next_file_id, path);

    let syntax_tree = Parser::default()
        .parse(&code, Some(next_file_id))
        .map_err(|error| error.format(&lines))?;

    EntryPointAnalyzer::new()
        .compile(syntax_tree, dependencies)
        .map_err(|error| error.format(&lines))
}

pub fn compile_module(path: PathBuf) -> Result<(Rc<RefCell<Scope>>, Representation), String> {
    let code = read(&path)?;
    let lines = code.lines().collect::<Vec<&str>>();

    let next_file_id = FILE_INDEX.read().expect(PANIC_MUTEX_SYNC).len();
    FILE_INDEX
        .write()
        .expect(PANIC_MUTEX_SYNC)
        .insert(next_file_id, path);

    let syntax_tree = Parser::default()
        .parse(&code, Some(next_file_id))
        .map_err(|error| error.format(&lines))?;

    ModuleAnalyzer::new()
        .compile(syntax_tree)
        .map_err(|error| error.format(&lines))
}

pub fn compile_test(code: &str) -> Result<Bytecode, String> {
    let lines = code.lines().collect::<Vec<&str>>();

    let syntax_tree = Parser::default()
        .parse(code, None)
        .map_err(|error| error.format(&lines))?;
    let bytecode = Rc::new(RefCell::new(Bytecode::new()));

    EntryPointAnalyzer::new()
        .compile(syntax_tree, HashMap::new())
        .map_err(|error| error.format(&lines))?;

    Ok(Rc::try_unwrap(bytecode)
        .expect(PANIC_LAST_SHARED_REFERENCE)
        .into_inner())
}

pub fn read(path: &PathBuf) -> Result<String, String> {
    let mut file = File::open(path)
        .map_err(FileError::Opening)
        .map_err(|error| error.to_string())?;
    let size = file
        .metadata()
        .map_err(FileError::Metadata)
        .map_err(|error| error.to_string())?
        .len() as usize;
    let mut code = String::with_capacity(size);
    file.read_to_string(&mut code)
        .map_err(FileError::Reading)
        .map_err(|error| error.to_string())?;
    Ok(code)
}
