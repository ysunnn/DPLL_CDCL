use crate::dpll::schemas::Value::Null;
use clap::ValueEnum;
use log::{debug, error};
use std::collections::{HashMap, HashSet, VecDeque};

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
    VSIDS,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum AssigmentType {
    Forced,
    Branching,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum SetResultType {
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
#[derive(PartialEq, Debug)]
pub enum ClauseType {
    Original,
    Learned,
}

/// The clause struct
///
/// Contains the list of [`literals`](Vec<i32>), the number of [`active_literals`](i32) and the [`satisfiable`](bool) flag.
/// The [`satisfiable`](bool) flag is used to determine if the clause is satisfied.
#[derive(Debug)]
pub struct Clause {
    // variable start with 1  index with 0
    pub(crate) literals: Vec<i16>,
    // both watched are indexes to the literals of the clause
    pub(crate) watched: (usize, usize),
    pub clause_type: ClauseType,
    pub activity: u16,
}

impl Clause {
    pub fn find_new_variable_to_watch(
        &mut self,
        variable_index: usize,
        variables: &mut Vec<Variable>,
        clause_index: usize,
    ) -> Result<Option<(usize, Value, usize)>, i8> {
        let my_watched_index;
        let other_watched_index;
        debug!(target: "find_new_variable_to_watch", "watched: {:?}", self.watched);
        if variable_index == (self.literals[self.watched.0].abs() - 1) as usize {
            my_watched_index = self.watched.0;
            other_watched_index = self.watched.1;
        } else {
            my_watched_index = self.watched.1;
            other_watched_index = self.watched.0;
        }
        let mut maybe_unit = false;
        let old_lit = self.literals[my_watched_index];
        let old_lit_pos = old_lit > 0;
        debug!(target: "find_new_variable_to_watch", "num of literals: {}", self.literals.len());
        for literal_index in 0..self.literals.len() {
            let lit = self.literals[literal_index];
            let variable = &mut variables[lit.abs() as usize - 1];
            debug!(target: "find_new_variable_to_watch", "current literal_index: {}", literal_index);
            debug!(target: "find_new_variable_to_watch", "current lit: {}", lit);
            debug!(target: "find_new_variable_to_watch", "current variable: {:?}", variable);
            // satisfied clause dont play a role
            let lit_pos = lit > 0;
            let lit_neq = lit < 0;
            if variable.value == Value::True && lit_pos || variable.value == Value::False && lit_neq
            {
                return Ok(None);
            }

            // if the variable is not free we're looking for the next on
            if variable.value != Value::Null {
                continue;
            }
            debug!(target: "find_new_variable_to_watch", "index: {}, other_watched_index: {}", literal_index, other_watched_index);
            if literal_index == other_watched_index {
                debug!(target: "find_new_variable_to_watch", "this is maybe a unit");
                maybe_unit = true;
                continue;
            }
            self.watched = (literal_index, other_watched_index);
            // Add the clause to the new variable that is watched
            if lit_pos {
                variable.watched_pos_occurrences.insert(clause_index);
            } else {
                variable.watched_neg_occurrences.insert(clause_index);
            }

            // remove the clause from the old variable that is not watched anymore !
            if old_lit_pos {
                variables[old_lit.abs() as usize - 1]
                    .watched_pos_occurrences
                    .remove(&clause_index);
            } else {
                variables[old_lit.abs() as usize - 1]
                    .watched_neg_occurrences
                    .remove(&clause_index);
            }
            debug!(target: "find_new_variable_to_watch", "update watched variables: {:?}", self.watched);
            return Ok(None);
        }

        // conflict id maybe_unit is false
        if maybe_unit {
            // variable to propagate
            let value = if self.literals[other_watched_index] > 0 {
                Value::True
            } else {
                Value::False
            };
            return Ok(Some((
                self.literals[other_watched_index].abs() as usize - 1,
                value,
                clause_index,
            )));
        }
        // conflict
        return Err(0);
    }
}

/// The variable struct
///
/// Contains the value of the variable and the list of clauses where it occurs.
#[derive(Debug, Clone)]
pub struct Variable {
    pub(crate) value: Value,
    // a set of all indexes to clauses where the current variables occur negative and is watched
    pub watched_neg_occurrences: HashSet<usize>,
    // a set of all indexes to clauses where the current variables occur positive and is watched
    pub watched_pos_occurrences: HashSet<usize>,
    pub(crate) positive_occurrences: Vec<usize>,
    pub(crate) negative_occurrences: Vec<usize>,
    pub score: f32,
    pub depth: usize,
    // None for branching and clauses index as usize for unit propagation trigger.
    pub reason: Option<usize>,
}

/// The assignment struct
///
/// Contains the variable and the value that was assigned to it.
/// This struct is used to store the assignments in the [`assigment_stack`](Vec<Assignment>).
#[derive(Copy, Clone, Debug)]
pub struct Assignment {
    pub(crate) variable_index: usize,
    pub(crate) assigment_type: AssigmentType,
    pub(crate) value: Value,
    pub(crate) depth: usize,
}

/// The formula struct
///
/// Combines the list of [`clauses`](Clause) and the list of [`variables`](Variable).
/// it also contains a [`units`](VecDeque) of units that need to be propagated.
#[derive(Debug)]
pub struct Formula {
    pub(crate) clauses: Vec<Clause>,
    pub(crate) variables: Vec<Variable>,
    pub(crate) units: VecDeque<(usize, Value, usize)>,
    pub(crate) assigment_stack: Vec<Assignment>,
    pub(crate) result: FormulaResultType,
    pub(crate) variables_index: Vec<(usize, f32)>,
    pub heuristic_type: HeuristicType,
    pub original_clause_vector_length: usize,
}

impl Formula {
    pub fn assigment_stack_pop(&mut self) -> Option<Assignment> {
        self.assigment_stack.pop()
    }
    pub fn assigment_stack_push(&mut self, assignment: Assignment) {
        if assignment.value == Null {
            error!("Null value");
            panic!("Null value");
        }
        self.assigment_stack.push(assignment);
    }

    /// Add a new learned clause to the formular by a list of literates,
    /// all dependent variables get updated accordingly.
    pub fn add_clauses(&mut self, literals: Vec<i16>) {
        // TODO remove for release only for testing
        if literals.len() < 2 {
            panic!(
                "It doesnt make sense to add an clause that has only one literal ! {:?}",
                literals
            )
        }
        let clause_index = self.clauses.len();
        // UPDATE all variables that appear in the new clause
        // Do wee need to update all variables or only the watched ones ??
        for lit in &literals {
            let variables_index = (lit.abs() - 1) as usize;
            if *lit > 0 {
                self.variables[variables_index]
                    .positive_occurrences
                    .push(clause_index);
                if *lit == literals[0] || *lit == literals[1] {
                    self.variables[variables_index]
                        .watched_pos_occurrences
                        .insert(clause_index);
                }
            } else {
                self.variables[variables_index]
                    .negative_occurrences
                    .push(clause_index);
                if *lit == literals[0] || *lit == literals[1] {
                    self.variables[variables_index]
                        .watched_neg_occurrences
                        .insert(clause_index);
                }
            }
        }

        let clause = Clause {
            literals,
            watched: (0, 1),
            clause_type: ClauseType::Learned,
            activity: 0,
        };
        self.clauses.push(clause);
    }
    /// Removes a learned clause from the formular by the clause index it panics if the index auf the
    /// clauses points to an original clauses!
    /// Also, the operation of deleting the clause index from the positive and negativ occurrences
    /// are really expressive, we first have to find the index of the value to remove it ...
    pub fn delete_clauses(&mut self, clause_index: usize) {
        if self.clauses[clause_index].clause_type != ClauseType::Learned {
            panic!(
                "You can not remove a original clause from the formular only learned ones! \
            and the clause with index {} is not learned: {:?}",
                clause_index, &self.clauses[clause_index]
            )
        }
        let clause = self.clauses.remove(clause_index);

        for lit in &clause.literals {
            let variables_index = (lit.abs() - 1) as usize;
            if *lit > 0 {
                // TODO there must be a better way, this is to fucking expensive !!
                let index = self.variables[variables_index]
                    .positive_occurrences
                    .iter()
                    .position(|&x| x == clause_index)
                    .unwrap();
                self.variables[variables_index]
                    .positive_occurrences
                    .remove(index);
                // maybe the hashset lookup ist less expensive than the if statement ?
                if *lit == clause.literals[clause.watched.0]
                    || *lit == clause.literals[clause.watched.1]
                {
                    self.variables[variables_index]
                        .watched_pos_occurrences
                        .remove(&clause_index);
                }
            } else {
                let index = self.variables[variables_index]
                    .negative_occurrences
                    .iter()
                    .position(|&x| x == clause_index)
                    .unwrap();
                self.variables[variables_index]
                    .negative_occurrences
                    .remove(index);
                if *lit == clause.literals[clause.watched.0]
                    || *lit == clause.literals[clause.watched.1]
                {
                    self.variables[variables_index]
                        .watched_neg_occurrences
                        .remove(&clause_index);
                }
            }
        }
    }
    pub fn assigment_stack_is_empty(&self) -> bool {
        return self.assigment_stack.is_empty();
    }
}
