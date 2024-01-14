use std::collections::VecDeque;
use std::fs::File;
use std::io::{Read};
use std::path::PathBuf;
use crate::schemas::{Clause, Formula, Value, Variable};


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
        // Split as part by whitespace
        let parts: Vec<&str> = s.split_whitespace().collect();
        let mut literals: Vec<i16> = Vec::new();
        for part in &parts[..parts.len()] {
            let lit = part.parse::<i16>().expect("Can parse number");
            if lit == 0 {
                continue;
            }
            literals.push(lit);
            let var = lit.abs() as usize;
            if lit > 0 {
                // DIMACS CNF format's variables are numbered from 1
                // but the variables are numbered from 0
                variables[var - 1].positive_occurrences.push(clause_index);
            } else {
                variables[var - 1].negative_occurrences.push(clause_index);
            }
        }
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
        file.read_to_string(&mut contents).expect("Error reading file");

        let mut clause_index = 0;
        let mut clauses = Vec::new();
        let mut variables = Vec::new();
        let units = VecDeque::new();
        let assigment_stack = Vec::new();

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
            } else {
                let clause = Clause::create_clause(line, &mut variables, clause_index);
                clauses.push(clause);
                clause_index += 1;
            }
        }

        if clauses.is_empty() || variables.is_empty() {
            return Err("file is empty");
        }

        Ok(Self {
            clauses,
            variables,
            units,
            assigment_stack,
        })
    }
}