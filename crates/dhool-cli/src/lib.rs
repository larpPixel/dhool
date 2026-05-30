mod args;
mod commands;
mod config;
mod render_loop;
mod sidecars;

use anyhow::Result;
use clap::Parser;

use args::{Cli, Command};

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::DeviceInfo => commands::print_device_info(),
        Command::Validate(args) => commands::validate(args),
        Command::Still(args) => commands::still(args),
        Command::Preview(args) => commands::movie("preview", args, true),
        Command::Final(args) => commands::movie("final", args, true),
        Command::Benchmark(args) => commands::benchmark(args),
    }
}
