use crate::dpll::dpll;
use log::{error, info, warn};
use std::path::{PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use crate::schemas::{Formula, FormulaResultType};
use std::sync::{Arc, Mutex};
use std::{fs, thread, time};
use std::thread::JoinHandle;
use std::time::Duration;
use plotters::prelude::{BitMapBackend, ChartBuilder, IntoDrawingArea, IntoFont, LineSeries, RED, WHITE};
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

    for variable in formula.variables.iter() {
        info!("Variable: {:?}", variable);
    }

    info!("Result: {:?}", result);
    info!("Time: {:?}", start.elapsed());
}

fn plot_data(times: &Vec<Duration>, num_of_problems: i32, name: &str) -> Result<(), Box<dyn std::error::Error>> {

    let mut data = Vec::new();

    for sec in 1..60 {
        let sec = Duration::from_secs(sec);
        let count = times.iter().filter(|&time| time <= &sec).count();
        data.push((count as i32, sec));
    }


    let root = BitMapBackend::new(name, (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Cactus Plot", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0i32..num_of_problems, 0i32..60)?;

    chart.configure_mesh()
        .x_desc("Solved Problems")
        .y_desc("Time in seconds")
        .draw()?;

    chart.draw_series(LineSeries::new(
        data.iter().map(|(x, y)| (*x, y.as_secs() as i32)),
        &RED,
    ))?;

    Ok(())
}

fn bench(path: &PathBuf, expected: &FormulaResultType) -> (i32, i32, i32, Duration) {
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
            let elapsed = start.elapsed();
            info!(target: "benchmark", "Time: {:?}", elapsed);
            return if result == *expected {
                info!(target: "benchmark", "Right result: {:?} Formula {:?}", result, path);
                (1, 0, 0, elapsed)
            } else if result == FormulaResultType::Timeout {
                warn!(target: "benchmark", "Timeout result: {:?} Formula {:?}", result, path);
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
            let (solved, timeout, error, time) = bench(&path, &expected);
            if solved == 1 {
                let mut data = solved_times.lock().unwrap();
                data.push(time);
            }
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

    let mut data = solved_times.lock().unwrap();
    data.sort();
    plot_data(&data, total_counter.load(Ordering::SeqCst) as i32,"cactus_plot.png").unwrap();

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
            let time = start.elapsed();
            info!("Time: {:?}", time);
            times.push(time);
        }
    }

    plot_data(&times, 30,"cactus_test_plot.png").unwrap();
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