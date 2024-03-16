use crate::benchmark::benchmark;
use crate::dpll::dpll as run_dpll;
use crate::dpll::schemas::{Formula, HeuristicType};
use crate::tests::{test, tests};
use clap::{Parser, Subcommand};
use log::info;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time;

mod benchmark;
mod dpll;
mod tests;
mod utils;

#[derive(Parser, Debug)]
pub struct CommandLineArgs {
    pub command: Option<String>,
    pub file: Option<PathBuf>,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Debug, Subcommand)]
enum Commands {
    /// run the test function
    Test,
    /// run the tests on the given directory
    Tests,
    /// runs the benchmark on the given directory, uses all of your cpu power
    Benchmark,
    /// solve the given cnf file
    Solve {
        /// The file to run
        file: PathBuf,
        /// The heuristic to use
        #[arg(value_enum)]
        heuristic: Option<HeuristicType>,
    },
}

fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    env_logger::init();

    let args = Cli::parse();

    match args.command {
        Commands::Test => test(),
        Commands::Tests => tests(),
        Commands::Benchmark => benchmark(),
        Commands::Solve { file, heuristic } => {
            let start = time::Instant::now();
            let mut formula = Formula::from_file(&file).unwrap();
            formula.heuristic_type = heuristic.unwrap_or(HeuristicType::None);
            formula.update_score();
            run_dpll::dpll(&mut formula, Arc::new(AtomicBool::new(false)));
            info!("solved in {:?}", start.elapsed());
            println!("{}", formula.write_solution());
        }
    }
}
