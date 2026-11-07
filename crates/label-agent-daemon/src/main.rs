//! Binary entrypoint for the `label-agent` autonomous appliance daemon.

mod app;
mod cli;

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match app::run_from_args(&args) {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("label-agent error: {err}");
            ExitCode::FAILURE
        }
    }
}
