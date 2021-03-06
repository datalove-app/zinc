pub mod arguments;

use arguments::{Arguments, Command};
use std::process::exit;
use structopt::StructOpt;

fn main() {
    let arguments: Arguments = Arguments::from_args();

    let result = match arguments.command {
        Command::GenKey(c) => c.execute(),
        Command::PubKey(c) => c.execute(),
        Command::Sign(c) => c.execute(),
    };

    if let Err(error) = result {
        eprintln!("{}", error);
        exit(1);
    }
}
