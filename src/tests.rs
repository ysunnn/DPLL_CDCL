use crate::dpll::dpll;
use crate::dpll::schemas::{Formula, FormulaResultType, HeuristicType};
use crate::utils::plot_data;
use log::{info, warn};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use std::{fs, time};

pub fn test() {
    let start = time::Instant::now();
    //let path = PathBuf::from("data/inputs/test\\sat\\count4_2.cnf");
    let path = PathBuf::from("data/inputs/test\\sat\\tent4_4.cnf");
    info!("Formula {:?}", path);
    let mut formula = Formula::from_file(&PathBuf::from(path)).unwrap();
    formula.heuristic_type = HeuristicType::None;
    //formula.update_score();
    dpll::dpll(&mut formula, Arc::new(AtomicBool::new(false)));

    for clause in formula.clauses.iter() {
        info!("Clause: {:?}", clause);
    }

    for variable in formula.variables.iter() {
        info!("Variable: {:?}", variable);
    }

    info!("Result: {:?}", formula.result);
    info!("Time: {:?}", start.elapsed());
}

pub fn tests() {
    let mut data = Vec::new();

    for heuristic in vec![
        HeuristicType::None,
        /*HeuristicType::DLIS,
        HeuristicType::DLCS,
        HeuristicType::MOM,
        HeuristicType::JeroslowWang,
        HeuristicType::VSIDS,*/
    ] {
        info!("Heuristic: {:?}", heuristic);
        let dirs = fs::read_dir("data/inputs/test").unwrap();
        let mut times: Vec<Duration> = Vec::new();
        for dir in dirs {
            let dir = dir.unwrap().path();
            let cdir = dir.file_name().unwrap();
            let excepted = match cdir.to_str().unwrap() {
                "sat" => FormulaResultType::Satisfiable,
                "unsat" => FormulaResultType::Unsatisfiable,
                _ => {
                    continue;
                }
            };
            for path in fs::read_dir(dir).unwrap() {
                let path = path.unwrap().path();
                if path == PathBuf::from("data/inputs/test\\unsat\\tree5.cnf")
                    && heuristic == HeuristicType::MOM
                {
                    warn!("Skipping {:?}", path);
                    continue;
                }
                info!("Formula {:?}", path);
                let start = time::Instant::now();
                let mut formula = Formula::from_file(&path).unwrap();
                formula.heuristic_type = heuristic;
                //formula.update_score();

                dpll::dpll(&mut formula, Arc::new(AtomicBool::new(false)));
                info!("Result: {}", Formula::write_solution(&formula));
                assert_eq!(formula.result, excepted);
                let time = start.elapsed();
                info!("Time: {:?}", time);
                times.push(time);
            }
        }
        let x: u32 = times.iter().map(|x| x.as_micros() as u32).sum();
        info!("{} micro seconds", x / times.len() as u32);
        data.push((heuristic, times.clone()));
    }

    plot_data(&data, 30, "cactus_test_plot.png").unwrap();
}
