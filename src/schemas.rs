use clap::ValueEnum;
use std::collections::VecDeque;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Value {
    Null,
    True,
    False,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum HeuristicType {
    None,
    MOM,
    DLIS,
    DLCS,
    JeroslowWang,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum AssigmentType {
    Forced,
    Branching,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ResultType {
    Conflict,
    Success,
}

#[derive(PartialEq, Debug)]
pub enum FormulaResultType {
    Unknown,
    Unsatisfiable,
    Satisfiable,
    Timeout,
}

#[derive(Debug)]
pub enum PureType {
    Positive,
    Negative,
}

/// The clause struct
///
/// Contains the list of [`literals`](Vec<i32>), the number of [`active_literals`](i32) and the [`satisfiable`](bool) flag.
/// The [`satisfiable`](bool) flag is used to determine if the clause is satisfied.
#[derive(Debug)]
pub struct Clause {
    pub(crate) satisfiable: bool,
    pub(crate) satisfied_by_variable: usize,
    // variable start with 1  index with 0
    pub(crate) literals: Vec<i16>,
    pub(crate) number_of_active_literals: u8,
}

/// The variable struct
///
/// Contains the value of the variable and the list of clauses where it occurs.
/// The list of clauses is split into [`positive`](Vec<i32>) and [`negative`](Vec<i32>) occurrences.
#[derive(Debug, Clone)]
pub struct Variable {
    pub(crate) value: Value,
    pub(crate) positive_occurrences: Vec<usize>,
    pub(crate) negative_occurrences: Vec<usize>,
}

impl Variable {
    pub(crate) fn is_pure(&self) -> Option<PureType> {
        if self.positive_occurrences.is_empty() {
            Some(PureType::Negative)
        } else if self.negative_occurrences.is_empty() {
            Some(PureType::Positive)
        } else {
            None
        }
    }

    pub(crate) fn dlis(&self) -> usize {
        if self.positive_occurrences.len() > self.negative_occurrences.len() {
            self.positive_occurrences.len()
        } else {
            self.negative_occurrences.len()
        }
    }

    pub(crate) fn dlcs(&self) -> usize {
        self.positive_occurrences.len() + self.negative_occurrences.len()
    }
}

/// The assignment struct
///
/// Contains the variable and the value that was assigned to it.
/// This struct is used to store the assignments in the [`assigment_stack`](Vec<Assignment>).
#[derive(Copy, Clone, Debug)]
pub struct Assignment {
    pub(crate) variable: usize,
    pub(crate) assigment_type: AssigmentType,
}

/// The formula struct
///
/// Combines the list of [`clauses`](Clause) and the list of [`variables`](Variable).
/// it also contains a [`units`](VecDeque) of units that need to be propagated.
#[derive(Debug)]
pub struct Formula {
    pub(crate) clauses: Vec<Clause>,
    pub(crate) variables: Vec<Variable>,
    pub(crate) units: VecDeque<i16>,
    pub(crate) assigment_stack: Vec<Assignment>,
    pub(crate) result: FormulaResultType,
    pub(crate) number_of_unsatisfied_clauses: i16,
    pub(crate) variables_index: Vec<(usize, usize)>,
}

impl Formula {
    pub fn is_solved(&self) -> bool {
        return self.number_of_unsatisfied_clauses == 0;
    }

    pub fn dlis(&mut self) {
        let mut variables_index = self
            .variables
            .iter()
            .enumerate()
            .map(|(index, var)| (index, var.dlis()))
            .collect::<Vec<(usize, usize)>>();
        variables_index.sort_by(|a, b| b.1.cmp(&a.1));
        self.variables_index = variables_index;
    }

    pub fn dlcs(&mut self) {
        let mut variables_index = self
            .variables
            .iter()
            .enumerate()
            .map(|(index, var)| (index, var.dlcs()))
            .collect::<Vec<(usize, usize)>>();
        variables_index.sort_by(|a, b| b.1.cmp(&a.1));
        self.variables_index = variables_index;
    }

    pub fn mom(&mut self) {
        let mut variables_index = self
            .variables
            .iter()
            .enumerate()
            .map(|(index, var)| (index, var.dlcs()))
            .collect::<Vec<(usize, usize)>>();
        variables_index.sort_by(|a, b| b.1.cmp(&a.1));
        variables_index.reverse();
        self.variables_index = variables_index;
    }

    pub fn jeroslow_wang_score(&mut self) {
        let mut variables_index = Vec::new();
        for (index, var) in self.variables.iter().enumerate() {
            let mut score = 0.0;
            for clause_index in var.positive_occurrences.iter() {
                score +=
                    2.0f64.powi(-(self.clauses[*clause_index].number_of_active_literals as i32));
            }
            for clause_index in var.negative_occurrences.iter() {
                score +=
                    2.0f64.powi(-(self.clauses[*clause_index].number_of_active_literals as i32));
            }
            variables_index.push((index, score as usize));
        }
        variables_index.sort_by(|a, b| b.1.cmp(&a.1));
        self.variables_index = variables_index;
    }
}
