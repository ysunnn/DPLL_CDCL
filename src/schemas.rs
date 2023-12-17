use std::collections::VecDeque;
#[derive(PartialEq)]
pub enum Value {
    Null,
    True,
    False,
}
/// The clause struct
///
/// Contains the list of [`literals`](Vec<i32>), the number of [`active_literals`](i32) and the [`satisfiable`](bool) flag.
/// The [`satisfiable`](bool) flag is used to determine if the clause is satisfied.
pub struct Clause {
    pub(crate) satisfiable: bool,
    pub(crate) literals: Vec<i32>,
    pub(crate) active_literals: i32,
}

/// The variable struct
///
/// Contains the value of the variable and the list of clauses where it occurs.
/// The list of clauses is split into [`positive`](Vec<i32>) and [`negative`](Vec<i32>) occurrences.
pub struct Variable {
    pub(crate) value: Value,
    pub(crate) positive_occurrences: Vec<i32>,
    pub(crate) negative_occurrences: Vec<i32>,
}

/// The formula struct
///
/// Combines the list of [`clauses`](Clause) and the list of [`variables`](Variable).
/// it also contains a [`units`](VecDeque) of units that need to be propagated.
pub struct Formula {
    pub(crate) clauses: Vec<Clause>,
    pub(crate) variables: Vec<Variable>,
    pub(crate) units: VecDeque<usize>,
}