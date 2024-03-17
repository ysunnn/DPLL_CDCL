use clap::ValueEnum;
use log::{debug, error};
use std::collections::{HashSet, VecDeque};

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
}

impl Clause {

    pub fn find_new_variable_to_watch(&mut self, variable_index: usize,
                                      variables: &mut Vec<Variable>,
                                      clause_index: usize) -> Result<Option<(usize, Value)>, i8 >{
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
        debug!(target: "find_new_variable_to_watch", "num of literals: {}", self.literals.len());
        for index in my_watched_index..self.literals.len()+my_watched_index {
            let literal_index = index % self.literals.len();
            let lit = self.literals[literal_index];
            let variable = &mut variables[lit.abs() as usize - 1];
            debug!(target: "find_new_variable_to_watch", "current index: {}", index);
            debug!(target: "find_new_variable_to_watch", "current literal_index: {}", literal_index);
            debug!(target: "find_new_variable_to_watch", "current lit: {}", lit);
            debug!(target: "find_new_variable_to_watch", "current variable: {:?}", variable);
            // satisfied clause dont play a role
            if variable.value == Value::True && lit > 0 || variable.value == Value::False && lit < 0 {
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
            if lit > 0 {
                variable.watched_pos_occurrences.insert(clause_index);
            } else {
                variable.watched_neg_occurrences.insert(clause_index);
            }
            let old_lit = self.literals[my_watched_index];
            // remove the clause from the old variable that is not watched anymore !
            if old_lit > 0 {
                variables[old_lit.abs() as usize - 1].watched_pos_occurrences.remove(&clause_index);
            } else {
                variables[old_lit.abs() as usize - 1].watched_neg_occurrences.remove(&clause_index);
            }
            debug!(target: "find_new_variable_to_watch", "update watched variables: {:?}", self.watched);
            return Ok(None);
        }

        // conflict id maybe_unit is false
        if maybe_unit{
            // variable to propagate
            let value = if self.literals[other_watched_index]>0{
                Value::True
            } else {
                Value::False
            };
            return Ok(Some((self.literals[other_watched_index].abs() as usize - 1, value)));
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
    pub score: f32,
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
}

/// The formula struct
///
/// Combines the list of [`clauses`](Clause) and the list of [`variables`](Variable).
/// it also contains a [`units`](VecDeque) of units that need to be propagated.
#[derive(Debug)]
pub struct Formula {
    pub(crate) clauses: Vec<Clause>,
    pub(crate) variables: Vec<Variable>,
    pub(crate) units: VecDeque<(usize, Value)>,
    pub(crate) assigment_stack: Vec<Assignment>,
    pub(crate) result: FormulaResultType,
    pub(crate) variables_index: Vec<(usize, f32)>,
    pub heuristic_type: HeuristicType,
}

impl Formula {

    pub fn assigment_stack_pop(&mut self) -> Option<Assignment> {
        self.assigment_stack.pop()
    }
    pub fn assigment_stack_push(&mut self, assignment: Assignment) {
        if assignment.value == Value::Null {
            error!("Null value");
            panic!("Null value");
        }
        self.assigment_stack.push(assignment);
    }

    pub fn assigment_stack_is_empty(&self) -> bool {
        return self.assigment_stack.is_empty();
    }
}
