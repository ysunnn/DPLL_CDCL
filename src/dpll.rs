use crate::schemas::{
    AssigmentType, Assignment, Formula, FormulaResultType, HeuristicType, PureType, ResultType,
    Value, Variable,
};
use log::{debug, error, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Find the last not assign unit from a clause
///
/// [`variables_indexes`](Vec<usize>) is the list if variables from one clause.
/// the numbers in the list should start with one the zero is not used. to as close on the DIMACS format.
/// [`variables`](Vec<Variable>) is a list of all variables from the formula. this start with the index zero,
/// where zero ist mapped to one and so one.
/// the function panic's if there are more than one unset unit or if no unset units a found
fn find_unit(variables: &Vec<i16>, formula_variables: &Vec<Variable>) -> i16 {
    debug!(target: "find_unit", "Find unit for: {:?}", variables);
    match variables
        .iter()
        .find(|&&variable| formula_variables[(variable.abs() - 1) as usize].value == Value::Null)
    {
        None => panic!("No unit found"),
        Some(unit) => {
            debug!(target: "find_unit", "Unit found {}", unit);
            *unit
        }
    }
}

fn set_variable_and_return_index(variable: usize, formula: &mut Formula, value: Value) -> usize {
    if variable == 0 {
        panic!("Variable (index) cannot be 0");
    }
    let index = variable - 1;
    formula.variables[index].value = value;
    return index;
}

fn set_variable(
    variable: usize,
    formula: &mut Formula,
    assigment: AssigmentType,
    value: Value,
) -> ResultType {
    debug!(target: "set_variable", "Set variable: {} to {:?} by: {:?}", variable, value, assigment);
    let variable_index = set_variable_and_return_index(variable, formula, value);

    formula.assigment_stack.push(Assignment {
        variable,
        assigment_type: assigment,
    });

    debug!(target: "set_variable", "{:?}", formula.variables[variable_index]);

    let (positive_occurrences, negative_occurrences) = match value {
        Value::True => (
            &formula.variables[variable_index].positive_occurrences,
            &formula.variables[variable_index].negative_occurrences,
        ),
        Value::False => (
            &formula.variables[variable_index].negative_occurrences,
            &formula.variables[variable_index].positive_occurrences,
        ),
        _ => panic!("Invalid value"),
    };

    let mut result = ResultType::Success;
    let mut clause;
    for &index in positive_occurrences {
        // first we set all the clauses where the variable occurs positive to satisfied
        // so if a variable appears positive and negative in a clause we set the clause to satisfied so we do not propagate the variable again
        clause = &mut formula.clauses[index];
        // set the clause to satisfied
        if clause.satisfiable {
            continue;
        }
        clause.satisfiable = true;
        clause.satisfied_by_variable = variable;
        formula.number_of_unsatisfied_clauses -= 1;
    }

    for &index in negative_occurrences {
        clause = &mut formula.clauses[index];
        clause.number_of_active_literals -= 1;

        if clause.satisfiable {
            continue;
        }
        match clause.number_of_active_literals {
            0 => {
                result = ResultType::Conflict;
            }
            1 => {
                formula
                    .units
                    .push_back(find_unit(&clause.literals, &formula.variables));
            }
            _ => {}
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
/// For the negative occurrences the number of of active literals is reduced and if there is only one active literal in the
/// clause we add the variables to the unit queue for unit propagation.
fn set_variable_true(
    variable: usize,
    formula: &mut Formula,
    assigment: AssigmentType,
) -> ResultType {
    set_variable(variable, formula, assigment, Value::True)
}

/// Set a Variable from the formula false.
///
/// [`variable`](usize) is the variable number/name (starts with 1)
/// [`formula`](Formula) is the complete formula we want to solve
///
/// The callstack is updated with the given assigment, all the clause were the variable a peres positives are set satisfiable.
/// For the negative occurrences the number of of active literals is reduced and if there is only one active literal in the
/// clause we add the variables to the unit queue for unit propagation.
fn set_variable_false(
    variable: usize,
    formula: &mut Formula,
    assigment: AssigmentType,
) -> ResultType {
    set_variable(variable, formula, assigment, Value::False)
}

/// Undo the assigment of a variable for backtracking.
/// First wie set the variable to free. We update every clause where the variable occurrences positive and is sat though this
/// variable. We set the clause to not sat.
/// for every negative occurrences in a clause we update the number of active literals by one.
fn undo_assignment(variable: usize, formula: &mut Formula) -> Value {
    let variable_index = variable - 1;
    let variable_ref = &mut formula.variables[variable_index];
    let old_value = variable_ref.value.clone();

    let (positive_occurrences, negative_occurrences) = match variable_ref.value {
        Value::True => (
            &variable_ref.positive_occurrences,
            &variable_ref.negative_occurrences,
        ),
        Value::False => (
            &variable_ref.negative_occurrences,
            &variable_ref.positive_occurrences,
        ),
        _ => panic!("Invalid value, {:?}", variable_ref.value),
    };
    variable_ref.value = Value::Null;

    for &index in positive_occurrences {
        let clause = &mut formula.clauses[index];
        if clause.satisfied_by_variable == variable {
            clause.satisfiable = false;
            clause.satisfied_by_variable = 0;
            formula.number_of_unsatisfied_clauses += 1;
        }
    }

    for &index in negative_occurrences {
        let clause = &mut formula.clauses[index];
        clause.number_of_active_literals += 1;
    }
    return old_value;
}

/// Backtrack the forced assigment
///
/// First we undo all the Forced assignments, if the assignment stack get empty in this process the formula is unsat.
/// If we still got an Branched assigment we undo this as well empty our unit queue and set the variable to false.
/// than we going back to normal and start with unit propagation and regular assignments.
fn backtrack(formula: &mut Formula) -> Result<i32, FormulaResultType> {
    let mut num_of_undones = 0;
    while let Some(top) = formula.assigment_stack.pop() {
        // Check the last element (the top of the stack)
        match top.assigment_type {
            AssigmentType::Branching => {
                // unset the last branched variable
                let old_value = undo_assignment(top.variable, formula);
                formula.units.clear();
                let value = match old_value {
                    Value::True => Value::False,
                    Value::False => Value::True,
                    _ => panic!("Invalid value"),
                };
                match set_variable(top.variable, formula, AssigmentType::Forced, value) {
                    ResultType::Success => {
                        debug!(target: "backtrack", "Unset Success variable: {}", top.variable);
                        return Ok(num_of_undones);
                    }
                    ResultType::Conflict => {
                        debug!(target: "backtrack", "Unset Conflict variable: {}", top.variable);
                        if formula.assigment_stack.is_empty() {
                            return Err(FormulaResultType::Unsatisfiable);
                        }
                        match formula.heuristic_type {
                            HeuristicType::VSIDS => {
                                info!("{:?}", formula.heuristic_type);
                                formula.vsids_score(top.variable - 1)
                            }
                            _ => {}
                        }
                    }
                }
            }
            AssigmentType::Forced => {
                // Pop the element if the condition is met
                debug!(target: "backtrack", "Undo assigment: {:?}", top);
                undo_assignment(top.variable, formula);
                debug!(target: "backtrack", "Assigment undone: {:?}", formula.variables[top.variable - 1]);
                num_of_undones += 1;
            }
        }
    }
    info!(target: "backtrack", "Backtrack finished");
    return Err(FormulaResultType::Unsatisfiable);
}

fn scan_for_units(formula: &mut Formula) {
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
        match variable.is_pure() {
            Some(pure) => {
                debug!("Pure positive: {}", variable_index + 1);
                let value = match pure {
                    PureType::Positive => Value::True,
                    PureType::Negative => Value::False,
                };
                match set_variable(variable_index + 1, formula, AssigmentType::Branching, value) {
                    ResultType::Success => {}
                    ResultType::Conflict => {
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
    pure_literal_elimination(formula);
    if formula.result == FormulaResultType::Unsatisfiable {
        return;
    }

    loop {
        if formula.is_solved() {
            formula.result = FormulaResultType::Satisfiable;
            return;
        }

        let variable_index = formula.variables_index[index].0;

        debug!(target: "dpll", "current variable index: {}", variable_index);
        if formula.variables[variable_index].value != Value::Null {
            debug!(target: "dpll", "Variable: {} is already set", variable_index + 1);
            index += 1;
            continue;
        }

        if timeout.load(Ordering::SeqCst) {
            formula.result = FormulaResultType::Timeout;
            return;
        }
        // start by setting the first variable to true
        // Branching type because we decided freely to set this variable!
        // theoretically can we ignore the result is the set variable true here, because a conflict can only occur if
        // we set variables though unit propagation.
        if set_variable_true(variable_index + 1, formula, AssigmentType::Branching)
            == ResultType::Conflict
        {
            // we should never get here
            match backtrack(formula) {
                Ok(_) => {}
                Err(result) => {
                    formula.result = result;
                    error!("set_variable_true Backtrack failed: {:?}", &formula.result);
                    return;
                }
            }
        }

        index += 1;
        // propagate the units that have to be true now
        // propagate the units that have to be true now
        while let Some(unit) = formula.units.pop_front() {
            // Forced Assigment because of unit propagation !
            //let unit = formula.units.pop_front().unwrap();
            if formula.variables[(unit.abs() - 1) as usize].value != Value::Null {
                debug!(target: "dpll", "Variable: {} is already set", variable_index + 1);
                continue;
            }
            debug!(target: "dpll", "Unit propagation: {}", unit);
            let result;
            if unit > 0 {
                result = set_variable_true(unit.abs() as usize, formula, AssigmentType::Forced);
            } else {
                result = set_variable_false(unit.abs() as usize, formula, AssigmentType::Forced);
            }

            if result == ResultType::Success {
                continue;
            }
            match formula.heuristic_type {
                HeuristicType::VSIDS => {
                    formula.vsids_score((unit.abs() - 1) as usize);
                }
                _ => {}
            }

            debug!(target: "dpll", "Unit propagation failed: {:?}", result);
            // after backtracking the unit queue should be empty. so we exiting the loop automatically.
            match backtrack(formula) {
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
    use crate::dpll::{find_unit, set_variable_true};
    use crate::schemas::{
        AssigmentType, Clause, Formula, FormulaResultType, HeuristicType, Value, Variable,
    };
    use std::collections::VecDeque;

    #[test]
    fn test_set_variable_true() {
        let variables = vec![
            Variable {
                value: Value::Null,
                positive_occurrences: vec![0, 1],
                negative_occurrences: vec![],
                score: 0.0,
            },
            Variable {
                value: Value::Null,
                positive_occurrences: vec![],
                negative_occurrences: vec![0],
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
        set_variable_true(1, &mut formular, AssigmentType::Branching);
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
                score: 0.0,
            },
            Variable {
                value: Value::False,
                positive_occurrences: vec![],
                negative_occurrences: vec![],
                score: 0.0,
            },
            Variable {
                value: Value::Null,
                positive_occurrences: vec![],
                negative_occurrences: vec![],
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
                score: 0.0,
            },
            Variable {
                value: Value::Null,
                positive_occurrences: vec![],
                negative_occurrences: vec![],
                score: 0.0,
            },
            Variable {
                value: Value::False,
                positive_occurrences: vec![],
                negative_occurrences: vec![],
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
                score: 0.0,
            },
            Variable {
                value: Value::False,
                positive_occurrences: vec![],
                negative_occurrences: vec![],
                score: 0.0,
            },
            Variable {
                value: Value::False,
                positive_occurrences: vec![],
                negative_occurrences: vec![],
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
