use crate::dpll::schemas::{AssigmentType, Assignment, Formula, FormulaResultType, HeuristicType, PureType, SetResultType, Value};
use log::{debug, warn};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn set_variable_true(
    variable_index: usize,
    formula: &mut Formula,
    assigment_type: AssigmentType,
    clause_index: Option<usize>,
) -> SetResultType {
    if assigment_type == AssigmentType::Branching {
        formula.depth += 1;
    }
    debug!(target: "set_variable_true", "Set variable true: {} by: {:?}, current depth: {}", variable_index, assigment_type, formula.depth);
    formula.variables[variable_index].value = Value::True;
    formula.variables[variable_index].depth = formula.depth;
    formula.variables[variable_index].reason = clause_index;
    let assignment = Assignment {
        variable_index,
        assigment_type,
        value: Value::True,
        depth: formula.depth,
    };
    formula.assigment_stack_push(assignment);
    //dbg!(&formula.assigment_stack);
    let mut result = SetResultType::Success;
    debug!(target: "set_variable_true","updating all negative occurrences: {:?}", formula.variables[variable_index].watched_neg_occurrences);
    for clause_index in formula.variables[variable_index]
        .watched_neg_occurrences
        .clone()
        .iter()
    {
        debug!(target: "set_variable_true","clause: {:?} index: {}", formula.clauses[*clause_index], clause_index);
        match formula.clauses[*clause_index].find_new_variable_to_watch(
            variable_index,
            &mut formula.variables,
            *clause_index,
        ) {
            Ok(option) => {
                debug!(target: "set_variable_true","result for finding new watched variable clause {}, {:?}", clause_index, option);
                match option {
                    Some(unit) => {
                        formula.units.push_back(unit);
                    }
                    None => {
                        continue;
                    }
                }
            }
            Err(_) => {
                warn!(target: "set_variable_true","conflict fore clause: {:?} index: {}", formula.clauses[*clause_index], clause_index);
                result = SetResultType::Conflict {
                    depth: analyse_conflict_with_decision_scheme(variable_index, formula)
                };
                // Update clauses activity for BerkMin's
                formula.clauses[*clause_index].activity += 1;
            }
        }
    }
    return result;
}

fn set_variable_false(
    variable_index: usize,
    formula: &mut Formula,
    assigment_type: AssigmentType,
    clause_index: Option<usize>,
) -> SetResultType {
    if assigment_type == AssigmentType::Branching {
        formula.depth += 1;
    }
    debug!(target: "set_variable_false", "Set variable false: {} by: {:?}, current depth: {}", variable_index, assigment_type, formula.depth);
    formula.variables[variable_index].value = Value::False;
    formula.variables[variable_index].depth = formula.depth;
    formula.variables[variable_index].reason = clause_index;
    let assignment = Assignment {
        variable_index,
        assigment_type,
        value: Value::True,
        depth: formula.depth,
    };
    formula.assigment_stack_push(assignment);
    //dbg!(&formula.assigment_stack);
    let mut result = SetResultType::Success;
    debug!(target: "set_variable_false","updating all positive occurrences: {:?}", formula.variables[variable_index].watched_pos_occurrences);
    for clause_index in formula.variables[variable_index]
        .watched_pos_occurrences
        .clone()
        .iter()
    {
        debug!(target: "set_variable_false","clause: {:?} index: {}", formula.clauses[*clause_index], clause_index);
        match formula.clauses[*clause_index].find_new_variable_to_watch(
            variable_index,
            &mut formula.variables,
            *clause_index,
        ) {
            Ok(option) => {
                debug!(target: "set_variable_false","result for finding new watched variable clause {}, {:?}", clause_index, option);
                match option {
                    Some(unit) => {
                        formula.units.push_back(unit);
                    }
                    None => {
                        continue;
                    }
                }
            }
            Err(_) => {
                warn!(target: "set_variable_false","conflict fore clause: {:?} index: {}", formula.clauses[*clause_index], clause_index);
                result = SetResultType::Conflict {
                    depth: analyse_conflict_with_decision_scheme(variable_index, formula)
                };
                // Update clauses activity for BerkMin's
                formula.clauses[*clause_index].activity += 1;
            }
        }
    }
    return result;
}

/// Undo the assigment of a variable for backtracking.
/// First wie set the variable to free. We update every clause where the variable occurrences positive and is sat though this
/// variable. We set the clause to not sat.
/// for every negative occurrences in a clause we update the number of active literals by one.
fn undo_assignment(variable_index: usize, formula: &mut Formula) {
    formula.variables[variable_index].value = Value::Null;
    formula.variables[variable_index].depth = 0;
    formula.variables[variable_index].reason = None;
    // somewhere here we have to check the number of assigned variables for the clauses to delete
    // it if it's a learned one with a length greater than k and less than m literates are assigned.
    // what do we do when we have to remove it from the assigment stack?
    // maybe only remove variables after backtracking?
}

/// Depth-first search to find reachable vertices.
fn dfs(
    conflict_vertex: usize,
    formula: &mut Formula,
) -> Vec<usize> {
    let mut stack = VecDeque::new();
    let mut visited = vec![false; formula.variables.len()];
    //todo should it be a set?
    let mut reachable_vertices = Vec::new();

    stack.push_back(conflict_vertex);
    visited[conflict_vertex] = true;

    while let Some(vertex) = stack.pop_back() {
        debug!(target: "dfs", "current_index: {}", vertex);
        if formula.variables[vertex].value != Value::Null {
            reachable_vertices.push(vertex);
        }

        // Explore neighbors from positive_occurrences and negative_occurrences
        for &clause_idx in formula.variables[vertex].positive_occurrences.iter().chain(formula.variables[vertex].negative_occurrences.iter()) {
            let clause = &formula.clauses[clause_idx];
            for &literal in &clause.literals {
                let neighbor = if literal > 0 {
                    literal as usize - 1
                } else {
                    (-literal) as usize - 1
                };

                if !visited[neighbor] {
                    stack.push_back(neighbor);
                    visited[neighbor] = true;
                }
            }
        }
    }
    reachable_vertices
}

/// Cut based on decision scheme and add an asserting conflict clause.
/// Find and give second-largest branching depth.
fn analyse_conflict_with_decision_scheme(conflict_vertex: usize, formula: &mut Formula) -> usize {
    let reachable_vertices = dfs(conflict_vertex, formula);
    debug!(target: "analyse_conflict_with_decision_scheme", "conflict_vertex: {},reachable_vertices: {:?}",conflict_vertex, reachable_vertices);
    let mut depths: Vec<usize> = Vec::new();
    let mut conflict_clause_literal = Vec::new();  // All branching vertices from which conflict clause can be reached.
    // let mut implied_vertices= Vec::new(); // Vertices that are not branching vertices but are part of the conflict clause.
    for literal in reachable_vertices {
        // All branching vertices from which conflict clause can be reached.
        debug!(target: "analyse_conflict_with_decision_scheme", "literal: {}", literal);
        if formula.variables[literal].reason == None {
            match formula.variables[literal].value {
                Value::True => conflict_clause_literal.push(-1 * ((literal + 1) as i16)),
                Value::False => conflict_clause_literal.push((literal + 1) as i16),
                _ => {
                    warn!(target: "analyse_conflict_with_decision_scheme", "branching_vertices should have assigned values");
                }
            }
            depths.push(formula.variables[literal].depth)
        }
    }
    debug!(target: "analyse_conflict_with_decision_scheme", "new clause to learn: {:?}", &conflict_clause_literal);
    formula.add_clauses(conflict_clause_literal);

    // Find the second-largest branching depth
    depths.sort_unstable_by(|a, b| b.cmp(a));
    if depths.len() >= 2 {
        depths[1]
    } else {
        0
    }
}

/// Backtrack the forced assigment
///
/// First we undo all the Forced assignments, if the assignment stack get empty in this process the formula is unsat.
/// If we still got a Branched assigment we undo this as well empty our unit queue and set the variable to false.
/// Then we're going back to normal and start with unit propagation and regular assignments.
fn backtrack(
    formula: &mut Formula,
    depth: usize,
) -> Option<FormulaResultType> {
    while let Some(top) = formula.assigment_stack_pop() {
        // undo all assigment where the depth is bigger than the given depth
        if top.depth > depth {
            undo_assignment(top.variable_index, formula);
            //continue;
        } else {
            formula.depth = depth;
            return None;
        }
        // undo all assigment where the depth is equal to the given depth and the assigment where forced
        /*if top.depth == depth && top.assigment_type == AssigmentType::Forced {
            undo_assignment(top.variable_index, formula);
            continue;
        }
        // if we reach the assigment where the depths are equal and the AssignmentType is Branching we are done backtracking.
        if top.depth == depth && top.assigment_type == AssigmentType::Branching {
            *gbd = depth;
            return None;
        }*/
    }
    debug!(target: "backtrack", "Backtrack finished");
    return Some(FormulaResultType::Unsatisfiable);
}

fn berk_mins_clause_deletion_strategies(formular: &mut Formula, threshold: u16) {
    if formular.original_clause_vector_length == formular.clauses.len() {
        debug!(target: "berk_mins_clause_deletion_strategies", "there a no learned clauses so there can not be removed any");
        return;
    }
    let diff = formular.clauses.len() - formular.original_clause_vector_length;
    // I am not sure if this is the right way to get the first 1/16 of all new learned clauses ???
    let first_sixteentel = diff / 16;
    // iterate over all new learned clauses
    let mut vec: Vec<usize> = Vec::with_capacity(diff);
    for (index, clause_index) in
    (formular.original_clause_vector_length..formular.clauses.len()).enumerate()
    {
        let clause = &formular.clauses[clause_index];
        if index <= first_sixteentel {
            // old
            if clause.activity <= threshold && clause.literals.len() > 8 {
                vec.push(clause_index);
            }
        } else {
            // jung
            if clause.activity <= 7 && clause.literals.len() > 42 {
                vec.push(clause_index);
            }
        }
    }
    for clause_index in vec {
        formular.delete_clauses(clause_index);
    }
}

fn unit_propagation(formula: &mut Formula) -> Option<FormulaResultType> {
    while let Some((unit, value, clause_index)) = formula.units.pop_front() {
        // Forced Assigment because of unit propagation !
        //let unit = formula.units.pop_front().unwrap();
        if formula.variables[unit].value != Value::Null {
            warn!(target: "unit_propagation", "Variable: {} is already set", unit);
            continue;
        }
        debug!(target: "unit_propagation", "Unit propagation: {}", unit);
        let result;
        match value {
            Value::True => {
                result = set_variable_true(
                    unit,
                    formula,
                    AssigmentType::Forced,
                    Some(clause_index),
                );
            }
            Value::False => {
                result = set_variable_false(
                    unit,
                    formula,
                    AssigmentType::Forced,
                    Some(clause_index),
                );
            }
            Value::Null => {
                panic!("i cannot set a unit to the value none in unit propagation")
            }
        }

        match result {
            SetResultType::Success => {
                continue;
            }
            SetResultType::Conflict { depth } => {
                if depth == 0 {
                    formula.result = FormulaResultType::Unsatisfiable;
                    return Some(FormulaResultType::Unsatisfiable);
                }
                match formula.heuristic_type {
                    HeuristicType::VSIDS => {
                        formula.vsids_score(unit);
                    }
                    _ => {}
                }

                debug!(target: "unit_propagation", "Unit propagation failed: {:?}", result);
                // after backtracking the unit queue should be empty. so we're exiting the loop automatically.
                match backtrack(formula, depth) {
                    None => {}
                    Some(result) => {
                        formula.result = result;
                        return Some(result);
                    }
                }
            }
        }
    }
    return None;
}

fn scan_for_units(formula: &mut Formula) {
    for (clause_index, clause) in formula.clauses.iter().enumerate() {
        if clause.watched.0 == clause.watched.1 {
            debug!(target: "scan_for_units", "unit found! {:?}", clause);
            let lit = clause.literals[clause.watched.0];
            let value = if lit > 0 { Value::True } else { Value::False };
            formula
                .units
                .push_back(((lit.abs() - 1) as usize, value, clause_index));
        }
    }
}

/// Eliminate pure literals
/// A pure literal is a variable that only occurs positive or negative in the formula.
/// If we find a pure literal we set the variable to the value that is needed to satisfy the formula.
fn pure_literal_elimination(formula: &mut Formula) {
    for index in 0..formula.variables.len() {
        let variable_index = formula.variables_index[index].0;
        let variable = &formula.variables[variable_index];
        if variable.value != Value::Null {
            continue;
        }
        match variable.is_pure() {
            Some(pure) => {
                debug!("Pure positive: {}", variable_index + 1);
                let value = match pure {
                    PureType::Positive => set_variable_true(variable_index + 1, formula, AssigmentType::Branching, None),
                    PureType::Negative => set_variable_false(variable_index + 1, formula, AssigmentType::Branching, None),
                };
                match value {
                    SetResultType::Success => {}
                    SetResultType::Conflict { depth } => {
                        warn!(target: "pure_literal_elimination", "formular unsat in depth: {}", depth);
                        formula.result = FormulaResultType::Unsatisfiable;
                        return;
                    }
                }
            }
            None => {}
        }
    }
}

pub fn dpll(formula: &mut Formula, timeout: Arc<AtomicBool>) {
    let mut index = 0;
    scan_for_units(formula);
    match unit_propagation(formula) {
        Some(_) => {
            return;
        }
        _ => {}
    }

    pure_literal_elimination(formula);
    if formula.result == FormulaResultType::Unsatisfiable {
        return;
    }

    loop {
        debug!(target: "dpll", "current index: {}", index);
        if index == formula.variables.len() {
            formula.result = FormulaResultType::Satisfiable;
            return;
        }
        debug!("current variables index: {:?}", formula.variables_index);
        let variable_index = formula.variables_index[index].0;

        debug!(target: "dpll", "current variable index: {}", variable_index);
        if formula.variables[variable_index].value != Value::Null {
            debug!(target: "dpll", "Variable: {} is already set", variable_index + 1);
            index += 1;
            continue;
        }
        debug!(target: "dpll", "Variable Value: {:?} ", formula.variables[variable_index]);
        if timeout.load(Ordering::SeqCst) {
            formula.result = FormulaResultType::Timeout;
            return;
        }
        // start by setting the first variable to true
        // Branching type because we decided freely to set this variable!
        // theoretically we can ignore the result is the set variable true here, because a conflict can only occur if
        // we set variables though unit propagation.
        match set_variable_true(variable_index, formula, AssigmentType::Branching, None) {
            SetResultType::Success => {}
            SetResultType::Conflict { depth } => {
                if depth == 0 {
                    formula.result = FormulaResultType::Unsatisfiable;
                    debug!(target: "dpll", "conflict depth is 0, {:?}", &formula.result);
                    return;
                }
                match backtrack(formula, depth) {
                    None => {}
                    Some(result) => {
                        formula.result = result;
                        debug!(target: "dpll","set_variable_true Backtrack failed: {:?}", &formula.result);
                        return;
                    }
                }
            }
        }
        //pure_literal_elimination(formula);
        //formula.update_score();

        index = 0;
        // propagate the units that have to be true now
        // propagate the units that have to be true now
        match unit_propagation(formula) {
            Some(_) => {
                return;
            }
            _ => {}
        }
    }
}

/*#[cfg(test)]
mod tests {
    use crate::dpll::dpll::{find_unit, set_variable_true};
    use crate::dpll::schemas::{
        AssigmentType, Clause, Formula, FormulaResultType, HeuristicType, ImplicationGraph, Value,
        Variable,
    };
    use std::collections::VecDeque;

    #[test]
    fn test_set_variable_true() {
        let variables = vec![
            Variable {
                value: Value::Null,
                positive_occurrences: vec![0, 1],
                negative_occurrences: vec![],
                num_of_unsolved_clauses_with_negative_occurrences: 0,
                num_of_unsolved_clauses_with_positive_occurrences: 0,
                score: 0.0,
            },
            Variable {
                value: Value::Null,
                positive_occurrences: vec![],
                negative_occurrences: vec![0],
                num_of_unsolved_clauses_with_negative_occurrences: 0,
                num_of_unsolved_clauses_with_positive_occurrences: 0,
                score: 0.0,
            },
        ];
        let mut formular = Formula {
            clauses: vec![
                Clause {
                    satisfiable: false,
                    satisfied_by_variable: 0,
                    literals: vec![1, -2],
                    number_of_active_literals: 2,
                },
                Clause {
                    satisfiable: false,
                    literals: vec![1],
                    satisfied_by_variable: 0,
                    number_of_active_literals: 1,
                },
            ],
            variables,
            units: VecDeque::new(),
            assigment_stack: vec![],
            number_of_unsatisfied_clauses: 0,
            result: FormulaResultType::Unknown,
            variables_index: vec![],
            heuristic_type: HeuristicType::None,
        };
        let mut implication_graph = ImplicationGraph {
            assignments: Vec::new(),
            edges: Vec::new(),
            conflict: None,
        };
        //todo is bd 1?
        set_variable_true(
            1,
            &mut formular,
            AssigmentType::Branching,
            1,
            &mut implication_graph,
        );
        assert_eq!(formular.variables[0].value, Value::True);
        assert_eq!(formular.variables[1].value, Value::Null);
        assert_eq!(formular.clauses[0].number_of_active_literals, 2);
        assert_eq!(formular.clauses[1].number_of_active_literals, 1);
        assert_eq!(formular.clauses[0].satisfiable, true);
        assert_eq!(formular.clauses[1].satisfiable, true);
    }

    #[test]
    fn test_find_unit_valid() {
        let variables_indexes: Vec<i16> = vec![1, 2, 3];
        let variables = vec![
            Variable {
                value: Value::True,
                positive_occurrences: vec![],
                negative_occurrences: vec![],
                num_of_unsolved_clauses_with_negative_occurrences: 0,
                num_of_unsolved_clauses_with_positive_occurrences: 0,
                score: 0.0,
            },
            Variable {
                value: Value::False,
                positive_occurrences: vec![],
                negative_occurrences: vec![],
                num_of_unsolved_clauses_with_negative_occurrences: 0,
                num_of_unsolved_clauses_with_positive_occurrences: 0,
                score: 0.0,
            },
            Variable {
                value: Value::Null,
                positive_occurrences: vec![],
                negative_occurrences: vec![],
                num_of_unsolved_clauses_with_negative_occurrences: 0,
                num_of_unsolved_clauses_with_positive_occurrences: 0,
                score: 0.0,
            },
        ];
        let formula = Formula {
            clauses: vec![
                Clause {
                    satisfiable: false,
                    satisfied_by_variable: 0,
                    literals: vec![1, -2, 3],
                    number_of_active_literals: 3,
                },
                Clause {
                    satisfiable: false,
                    satisfied_by_variable: 0,
                    literals: vec![1, 2, 3],
                    number_of_active_literals: 3,
                },
            ],
            variables,
            units: VecDeque::new(),
            assigment_stack: vec![],
            result: FormulaResultType::Unknown,
            number_of_unsatisfied_clauses: 0,
            variables_index: vec![],
            heuristic_type: HeuristicType::None,
        };
        let unit = find_unit(&variables_indexes, &formula.variables);
        assert_eq!(unit, 3);
    }

    #[test]
    fn test_find_unit_more_clauses() {
        let variables_indexes: Vec<i16> = vec![1, 2, 3];
        let variables = vec![
            Variable {
                value: Value::Null,
                positive_occurrences: vec![],
                negative_occurrences: vec![],
                num_of_unsolved_clauses_with_negative_occurrences: 0,
                num_of_unsolved_clauses_with_positive_occurrences: 0,
                score: 0.0,
            },
            Variable {
                value: Value::Null,
                positive_occurrences: vec![],
                negative_occurrences: vec![],
                num_of_unsolved_clauses_with_negative_occurrences: 0,
                num_of_unsolved_clauses_with_positive_occurrences: 0,
                score: 0.0,
            },
            Variable {
                value: Value::False,
                positive_occurrences: vec![],
                negative_occurrences: vec![],
                num_of_unsolved_clauses_with_negative_occurrences: 0,
                num_of_unsolved_clauses_with_positive_occurrences: 0,
                score: 0.0,
            },
        ];

        let formula = Formula {
            clauses: vec![
                Clause {
                    satisfiable: false,
                    satisfied_by_variable: 0,
                    literals: vec![1, -2, 3],
                    number_of_active_literals: 3,
                },
                Clause {
                    satisfiable: false,
                    satisfied_by_variable: 0,
                    literals: vec![1, 2, 3],
                    number_of_active_literals: 3,
                },
            ],
            variables,
            units: VecDeque::new(),
            assigment_stack: vec![],
            result: FormulaResultType::Unknown,
            number_of_unsatisfied_clauses: 0,
            variables_index: vec![],
            heuristic_type: HeuristicType::None,
        };

        let _ = find_unit(&variables_indexes, &formula.variables);
    }

    #[test]
    #[should_panic]
    fn test_find_unit_no_clauses() {
        let variables_indexes: Vec<i16> = vec![1, 2, 3];
        let variables = vec![
            Variable {
                value: Value::False,
                positive_occurrences: vec![],
                negative_occurrences: vec![],
                num_of_unsolved_clauses_with_negative_occurrences: 0,
                num_of_unsolved_clauses_with_positive_occurrences: 0,
                score: 0.0,
            },
            Variable {
                value: Value::False,
                positive_occurrences: vec![],
                negative_occurrences: vec![],
                num_of_unsolved_clauses_with_negative_occurrences: 0,
                num_of_unsolved_clauses_with_positive_occurrences: 0,
                score: 0.0,
            },
            Variable {
                value: Value::False,
                positive_occurrences: vec![],
                negative_occurrences: vec![],
                num_of_unsolved_clauses_with_negative_occurrences: 0,
                num_of_unsolved_clauses_with_positive_occurrences: 0,
                score: 0.0,
            },
        ];
        let formula = Formula {
            clauses: vec![],
            variables,
            units: VecDeque::new(),
            assigment_stack: vec![],
            result: FormulaResultType::Unknown,
            number_of_unsatisfied_clauses: 0,
            variables_index: vec![],
            heuristic_type: HeuristicType::None,
        };
        let _ = find_unit(&variables_indexes, &formula.variables);
    }
}
*/
