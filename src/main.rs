use crate::dpll::dpll;
use log::{error, info, warn};
use std::path::{PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use crate::schemas::{Formula, FormulaResultType};
use std::sync::Arc;
use std::{fs, thread, time};
use std::thread::JoinHandle;
use rayon::prelude::ParallelBridge;
use rayon::iter::ParallelIterator;

mod dpll;
mod dimacs_converter;
mod schemas;

fn test() {
    let start = time::Instant::now();
    let mut formula = Formula::from_file(&PathBuf::from("data/inputs/unsat/aim-100-2_0-no-4.cnf")).unwrap();
    let result = dpll(&mut formula, Arc::new(AtomicBool::new(false)));

    for clause in formula.clauses.iter() {
        if !clause.satisfiable {
            error!("Unsatisfiable clause: {:?}", clause);
        }
        info!("Clause: {:?}", clause);
    }

    info!("Result: {:?}", result);
    info!("Time: {:?}", start.elapsed());
}

fn bench(path: &PathBuf, expected: &FormulaResultType) -> (i32, i32, i32) {
    //info!(target: "benchmark", "Formula {:?}", path);

    let start = time::Instant::now();
    let mut formula = Formula::from_file(&path).unwrap();
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
            info!(target: "benchmark", "Time: {:?}", start.elapsed());
            return if result == *expected {
                info!(target: "benchmark", "Right result: {:?} Formula {:?}", result, path);
                (1, 0, 0)
            } else if result == FormulaResultType::Timeout {
                warn!(target: "benchmark", "Timeout result: {:?} Formula {:?}", result, path);
                (0, 1, 0)
            } else {
                error!(target: "benchmark", "Wrong result: {:?} Formula {:?}", result, path);
                (0, 0, 1)
            }
        } // result is the return value from dpll
        Err(e) => panic!("Thread panicked: {:?} Formula {:?}", e, path),
    }
}

fn benchmark() {
    let paths = fs::read_dir("data/inputs").unwrap();

    let  solved_counter = AtomicUsize::new(0);
    let  timeout_counter =AtomicUsize::new(0);
    let  error_counter = AtomicUsize::new(0);
    let  total_counter = AtomicUsize::new(0);

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
        fs::read_dir(dir).unwrap().par_bridge().for_each(|path|{
            let path = path.unwrap().path();
            total_counter.fetch_add(1, Ordering::SeqCst);
            let (solved, timeout, error) = bench(&path, &expected);
            solved_counter.fetch_add(solved as usize, Ordering::SeqCst);
            timeout_counter.fetch_add(timeout as usize, Ordering::SeqCst);
            error_counter.fetch_add(error as usize, Ordering::SeqCst);
        });
    });

    info!(target: "benchmark", "Solved: {}", solved_counter.load(Ordering::SeqCst));
    info!(target: "benchmark", "Timeout: {}", timeout_counter.load(Ordering::SeqCst));
    info!(target: "benchmark", "Error: {}", error_counter.load(Ordering::SeqCst));
    info!(target: "benchmark", "Total: {}", total_counter.load(Ordering::SeqCst));
    info!(target: "benchmark", "Solved: {}%", solved_counter.load(Ordering::SeqCst) as f64 / total_counter.load(Ordering::SeqCst) as f64 * 100.0);
}

fn tests() {
    let dirs = fs::read_dir("data/inputs/test").unwrap();
    for dir in dirs {
        let dir = dir.unwrap().path();
        let cdir = dir.file_name().unwrap();
        let excpexted = match cdir.to_str().unwrap() {
            "sat" => FormulaResultType::Satisfiable,
            "unsat" => FormulaResultType::Unsatisfiable,
            _ => { continue; }
        };
        for path in fs::read_dir(dir).unwrap() {
            let path = path.unwrap().path();
            info!("Formula {:?}", path);
            let start = time::Instant::now();
            let mut formula = Formula::from_file(&path).unwrap();
            dpll(&mut formula, Arc::new(AtomicBool::new(false)));
            info!("Result: {}", Formula::write_solution(&formula));
            assert_eq!(formula.result, excpexted);
            info!("Time: {:?}", start.elapsed());
        }
    }
}

fn main() {
    #[cfg(feature = "dhat-heap")]
        let _profiler = dhat::Profiler::new_heap();
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("test") => test(),
        Some("tests") => tests(),
        Some("benchmark") => benchmark(),
        _ => println!("Invalid argument. Please use 'test', 'tests', or 'benchmark'."),
    }
}
// sat sollte aber unsat sein aim-100-1_6-no-2.cnf, aim-100-2_0-no-4.cnf