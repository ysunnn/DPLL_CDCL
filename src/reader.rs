use crate::schemas::{Clause, Formula, FormulaResultType, Value, Variable};
use log::warn;
use std::collections::{HashSet, VecDeque};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

impl Variable {
    /// Create new variable
    fn new() -> Self {
        Self {
            value: Value::Null,
            positive_occurrences: Vec::new(),
            negative_occurrences: Vec::new(),
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
        Self {
            satisfiable: false,
            satisfied_by_variable: 0,
            number_of_active_literals: literals.len() as u8,
            literals,
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
            } else if line.starts_with('0') {
                let clause = Clause::create_clause(
                    current_literals.clone().as_str(),
                    &mut variables,
                    clause_index,
                );
                clauses.push(clause);
                clause_index += 1;
                current_literals = String::new();
            } else if line.ends_with('0') {
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

        Ok(Self {
            assigment_stack: Vec::with_capacity(variables.len()),
            clauses,
            variables,
            units: VecDeque::new(),
            result: FormulaResultType::Unknown,
        })
    }
}
