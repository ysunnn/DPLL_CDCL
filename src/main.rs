use crate::dpll::dpll;
use crate::schemas::{Formula, FormulaResultType};
use log::{error, info};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::{fs, thread, time};

mod dpll;
mod reader;
mod schemas;

fn test() {
    let start = time::Instant::now();
    let mut formula = Formula::from_file(&PathBuf::from("data/inputs/sat/par8-4.cnf")).unwrap();
    //dbg!(&formula);
    dpll(&mut formula, Arc::new(AtomicBool::new(false)));
    info!("Result: {:?}", formula.result);
    info!("Time: {:?}", start.elapsed());
}

/*fn benchmark() {
    let timout = Arc::new(AtomicBool::new(false));
    let paths = fs::read_dir("data/inputs").unwrap();

    let mut solved = 0;
    let mut timeout = 0;
    let mut error = 0;

    for dir in paths {
        let dir = dir.unwrap().path();
        if dir.file_name().unwrap() == "test" {
            continue;
        }
        let cdir = dir.file_name().unwrap();
        let excpexted = match cdir.to_str().unwrap() {
            "sat" => ResultType::Satisfiable,
            "unsat" => ResultType::Unsatisfiable,
            _ => panic!("Invalid dir name"),
        };
        for path in fs::read_dir(dir).unwrap() {
            let path = path.unwrap().path();

            info!(target: "benchmark", "Formula {:?}", path);

            let start = time::Instant::now();

            let mut formula = Formula::from_file(&path).unwrap();
            let timout_clone = timout.clone();
            let handle: JoinHandle<_> = thread::spawn(move || {
                dpll(&mut formula, timout_clone);
            });

            while start.elapsed().as_secs() < 60 {
                if handle.is_finished() {
                    info!(target: "benchmark", "thread finished");
                    break;
                }
            }

            if start.elapsed().as_secs() >= 60 {
                error!(target: "benchmark", "thread timeout");

                // Time limit exceeded, signal to terminate
                timout.store(true, Ordering::SeqCst);
            }

            // Wait for the thread to finish
            match handle.join() {
                Ok(_) => {
                    info!(target: "benchmark", "Result: {:?}", &formula.result);
                    if formula.result == excpexted {
                        solved += 1;
                    } else if formula.result == ResultType::Timeout {
                        timeout += 1;
                    } else {
                        error += 1;
                    }
                } // result is the return value from dpll
                Err(e) => info!(target: "benchmark", "Thread panicked: {:?}", e),
            }

            info!(target: "benchmark", "Time: {:?}", start.elapsed());
            timout.store(false, Ordering::SeqCst);
        }
    }

    info!(target: "benchmark", "Solved: {}", solved);
    info!(target: "benchmark", "Timeout: {}", timeout);
    info!(target: "benchmark", "Error: {}", error);
}*/

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
            info!("Result: {:?}", formula.result);
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
        //Some("benchmark") => benchmark(),
        _ => println!("Invalid argument. Please use 'test', 'tests', or 'benchmark'."),
    }
}
