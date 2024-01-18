use crate::dpll::dpll;
use crate::schemas::{Formula, FormulaResultType, HeuristicType};
use clap::{Parser, Subcommand};
use log::{debug, error, info};
use plotters::element::PathElement;
use plotters::prelude::{
    BitMapBackend, ChartBuilder, Color, IntoDrawingArea, IntoFont, LineSeries, BLACK, BLUE, CYAN,
    GREEN, MAGENTA, RED, WHITE,
};
use rayon::iter::ParallelIterator;
use rayon::prelude::ParallelBridge;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;
use std::{fs, thread, time};

mod dimacs_converter;
mod dpll;
mod schemas;

fn test() {
    let start = time::Instant::now();
    let path = PathBuf::from("data/inputs/sat\\aim-100-2_0-yes1-2.cnf");
    info!("Formula {:?}", path);
    let mut formula = Formula::from_file(&PathBuf::from(path)).unwrap();
    formula.heuristic_type=HeuristicType::VSIDS;
    formula.jeroslow_wang_score();
    dpll(&mut formula, Arc::new(AtomicBool::new(false)));

    for clause in formula.clauses.iter() {
        info!("Clause: {:?}", clause);
    }

    for variable in formula.variables.iter() {
        info!("Variable: {:?}", variable);
    }

    info!("Result: {:?}", formula.result);
    info!("Time: {:?}", start.elapsed());
}

fn plot_data(
    data: &Vec<(HeuristicType, Vec<Duration>)>,
    num_of_problems: i32,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(name, (1920, 1080)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Cactus Plot", ("sans-serif", 50).into_font())
        .margin(20)
        .x_label_area_size(80)
        .y_label_area_size(80)
        .build_cartesian_2d(0i32..num_of_problems, 0i32..60000)
        .unwrap();

    chart
        .configure_mesh()
        .x_desc("Solved Problems")
        .y_desc("Time in miliseconds")
        .label_style(("sans-serif", 20).into_font())
        .draw()?;

    for (heuristic, times) in data {
        let mut current_data = Vec::new();
        for mil_sec in 1..60000 {
            let sec = Duration::from_millis(mil_sec);
            let count = times.iter().filter(|&time| time <= &sec).count();
            current_data.push((count as i32, sec));
        }

        let color = match heuristic {
            HeuristicType::DLIS => RED,
            HeuristicType::DLCS => GREEN,
            HeuristicType::MOM => BLUE,
            HeuristicType::JeroslowWang => MAGENTA,
            HeuristicType::VSIDS => CYAN,
            HeuristicType::None => BLACK,
        };

        let name = match heuristic {
            HeuristicType::DLIS => "DLIS",
            HeuristicType::DLCS => "DLCS",
            HeuristicType::MOM => "MOM",
            HeuristicType::JeroslowWang => "JeroslowWang",
            HeuristicType::VSIDS => "VSIDS",
            HeuristicType::None => "None",
        };

        chart
            .draw_series(
                LineSeries::new(
                    current_data.iter().map(|(x, y)| (*x, y.as_millis() as i32)),
                    &color,
                )
                .point_size(2),
            )
            .unwrap()
            .label(name)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.clone()));
    }
    chart
        .configure_series_labels()
        .border_style(&BLACK)
        .background_style(&WHITE.mix(0.8))
        .label_font(("sans-serif", 20).into_font())
        .draw()
        .unwrap();

    Ok(())
}

fn bench(
    path: &PathBuf,
    expected: &FormulaResultType,
    h: HeuristicType,
) -> (i32, i32, i32, Duration) {
    //info!(target: "benchmark", "Formula {:?}", path);

    let start = time::Instant::now();
    let mut formula = Formula::from_file(&path).unwrap();

    match h {
        HeuristicType::DLIS => formula.dlis(),
        HeuristicType::DLCS => formula.dlcs(),
        HeuristicType::MOM => formula.mom(),
        HeuristicType::JeroslowWang => formula.jeroslow_wang_score(),
        HeuristicType::VSIDS => {
            formula.heuristic_type = HeuristicType::VSIDS;
            formula.jeroslow_wang_score()
        }
        HeuristicType::None => {}
    }

    let timeout = Arc::new(AtomicBool::new(false));
    let timeout_copy = timeout.clone();
    let handle: JoinHandle<FormulaResultType> = thread::spawn(move || {
        dpll(&mut formula, timeout_copy);
        formula.result
    });

    while start.elapsed().as_secs() < 60 {
        if handle.is_finished() {
            //info!(target: "benchmark", "thread finished");
            break;
        }
    }

    if start.elapsed().as_secs() >= 60 {
        //error!(target: "benchmark", "thread timeout");

        // Time limit exceeded, signal to terminate
        timeout.store(true, Ordering::SeqCst);
    }

    // Wait for the thread to finish
    match handle.join() {
        Ok(result) => {
            let elapsed = start.elapsed();
            debug!(target: "benchmark", "Time: {:?}", elapsed);
            return if result == *expected {
                debug!(target: "benchmark", "Right result: {:?} Formula {:?}", result, path);
                (1, 0, 0, elapsed)
            } else if result == FormulaResultType::Timeout {
                debug!(target: "benchmark", "Timeout result: {:?} Formula {:?}", result, path);
                (0, 1, 0, elapsed)
            } else {
                error!(target: "benchmark", "Wrong result: {:?} Formula {:?}", result, path);
                (0, 0, 1, elapsed)
            };
        } // result is the return value from dpll
        Err(e) => panic!("Thread panicked: {:?} Formula {:?}", e, path),
    }
}

fn benchmark() {
    let mut data: Vec<(HeuristicType, Vec<Duration>)> = Vec::new();
    let mut out_total_counter = 0;
    for heuristic in vec![
        HeuristicType::None,
        HeuristicType::DLIS,
        HeuristicType::DLCS,
        HeuristicType::MOM,
        HeuristicType::JeroslowWang,
        HeuristicType::VSIDS,
    ] {
        let paths = fs::read_dir("data/inputs").unwrap();

        let solved_counter = AtomicUsize::new(0);
        let timeout_counter = AtomicUsize::new(0);
        let error_counter = AtomicUsize::new(0);
        let total_counter = AtomicUsize::new(0);
        let solved_times: Arc<Mutex<Vec<Duration>>> = Arc::new(Mutex::new(Vec::new()));

        paths.par_bridge().for_each(|dir| {
            let dir = dir.unwrap().path();
            if dir.file_name().unwrap() == "test" {
                return;
            }
            let cdir = dir.file_name().unwrap();
            let expected = match cdir.to_str().unwrap() {
                "sat" => FormulaResultType::Satisfiable,
                "unsat" => FormulaResultType::Unsatisfiable,
                _ => panic!("Invalid dir name"),
            };
            fs::read_dir(dir).unwrap().par_bridge().for_each(|path| {
                let path = path.unwrap().path();
                total_counter.fetch_add(1, Ordering::SeqCst);
                let (solved, timeout, error, time) = bench(&path, &expected, heuristic);
                if solved == 1 {
                    let mut data = solved_times.lock().unwrap();
                    data.push(time);
                }
                solved_counter.fetch_add(solved as usize, Ordering::SeqCst);
                timeout_counter.fetch_add(timeout as usize, Ordering::SeqCst);
                error_counter.fetch_add(error as usize, Ordering::SeqCst);
            });
        });

        info!(target: "benchmark", "Heuristic: {:?}", heuristic);
        info!(target: "benchmark", "Solved: {}", solved_counter.load(Ordering::SeqCst));
        info!(target: "benchmark", "Timeout: {}", timeout_counter.load(Ordering::SeqCst));
        info!(target: "benchmark", "Error: {}", error_counter.load(Ordering::SeqCst));
        info!(target: "benchmark", "Total: {}", total_counter.load(Ordering::SeqCst));
        info!(target: "benchmark", "Solved: {}%", solved_counter.load(Ordering::SeqCst) as f64 / total_counter.load(Ordering::SeqCst) as f64 * 100.0);

        let d = solved_times.lock().unwrap().clone();

        out_total_counter = total_counter.load(Ordering::SeqCst) as i32;
        data.push((heuristic, d.clone()));
    }

    //data.sort();
    plot_data(&data, out_total_counter, "cactus_plot.png").unwrap();
}

fn tests() {
    let dirs = fs::read_dir("data/inputs/test").unwrap();
    let mut times: Vec<Duration> = Vec::new();
    for dir in dirs {
        let dir = dir.unwrap().path();
        let cdir = dir.file_name().unwrap();
        let excpexted = match cdir.to_str().unwrap() {
            "sat" => FormulaResultType::Satisfiable,
            "unsat" => FormulaResultType::Unsatisfiable,
            _ => {
                continue;
            }
        };
        for path in fs::read_dir(dir).unwrap() {
            let path = path.unwrap().path();
            info!("Formula {:?}", path);
            let start = time::Instant::now();
            let mut formula = Formula::from_file(&path).unwrap();
            dpll(&mut formula, Arc::new(AtomicBool::new(false)));
            info!("Result: {}", Formula::write_solution(&formula));
            assert_eq!(formula.result, excpexted);
            let time = start.elapsed();
            info!("Time: {:?}", time);
            times.push(time);
        }
    }

    let mut data = Vec::new();
    data.push((HeuristicType::None, times.clone()));

    plot_data(&data, 30, "cactus_test_plot.png").unwrap();
}

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
            match heuristic {
                Some(heuristic) => match heuristic {
                    HeuristicType::DLIS => formula.dlis(),
                    HeuristicType::DLCS => formula.dlcs(),
                    HeuristicType::MOM => formula.mom(),
                    HeuristicType::JeroslowWang => formula.jeroslow_wang_score(),
                    HeuristicType::VSIDS => {
                        formula.heuristic_type = HeuristicType::VSIDS;
                        formula.jeroslow_wang_score()
                    }
                    HeuristicType::None => {}
                },
                None => {}
            }
            dpll(&mut formula, Arc::new(AtomicBool::new(false)));
            info!("solved in {:?}", start.elapsed());
            println!("{}", formula.write_solution());
        }
    }
}
