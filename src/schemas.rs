use std::collections::VecDeque;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Value {
    Null,
    True,
    False,
}
#[derive(PartialEq, Clone, Copy)]
pub enum AssigmentType{
    Forced,
    Branching,
}

#[derive(PartialEq)]
pub enum ResultType{
    Conflict,
    Success
}

/// The clause struct
///
/// Contains the list of [`literals`](Vec<i32>), the number of [`active_literals`](i32) and the [`satisfiable`](bool) flag.
/// The [`satisfiable`](bool) flag is used to determine if the clause is satisfied.
#[derive(Debug)]
pub struct Clause {
    pub(crate) satisfiable: bool,
    pub(crate) satisfied_by_variable: usize, // variable start with 1  index with 0
    pub(crate) literals: Vec<i32>,
    pub(crate) number_of_active_literals: u8,
}

/// The variable struct
///
/// Contains the value of the variable and the list of clauses where it occurs.
/// The list of clauses is split into [`positive`](Vec<i32>) and [`negative`](Vec<i32>) occurrences.
#[derive(Debug)]
pub struct Variable {
    pub(crate) value: Value,
    pub(crate) positive_occurrences: Vec<usize>,
    pub(crate) negative_occurrences: Vec<usize>,
}

/// The assignment struct
///
/// Contains the variable and the value that was assigned to it.
/// This struct is used to store the assignments in the [`assigment_stack`](Vec<Assignment>).
#[derive(Copy, Clone)]
pub struct Assignment {
    pub(crate) variable: usize,
    pub(crate) value: Value,
    pub(crate) assigment_type: AssigmentType
}

/// The formula struct
///
/// Combines the list of [`clauses`](Clause) and the list of [`variables`](Variable).
/// it also contains a [`units`](VecDeque) of units that need to be propagated.
pub struct Formula {
    pub(crate) clauses: Vec<Clause>,
    pub(crate) variables: Vec<Variable>,
    pub(crate) units: VecDeque<usize>,
    pub(crate) assigment_stack: Vec<Assignment>,
}