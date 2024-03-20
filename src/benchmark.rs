use crate::dpll::dpll;
use crate::dpll::schemas::{Formula, FormulaResultType, HeuristicType};
use crate::utils::plot_data;
use log::{debug, error, info};
use rayon::iter::ParallelBridge;
use rayon::iter::ParallelIterator;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;
use std::{fs, thread, time};

fn bench(
    path: &PathBuf,
    expected: &FormulaResultType,
    h: HeuristicType,
) -> (i32, i32, i32, Duration) {
    //info!(target: "benchmark", "Formula {:?}", path);

    let start = time::Instant::now();
    let mut formula = Formula::from_file(&path).unwrap();
    formula.heuristic_type = h;

    let timeout = Arc::new(AtomicBool::new(false));
    let timeout_copy = timeout.clone();
    let handle: JoinHandle<FormulaResultType> = thread::spawn(move || {
        dpll::dpll(&mut formula, timeout_copy);
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

pub fn benchmark() {
    let mut data: Vec<(HeuristicType, Vec<Duration>)> = Vec::new();
    let mut out_total_counter = 0;
    for heuristic in vec![HeuristicType::None, HeuristicType::VSIDS] {
        let paths = fs::read_dir("data/inputs").unwrap();

        let solved_counter = AtomicUsize::new(0);
        let timeout_counter = AtomicUsize::new(0);
        let error_counter = AtomicUsize::new(0);
        let total_counter = AtomicUsize::new(0);
        let solved_times: Arc<Mutex<Vec<Duration>>> = Arc::new(Mutex::new(Vec::new()));

        paths.for_each(|dir| {
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
