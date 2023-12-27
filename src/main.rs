use crate::schemas::{Formula, Variable, Value, Assignment, AssigmentType, ResultType};

mod schemas;

/// Find the last not assign unit from a clause
///
/// [`variables_indexes`](Vec<usize>) is the list if variables from one clause.
/// the numbers in the list should start with one the zero is not used. to as close on the DIMACS format.
/// [`variables`](Vec<Variable>) is a list of all variables from the formula. this start with the index zero,
/// where zero ist mapped to one and so one.
/// the function panic's if there are more than one unset unit or if no unset units a found
fn find_unit(variables_indexes: Vec<usize>, variables: &Vec<Variable>) -> usize {
    // Find the unit in the clause that is not yet set
    let mut counter = 0;
    let mut current_index = 0;
    for mut index in variables_indexes {
        index -= 1;
        let variable = &variables[index];
        if variable.value != Value::Null {
            continue;
        }
        counter += 1;
        current_index = index;
    }
    if counter == 1 {
        return current_index + 1;
    }
    if counter > 1 {
        panic!("More than one unit found");
    }
    panic!("No unit found");
}

fn set_variable_and_return_index(variable: usize, formula: &mut Formula, value: Value) -> usize {
    if variable == 0 {
        panic!("Variable (index) cannot be 0");
    }
    let index = variable - 1;
    formula.variables[index].value = value;
    return index;
}

/// Set a Variable from the formula true.
///
/// [`variable`](usize) is the variable number/name (starts with 1)
/// [`formula`](Formula) is the complete formula we want to solve
///
/// The callstack is updated with the given assigment, all the clause were the variable a peres positives are set satisfiable.
/// For the negative occurrences the number of of active literals is reduced and if there is only one active literal in the
/// clause we add the variables to the unit queue for unit propagation.
/// TODO: report an conflict if there a 0 active literals in one clause, we need to backpropagation !
fn set_variable_true(variable: usize, formula: &mut Formula, assigment: AssigmentType) -> ResultType {
    let variable_index = set_variable_and_return_index(variable, formula, Value::True);

    formula.assigment_stack.push(Assignment {
        variable,
        value: Value::True,
        assigment_type: assigment,
    });

    for index in formula.variables[variable_index].negative_occurrences.iter() {
        let clause = &mut formula.clauses[(index - 1) as usize];
        // decrease the number of active literals in the clause
        clause.number_of_active_literals -= 1;
        if clause.number_of_active_literals == 1 {
            // Add units to the queue for propagation
            let x: Vec<usize> = clause.literals.iter().map(|x| x.abs() as usize).collect();
            formula.units.push_back(find_unit(x, &formula.variables));
        }
        if clause.number_of_active_literals == 0 {
            // set the clause to unsatisfiable
            // report conflict
            return ResultType::Conflict;
        }
    }

    for index in formula.variables[variable_index].positive_occurrences.iter() {
        let clause = &mut formula.clauses[(index - 1) as usize];
        // set the clause to satisfied
        clause.satisfiable = true;
    }
    return ResultType::Success;
}

/// Undo the assigment of a variable for backtracking.
/// First wie set the variable to free. We update every clause where the variable occurrences positive and is sat though this
/// variable. We set the clause to not sat.
/// for every negative occurrences in a clause we update the number of active literals by one.
fn undo_assigment(variable: usize, formula: &mut Formula,) {
    let variable_index = set_variable_and_return_index(variable, formula, Value::Null);

    for index in formula.variables[variable_index].negative_occurrences.iter() {
        // update the neq occurrences to have an active variable again.
        formula.clauses[*index as usize].number_of_active_literals += 1;
    }

    for index in formula.variables[variable_index].positive_occurrences.iter() {
        if formula.clauses[*index as usize].satisfied_by_variable == variable {
            formula.clauses[*index as usize].satisfiable = false;
        }
    }
}

/// Backtrack the forced assigment
///
/// First we undo all the Forced assignments, if the assignment stack get empty in this process the formula is unsat.
/// If we still got an Branched assigment we undo this as well empty our unit queue and set the variable to false.
/// than we going back to normal and start with unit propagation and regular assignments.
fn backtrack(formula: &mut Formula) -> i32 {
    while let Some(&top) = formula.assigment_stack.last() {
        // Check the last element (the top of the stack)
        if top.assigment_type == AssigmentType::Forced {
            // Pop the element if the condition is met
            formula.assigment_stack.pop();
            //undo_assigment(top.variable, formula);
        } else {
            // unset the last branched variable
            //undo_assigment(top.variable, formula);
            formula.units.clear();
            // set last unset variable as not and forced because that the only possible solution a this point.
            // return to unit propagation
            return 1;
        }
    }
    panic!("Unsat");
}

fn dpll(formula: &mut Formula) {
    for variable_index in 0..formula.variables.len() {
        // start by setting the first variable to true
        // Branching type because we decided freely to set this variable!
        // theoretically can we ignore the result is the set variable true here, because a conflict can only occur if
        // we set variables though unit propagation.
        set_variable_true(variable_index + 1, formula, AssigmentType::Branching);
        // propagate the units that have to be true now
        while !formula.units.is_empty() {
            // Forced Assigment because of unit propagation !
            let result = set_variable_true(formula.units.pop_front().unwrap(), formula, AssigmentType::Forced);
            if (result == ResultType::Success){
                continue;
            }
            // after backtracking the unit queue should be empty. so we exiting the loop automatically.
            backtrack(formula);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use crate::{find_unit, set_variable_true};
    use crate::schemas::{AssigmentType, Clause, Formula, Value, Variable};

    #[test]
    fn test_set_variable_true() {
        let variables = vec![
            Variable {
                value: Value::Null,
                positive_occurrences: vec![1, 2],
                negative_occurrences: vec![],
            },
            Variable {
                value: Value::Null,
                positive_occurrences: vec![],
                negative_occurrences: vec![1],
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
        let variables_indexes = vec![1, 2, 3];
        let variables = vec![
            Variable {
                value: Value::True,
                positive_occurrences: vec![],
                negative_occurrences: vec![],
            },
            Variable {
                value: Value::False,
                positive_occurrences: vec![],
                negative_occurrences: vec![],
            },
            Variable {
                value: Value::Null,
                positive_occurrences: vec![],
                negative_occurrences: vec![],
            },
        ];
        let unit = find_unit(variables_indexes, &variables);
        assert_eq!(unit, 3);
    }

    #[test]
    #[should_panic]
    fn test_find_unit_more_clauses() {
        let variables_indexes = vec![1, 2, 3];
        let variables = vec![
            Variable {
                value: Value::Null,
                positive_occurrences: vec![],
                negative_occurrences: vec![],
            },
            Variable {
                value: Value::Null,
                positive_occurrences: vec![],
                negative_occurrences: vec![],
            },
            Variable {
                value: Value::False,
                positive_occurrences: vec![],
                negative_occurrences: vec![],
            },
        ];
        let _ = find_unit(variables_indexes, &variables);
    }

    #[test]
    #[should_panic]
    fn test_find_unit_no_clauses() {
        let variables_indexes = vec![1, 2, 3];
        let variables = vec![
            Variable {
                value: Value::False,
                positive_occurrences: vec![],
                negative_occurrences: vec![],
            },
            Variable {
                value: Value::False,
                positive_occurrences: vec![],
                negative_occurrences: vec![],
            },
            Variable {
                value: Value::False,
                positive_occurrences: vec![],
                negative_occurrences: vec![],
            },
        ];
        let _ = find_unit(variables_indexes, &variables);
    }
}

fn main() {}
