mod run;

use colored::Colorize;
use run::*;
use std::process::exit;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "The Wipple interpreter", bin_name = "wipple", no_version)]
pub enum Args {
    Run(Run),
}

fn main() {
    exit(run())
}

fn run() -> i32 {
    let args = Args::from_args();

    match args {
        Args::Run(run) => {
            if let Err(state) = run.run() {
                eprintln!(
                    "{}",
                    state.into_error(&wipple::Stack::new()).to_string().red()
                );
                return 1;
            }
        }
    }

    0
}
