use crate::dpll::schemas::{Clause, Formula, FormulaResultType, HeuristicType, Value, Variable};
use log::warn;
use plotters::backend::BitMapBackend;
use plotters::chart::ChartBuilder;
use plotters::drawing::IntoDrawingArea;
use plotters::element::PathElement;
use plotters::prelude::{IntoFont, LineSeries, BLACK, BLUE, CYAN, GREEN, MAGENTA, RED, WHITE};
use plotters::style::Color;
use std::collections::{HashSet, VecDeque};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;

impl Variable {
    /// Create new variable
    fn new() -> Self {
        Self {
            value: Value::Null,
            watched_neg_occurrences: HashSet::new(),
            watched_pos_occurrences: HashSet::new(),
            positive_occurrences: Vec::new(),
            negative_occurrences: Vec::new(),
            score: 0.0,
            depth: 0,
        }
    }
}

impl Clause {
    /// Read line and convert it to Clause
    fn create_clause(s: &str, variables: &mut Vec<Variable>, clause_index: usize) -> Self {
        let mut literals_set: HashSet<i16> = HashSet::new();
        // Split as part by whitespace
        let parts: Vec<&str> = s.split_whitespace().collect();
        for part in &parts[..parts.len()] {
            let lit = part.parse::<i16>().expect("Can parse number");
            if lit == 0 {
                continue;
            }
            if literals_set.contains(&lit) {
                warn!("Duplicate literal: {}", lit);
                continue;
            }
            literals_set.insert(lit);
            let var = lit.abs() as usize;
            if lit > 0 {
                // DIMACS CNF format's variables are numbered from 1
                // but the variables are numbered from 0
                variables[var - 1].positive_occurrences.push(clause_index);
            } else {
                variables[var - 1].negative_occurrences.push(clause_index);
            }
        }

        let literals: Vec<i16> = literals_set.into_iter().collect();
        let watched;
        if literals.len() == 1 {
            let lit = literals[0];
            if lit > 0 {
                variables[(literals[0].abs() - 1) as usize].watched_pos_occurrences.insert(clause_index);
            }else {
                variables[(literals[0].abs() - 1) as usize].watched_neg_occurrences.insert(clause_index);
            }
            watched = (0,0);
        }else{
            let lit = literals[0];
            if lit > 0 {
                variables[(literals[0].abs() - 1) as usize].watched_pos_occurrences.insert(clause_index);
            }else {
                variables[(literals[0].abs() - 1) as usize].watched_neg_occurrences.insert(clause_index);
            }
            let lit = literals[1];
            if lit > 0 {
                variables[(literals[1].abs() - 1) as usize].watched_pos_occurrences.insert(clause_index);
            }else {
                variables[(literals[1].abs() - 1) as usize].watched_neg_occurrences.insert(clause_index);
            }
            watched = (0,1);
        }

        Self {
            literals,
            watched,
        }
    }
}

impl Formula {
    /// Read a DIMACS CNF file and convert it to predefined Formula
    pub fn from_file(filename: &PathBuf) -> Result<Self, &str> {
        let mut file = File::open(filename).expect("File not found");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Error reading file");

        let mut clause_index = 0;
        let mut clauses = Vec::new();
        let mut variables = Vec::new();

        let mut current_literals = String::new();

        for line in contents.lines() {
            let line = line.trim();
            // Ignore empty lines and comments
            if line.is_empty() || line.starts_with('c') {
                continue;
            }
            // Handel header
            if line.starts_with('p') {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() != 4 || parts[1] != "cnf" {
                    return Err("Invalid header");
                }
                let num_vars = parts[2].parse::<u16>().expect("Can parse num vars");
                let num_clauses = parts[3].parse::<u16>().expect("Can parse num clauses");
                clauses = Vec::with_capacity(num_clauses as usize);
                variables = vec![Variable::new(); num_vars as usize];
            } else if line.trim() == "0" {
                let clause = Clause::create_clause(
                    current_literals.clone().as_str(),
                    &mut variables,
                    clause_index,
                );
                clauses.push(clause);
                clause_index += 1;
                current_literals = String::new();
            } else if line.ends_with(" 0") {
                let clause = Clause::create_clause(line, &mut variables, clause_index);
                clauses.push(clause);
                clause_index += 1;
            } else {
                current_literals = format!("{current_literals} {line}");
            }
        }

        if clauses.is_empty() || variables.is_empty() {
            return Err("file is empty");
        }

        let variables_index = variables
            .iter()
            .enumerate()
            .map(|(index, _)| (index, 0.0))
            .collect::<Vec<(usize, f32)>>();

        Ok(Self {
            assigment_stack: Vec::with_capacity(variables.len()),
            clauses,
            variables,
            units: VecDeque::new(),
            result: FormulaResultType::Unknown,
            variables_index,
            heuristic_type: HeuristicType::None,
        })
    }

    pub fn write_solution(&self) -> String {
        let solution = match self.result {
            FormulaResultType::Satisfiable => {
                let literals: Vec<String> = self
                    .variables
                    .iter()
                    .enumerate()
                    .map(|(index, var)| {
                        if var.value == Value::True {
                            (index + 1).to_string()
                        } else {
                            (-((index + 1) as i32)).to_string()
                        }
                    })
                    .collect();
                format!("s SATISFIABLE\nv {}", literals.join(" "))
            }
            FormulaResultType::Unsatisfiable => "s UNSATISFIABLE".to_string(),
            FormulaResultType::Timeout => "s UNKNOWN\nc Timeout".to_string(),
            _ => "s UNKNOWN".to_string(),
        };
        solution
    }
}

pub fn plot_data(
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
        .build_cartesian_2d(0i32..num_of_problems, 0u128..60000)
        .unwrap();

    chart
        .configure_mesh()
        .x_desc("Solved Problems")
        .y_desc("Time in milliseconds")
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
                    current_data.iter().map(|(x, y)| (*x, y.as_millis())),
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
