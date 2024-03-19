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

pub enum ImplicationReason {
    Decision,
    LearnedClause(usize),
    Null, // branching assignments
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
    pub(crate) positive_occurrences: Vec<usize>,
    pub(crate) negative_occurrences: Vec<usize>,
    pub score: f32,
    pub assigment_depth: usize,
}

/// The assignment struct
///
/// Contains the variable and the value that was assigned to it.
/// This struct is used to store the assignments in the [`assigment_stack`](Vec<Assignment>).
#[derive(Copy, Clone, Debug)]
pub struct Assignment {
    pub(crate) variable_index: usize,
    pub(crate) assigment_type: AssigmentType,
    pub(crate) variable_value: Value,
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
        if assignment.variable_value == Value::Null {
            error!("Null value");
            panic!("Null value");
        }
        self.assigment_stack.push(assignment);
    }

    pub fn assigment_stack_is_empty(&self) -> bool {
        return self.assigment_stack.is_empty();
    }
}

/// The implication graph struct
///
/// Directed acyclic graph representing implications between assignments.
/// Assignment vertices for variables, values, and branch depth.
/// Edges for which clause caused the conflict and the reason().
pub(crate) struct Edge {
    reason: ImplicationReason,
    trigger: Option<Assignment>, // Clause triggering the unit propagation
}

pub(crate) struct ImplicationGraph {
    pub(crate) assignments: Vec<Assignment>,
    pub(crate) edges: Vec<Edge>,
    pub(crate) conflict: Option<Clause>, // Clause that caused a conflict
}

impl ImplicationGraph {
    fn add_assignment(&mut self, assignment: Assignment) {
        self.assignments.push(assignment);
    }

    fn add_edge(&mut self, reason: ImplicationReason, trigger: Option<Assignment>) {
        let edge = Edge { reason, trigger };
        self.edges.push(edge);
    }

    pub fn set_conflict(&mut self, clause: Clause) {
        self.conflict = Some(clause);
    }
    pub fn update_graph_for_branching(&mut self, new_assignment: Assignment) {
        self.add_assignment(new_assignment);
    }
    pub fn update_graph_for_unit_propagation(&mut self, formula: &mut Formula, new_assignment: Assignment) {
        self.add_assignment(new_assignment);
        fn process_occurrences(
            graph: &mut ImplicationGraph,
            formula: &Formula,
            assignment: &Assignment,
            occurrences: &[usize],
        ) {
            for &clause_index in occurrences {
                for &literal in &formula.clauses[clause_index].literals {
                    let literal_index = (literal.abs() - 1) as usize;
                    if &formula.variables[literal_index].assigment_depth <= &assignment.depth {
                        graph.add_edge(ImplicationReason::LearnedClause(clause_index), Some(*assignment));
                    }
                }
            }
        }

        let variable_index = new_assignment.variable_index;
        process_occurrences(self, formula, &new_assignment, &formula.variables[variable_index].positive_occurrences);
        process_occurrences(self, formula, &new_assignment, &formula.variables[variable_index].negative_occurrences);
    }
}

