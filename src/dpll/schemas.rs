use clap::ValueEnum;
use log::error;
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

#[derive(Debug)]
pub enum PureType {
    Positive,
    Negative,
}

pub enum ImplicationReason {
    Decision,
    LearnedClause(Clause),
    Null, // branching assignments
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
impl Clause {
    pub fn is_satisfied(&self) -> bool {
        return self.satisfiable;
    }

    /// Set the clause to true
    ///
    /// This function sets the clause to true and updates the variables scores.
    /// For all the literals in the clause, the function will decrease the number of unsolved clauses of the variable.
    pub fn set_true(&mut self, variable: usize, variables: &mut Vec<Variable>) {
        if !self.satisfiable {
            // we update all the variables scores and decrease the number of unsolved clauses because this is solved.
            self.update_variables_scores(variables, -1);
        }
        self.satisfiable = true;
        self.satisfied_by_variable = variable;
    }

    pub fn undo(&mut self, variable: usize, variables: &mut Vec<Variable>) -> i8 {
        if self.satisfied_by_variable == variable {
            self.satisfiable = false;
            self.satisfied_by_variable = 0;
            self.update_variables_scores(variables, 1);
            return 1;
        }
        return 0;
    }
    fn update_variables_scores(&self, variables: &mut Vec<Variable>, value: i8) {
        for literal in &self.literals {
            let variable_index = literal.abs() as usize - 1;
            let variable = &mut variables[variable_index];
            if *literal > 0 {
                variable.num_of_unsolved_clauses_with_positive_occurrences += value as i16;
            } else {
                variable.num_of_unsolved_clauses_with_negative_occurrences += value as i16;
            }
        }
    }
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
    pub(crate) num_of_unsolved_clauses_with_negative_occurrences: i16,
    pub(crate) num_of_unsolved_clauses_with_positive_occurrences: i16,
    pub score: f32,
}

impl Variable {
    pub(crate) fn is_pure(&self) -> Option<PureType> {
        if self.num_of_unsolved_clauses_with_positive_occurrences == 0 {
            Some(PureType::Negative)
        } else if self.num_of_unsolved_clauses_with_negative_occurrences == 0 {
            Some(PureType::Positive)
        } else {
            None
        }
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
    pub(crate) units: VecDeque<i16>,
    pub(crate) assigment_stack: Vec<Assignment>,
    pub(crate) result: FormulaResultType,
    pub(crate) number_of_unsatisfied_clauses: i16,
    pub(crate) variables_index: Vec<(usize, f32)>,
    pub heuristic_type: HeuristicType,
}

impl Formula {
    pub fn is_solved(&self) -> bool {
        return self.number_of_unsatisfied_clauses == 0;
    }

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
    pub fn add_assignment(&mut self, assignment: Assignment) {
        self.assignments.push(assignment);
    }

    pub fn add_edge(&mut self, reason: ImplicationReason, trigger: Option<Assignment>) {
        let edge = Edge { reason, trigger };
        self.edges.push(edge);
    }

    pub fn set_conflict(&mut self, clause: Clause) {
        self.conflict = Some(clause);
    }
    pub fn update_graph_for_unit_propagation(&mut self, formula: &mut Formula, new_assignment: Assignment) {
        self.add_assignment(new_assignment);
    }
}
