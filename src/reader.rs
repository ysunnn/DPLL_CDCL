use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use crate::schemas::{Clause, Formula, Value, Variable};


impl Variable {
    /// Create new variable
    fn create_variable(_num: u16) -> Self {
        Self {
            value: Value::Null,
            positive_occurrences: Vec::new(),
            negative_occurrences: Vec::new(),
        }
    }
}


impl Clause {
    /// Read line and convert it to Clause
    fn create_clause(s: &str, variables: &mut Vec<Variable>, clause_index: usize) -> Result<Self, String> {
        // Split as part by whitespace
        let parts: Vec<&str> = s.split_whitespace().filter(|p| !p.is_empty()).collect();
        // Check if the last part is zero
        if parts.last() != Some(&"0") {
            return Err("Invalid format: last part of clause is not 0".to_string());
        }
        let mut literals: Vec<i16> = Vec::new();
        for part in &parts[..parts.len() - 1] { // except the last past(zero)
            let lit = i16::from_str(part).map_err(|e| e.to_string())?;
            if lit == 0 {
                return Err("Invalid format: Zero in clause".to_string());
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
        Ok(Self {
            satisfiable: false,
            satisfied_by_variable: 0,
            number_of_active_literals: literals.len() as u8,
            literals,

        })
    }
}


impl Formula {
    /// Read a DIMACS CNF file and convert it to predefined Formula
    pub fn read_formula(filename: &str) -> Result<Self, String> {
        let file = File::open(filename).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);

        let mut clause_index = 0;
        let mut clauses = Vec::new();
        let mut variables = Vec::new();
        let mut units = VecDeque::new();
        let mut assigment_stack = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| e.to_string())?;
            let line = line.trim();
            // Ignore empty lines and comments
            if line.is_empty() || line.starts_with('c') {
                continue;
            }
            // Handel header
            if line.starts_with('p') {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() != 4 || parts[1] != "cnf" {
                    return Err("Invalid header".to_string());
                }
                let num_vars = u16::from_str(parts[2]).map_err(|e| e.to_string())?;
                let num_clauses = u16::from_str(parts[3]).map_err(|e| e.to_string())?;
                clauses.reserve(num_clauses as usize);
                variables.reserve(num_vars as usize);
                for i in 1..=num_vars {
                    let var = Variable::create_variable(i);
                    variables.push(var);
                }
            } else {
                let clause = Clause::create_clause(line, &mut variables, clause_index)?;
                clauses.push(clause);
                clause_index += 1;
            }
        }

        if clauses.is_empty() || variables.is_empty() {
            return Err("file is empty".to_string());
        }

        Ok(Self {
            clauses,
            variables,
            units,
            assigment_stack,
        })
    }
}