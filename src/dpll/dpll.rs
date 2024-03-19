use std::collections::HashMap;
use crate::dpll::schemas::{
    AssigmentType, Assignment, Formula, FormulaResultType, HeuristicType, SetResultType,
    Value, Variable, ImplicationGraph,
    AssigmentType, Assignment, Formula, HeuristicType, SetResultType, Value,
};
use log::debug;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn set_variable_true(
    variable_index: usize,
    formula: &mut Formula,
    assigment_type: AssigmentType,
    bd: usize,
    implication_graph: & mut ImplicationGraph,
)-> SetResultType{
    debug!(target: "set_variable_true", "Set variable true: {} by: {:?}, current depth: {}", variable_index, assigment_type, bd);
    formula.variables[variable_index].value = Value::True;
    formula.variables[variable_index].depth = bd;
    let assignment = Assignment {
        variable_index,
        assigment_type,
        value: Value::True,
        depth: bd,
    };
    formula.assigment_stack_push(assignment);
    match assigment_type {
        AssigmentType::Branching => implication_graph.update_graph_for_branching(assignment),
        AssigmentType::Forced => implication_graph.update_graph_for_unit_propagation(formula,assignment),
        _ => {}
    }
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
                //warn!(target: "set_variable_true","conflict fore clause: {:?} index: {}", formula.clauses[*clause_index], clause_index);
                result = SetResultType::Conflict;
                // Update clauses activity for BerkMin's
                formula.clauses[*clause_index].activity += 1;
                implication_graph.create_conflict_vertex(formula, variable_index, bd);
            }
        }
    }
    return result;
}

fn set_variable_false(
    variable_index: usize,
    formula: &mut Formula,
    assigment_type: AssigmentType,
    bd: usize,
    implication_graph: & mut ImplicationGraph,
)-> SetResultType{
    debug!(target: "set_variable_false", "Set variable false: {} by: {:?}, current depth: {}", variable_index, assigment_type, bd);
    formula.variables[variable_index].value = Value::False;
    formula.variables[variable_index].depth = bd;
    let assignment = Assignment {
        variable_index,
        assigment_type,
        value: Value::True,
        depth: bd,
    };
    formula.assigment_stack_push(assignment);
    match assigment_type {
        AssigmentType::Branching => implication_graph.update_graph_for_branching(assignment),
        AssigmentType::Forced => implication_graph.update_graph_for_unit_propagation(formula,assignment),
        _ => {}
    }
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
                //warn!(target: "set_variable_false","conflict fore clause: {:?} index: {}", formula.clauses[*clause_index], clause_index);
                result = SetResultType::Conflict;
                implication_graph.create_conflict_vertex(formula, variable_index, bd);
                // Update clauses activity for BerkMin's
                formula.clauses[*clause_index].activity += 1;
            }
        }
    }
    return result;
}

/// Set a Variable from the formula true.
///
/// [`variable`](usize) is the variable number/name (starts with 1)
/// [`formula`](Formula) is the complete formula we want to solve
///
/// The callstack is updated with the given assigment, all the clause were the variable a peres positives are set satisfiable.
/// For the negative occurrences the number of active literals is reduced and if there is only one active literal in the
/// clause we add the variables to the unit queue for unit propagation.
/*fn set_variable_true(
    variable: usize,
    formula: &mut Formula,
    assigment: AssigmentType,
    bd: usize,
) -> SetResultType {
    set_variable(variable, formula, assigment, Value::True)
}*/

/// Set a Variable from the formula false.
///
/// [`variable`](usize) is the variable number/name (starts with 1)
/// [`formula`](Formula) is the complete formula we want to solve
///
/// The callstack is updated with the given assigment, all the clause were the variable a peres positives are set satisfiable.
/// For the negative occurrences the number of active literals is reduced and if there is only one active literal in the
/// clause we add the variables to the unit queue for unit propagation.
/*fn set_variable_false(
    variable: usize,
    formula: &mut Formula,
    assigment: AssigmentType,
    bd: usize,
) -> SetResultType {
    set_variable(variable, formula, assigment, Value::False)
}*/

/// Undo the assigment of a variable for backtracking.
/// First wie set the variable to free. We update every clause where the variable occurrences positive and is sat though this
/// variable. We set the clause to not sat.
/// for every negative occurrences in a clause we update the number of active literals by one.
fn undo_assignment(variable_index: usize, formula: &mut Formula) {
    formula.variables[variable_index].value = Value::Null;
    // somewhere here we have to check the number of assigned variables for the clauses to delete
    // it if it's a learned one with a length greater than k and less than m literates are assigned.
    // what do we do when we have to remove it from the assigment stack?
    // maybe only remove variables after backtracking?
}

/// Backtrack the forced assigment
///
/// First we undo all the Forced assignments, if the assignment stack get empty in this process the formula is unsat.
/// If we still got a Branched assigment we undo this as well empty our unit queue and set the variable to false.
/// Then we're going back to normal and start with unit propagation and regular assignments.
fn backtrack(formula: &mut Formula, gbd: &mut usize, implication_graph: &mut ImplicationGraph) -> Result<i32, FormulaResultType> {
    let mut numb_of_undone = 0;
    while let Some(top) = formula.assigment_stack_pop() {
        // Check the last element (the top of the stack)
        match top.assigment_type {
            AssigmentType::Branching => {
                *gbd -= 1;
                // unset the last branched variable
                undo_assignment(top.variable_index, formula);
                formula.units.clear();
                let result;
                match top.value {
                    Value::True => {
                        result = set_variable_false(top.variable_index, formula, AssigmentType::Forced, *gbd, implication_graph)
                    },
                    Value::False => {
                        result = set_variable_true(top.variable_index, formula, AssigmentType::Forced, *gbd, implication_graph)
                    },
                    _ => panic!("Invalid value"),
                };
                match result {
                    SetResultType::Success => {
                        debug!(target: "backtrack", "Unset Success variable: {}", top.variable_index);
                        //formula.update_score();
                        return Ok(numb_of_undone);
                    }
                    SetResultType::Conflict => {
                        debug!(target: "backtrack", "Unset Conflict variable: {}", top.variable_index);
                        if formula.assigment_stack_is_empty() {
                            return Err(FormulaResultType::Unsatisfiable);
                        }
                        implication_graph.create_conflict_vertex(formula, top.variable_index, top.depth);
                        match formula.heuristic_type {
                            HeuristicType::VSIDS => {
                                debug!("{:?}", formula.heuristic_type);
                                formula.vsids_score(top.variable_index);
                            }
                            _ => {}
                        }
                    }
                }
            }
            AssigmentType::Forced => {
                // Pop the element if the condition is met
                debug!(target: "backtrack", "Undo assigment: {:?}", top);
                undo_assignment(top.variable_index, formula);
                debug!(target: "backtrack", "Assigment undone: {:?}", formula.variables[top.variable_index]);
                numb_of_undone += 1;
            }
        }
    }
    debug!(target: "backtrack", "Backtrack finished");
    return Err(FormulaResultType::Unsatisfiable);
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

/*fn scan_for_units(formula: &mut Formula) {
    for clause in formula.clauses.iter() {
        if clause.number_of_active_literals == 1 {
            formula
                .units
                .push_back(find_unit(&clause.literals, &formula.variables));
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
                    PureType::Positive => Value::True,
                    PureType::Negative => Value::False,
                };
                match set_variable(variable_index + 1, formula, AssigmentType::Branching, value) {
                    SetResultType::Success => {}
                    SetResultType::Conflict => {
                        formula.result = FormulaResultType::Unsatisfiable;
                        return;
                    }
                }
            }
            None => {}
        }
    }
}*/

pub fn dpll(formula: &mut Formula, timeout: Arc<AtomicBool>) {
    let mut index = 0;
    // Global branching depth counter
    let mut gbd: usize = 0;

    let mut implication_graph = ImplicationGraph {
        assignments: HashMap::new(),
        edges: HashMap::new(),
        conflict: None,
    };

    //scan_for_units(formula);
    //pure_literal_elimination(formula);

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
        gbd += 1;
        if set_variable_true(variable_index, formula, AssigmentType::Branching, gbd, &mut implication_graph)
            == SetResultType::Conflict
        {
            // we should never get here
            implication_graph.create_conflict_vertex(formula, variable_index, gbd);
            match backtrack(formula, &mut gbd, &mut implication_graph) {
                Ok(_) => {}
                Err(result) => {
                    formula.result = result;
                    debug!("set_variable_true Backtrack failed: {:?}", &formula.result);
                    return;
                }
            }
        }
        //pure_literal_elimination(formula);
        //formula.update_score();

        index = 0;
        // propagate the units that have to be true now
        // propagate the units that have to be true now
        while let Some((unit, value)) = formula.units.pop_front() {
            // Forced Assigment because of unit propagation !
            //let unit = formula.units.pop_front().unwrap();
            if formula.variables[unit].value != Value::Null {
                debug!(target: "dpll", "Variable: {} is already set", variable_index + 1);
                continue;
            }
            debug!(target: "dpll", "Unit propagation: {}", unit);
            let result;
            match value {
                Value::True =>{
                    result = set_variable_true(unit, formula, AssigmentType::Forced, gbd, &mut implication_graph);
                }
                Value::False =>{
                    result = set_variable_false(unit, formula, AssigmentType::Forced, gbd, &mut implication_graph);
                }
                Value::Null =>{
                    panic!("i cannot set a unit to the value none in unit propagation")
                }
            }
            //pure_literal_elimination(formula);
            //formula.update_score();
            if result == SetResultType::Success {
                continue;
            }
            match formula.heuristic_type {
                HeuristicType::VSIDS => {
                    formula.vsids_score(unit);
                }
                _ => {}
            }

            debug!(target: "dpll", "Unit propagation failed: {:?}", result);
            // after backtracking the unit queue should be empty. so we're exiting the loop automatically.
            match backtrack(formula, &mut gbd, &mut implication_graph) {
                Ok(_) => {
                    index = 0;
                }
                Err(result) => {
                    formula.result = result;
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dpll::dpll::{find_unit, set_variable_true};
    use crate::dpll::schemas::{
        AssigmentType, Clause, Formula, FormulaResultType, HeuristicType, Value, Variable, ImplicationGraph,
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
        set_variable_true(1, &mut formular, AssigmentType::Branching, 1, &mut implication_graph);
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
