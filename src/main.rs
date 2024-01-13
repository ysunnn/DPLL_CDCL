#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use std::collections::VecDeque;
use log::{debug};
use crate::schemas::{Formula, Value, Assignment, AssigmentType, ResultType, Variable, Clause};


mod schemas;

/// Find the last not assign unit from a clause
///
/// [`variables_indexes`](Vec<usize>) is the list if variables from one clause.
/// the numbers in the list should start with one the zero is not used. to as close on the DIMACS format.
/// [`variables`](Vec<Variable>) is a list of all variables from the formula. this start with the index zero,
/// where zero ist mapped to one and so one.
/// the function panic's if there are more than one unset unit or if no unset units a found
fn find_unit(variables: &Vec<i16>, formula_variables: &Vec<Variable>) -> i16 {
    debug!(target: "find_unit", "Find unit for: {:?}", variables);
    match variables.iter()
        .find(|&&variable| formula_variables[(variable.abs() - 1) as usize].value == Value::Null) {
        None => panic!("No unit found"),
        Some(unit) => *unit,
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

fn set_variable(variable: usize, formula: &mut Formula, assigment: AssigmentType, value: Value) -> ResultType {
    debug!(target: "set_variable", "Set variable: {} to {:?} by: {:?}", variable, value, assigment);
    let variable_index = set_variable_and_return_index(variable, formula, value);

    formula.assigment_stack.push(Assignment {
        variable,
        assigment_type: assigment,
    });

    debug!(target: "set_variable", "{:?}", formula.variables[variable_index]);

    let (positive_occurrences, negative_occurrences) = match value {
        Value::True => (&formula.variables[variable_index].positive_occurrences, &formula.variables[variable_index].negative_occurrences),
        Value::False => (&formula.variables[variable_index].negative_occurrences, &formula.variables[variable_index].positive_occurrences),
        _ => panic!("Invalid value"),
    };

    let mut result = ResultType::Success;
    let mut clause;
    for &index in positive_occurrences { // first we set all the clauses where the variable occurs positive to satisfied
        // so if a variable appears positive and negative in a clause we set the clause to satisfied so we do not propagate the variable again
        clause = &mut formula.clauses[index];
        // set the clause to satisfied
        if clause.satisfiable {
            continue;
        }
        clause.satisfiable = true;
        clause.satisfied_by_variable = variable;
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
                formula.units.push_back(find_unit(&clause.literals, &formula.variables));
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
fn set_variable_true(variable: usize, formula: &mut Formula, assigment: AssigmentType) -> ResultType {
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
fn set_variable_false(variable: usize, formula: &mut Formula, assigment: AssigmentType) -> ResultType {
    set_variable(variable, formula, assigment, Value::False)
}

/// Undo the assigment of a variable for backtracking.
/// First wie set the variable to free. We update every clause where the variable occurrences positive and is sat though this
/// variable. We set the clause to not sat.
/// for every negative occurrences in a clause we update the number of active literals by one.
fn undo_assignment(variable: usize, formula: &mut Formula) {
    let variable_index = variable - 1;
    let variable_ref = &mut formula.variables[variable_index];

    let (positive_occurrences, negative_occurrences) = match variable_ref.value {
        Value::True => (&variable_ref.positive_occurrences, &variable_ref.negative_occurrences),
        Value::False => (&variable_ref.negative_occurrences, &variable_ref.positive_occurrences),
        _ => panic!("Invalid value, {:?}", variable_ref.value),
    };
    variable_ref.value = Value::Null;

    for &index in positive_occurrences {
        let clause = &mut formula.clauses[index];
        if clause.satisfied_by_variable == variable {
            clause.satisfiable = false;
            clause.satisfied_by_variable = 0;
        }
    }

    for &index in negative_occurrences {
        let clause = &mut formula.clauses[index];
        clause.number_of_active_literals += 1;
    }
}

/// Backtrack the forced assigment
///
/// First we undo all the Forced assignments, if the assignment stack get empty in this process the formula is unsat.
/// If we still got an Branched assigment we undo this as well empty our unit queue and set the variable to false.
/// than we going back to normal and start with unit propagation and regular assignments.
fn backtrack(formula: &mut Formula) -> Result<i32, ResultType> {

    let mut num_of_undones = 0;
    while let Some(top) = formula.assigment_stack.pop() {
        // Check the last element (the top of the stack)
        match top.assigment_type {
            AssigmentType::Branching =>{
                // unset the last branched variable
                undo_assignment(top.variable, formula);
                formula.units.clear();
                match set_variable_false(top.variable, formula, AssigmentType::Forced) {
                    ResultType::Success => {
                        debug!(target: "backtrack", "Unset variable: {}", top.variable);
                        return Ok(num_of_undones);
                    }
                    ResultType::Conflict => {
                        debug!(target: "backtrack", "Unset variable: {}", top.variable);
                        return Err(ResultType::Unsatisfiable);
                    }
                    _ => {panic!("Invalid result")}
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

    return Err(ResultType::Unsatisfiable);
}

fn dpll(formula: &mut Formula) -> ResultType {
    
    let mut variable_index = 0;
    while variable_index < formula.variables.len() {
        // TODO: there must be a better way
        debug!(target: "dpll", "current variable index: {}", variable_index);
        if formula.variables[variable_index].value != Value::Null {
            debug!(target: "dpll", "Variable: {} is already set", variable_index + 1);
            variable_index += 1;
            continue;
        }
        // start by setting the first variable to true
        // Branching type because we decided freely to set this variable!
        // theoretically can we ignore the result is the set variable true here, because a conflict can only occur if
        // we set variables though unit propagation.
        let f = set_variable_true(variable_index + 1, formula, AssigmentType::Branching);
        if f == ResultType::Conflict {
            match backtrack(formula) {
                Ok(back) => {
                    if back as usize > variable_index {
                        variable_index = 0;
                    } else {
                        variable_index -= back as usize;
                    }
                }
                Err(result) => {
                    //error!(target: "dpll", "Backtrack failed: {:?}", result);
                    return result;
                }
            }
        }

        variable_index += 1;
        // propagate the units that have to be true now
        while let Some(unit) =  formula.units.pop_front() {
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
            debug!(target: "dpll", "Unit propagation failed: {:?}", result);
            // after backtracking the unit queue should be empty. so we exiting the loop automatically.
            match backtrack(formula) {
                Ok(back) => {
                    if back as usize > variable_index {
                        variable_index = 0;
                    } else {
                        variable_index -= back as usize;
                    }
                }
                Err(result) => {
                    //error!(target: "dpll", "Backtrack failed: {:?}", result);
                    return result;
                }
            }
        }
    }
    for variable_index in 0..formula.variables.len() {
        let variable = &formula.variables[variable_index];
        debug!("{}: {:?} ", variable_index + 1, variable.value);
    }
    
    return ResultType::Satisfiable;
}

fn main() {
    #[cfg(feature = "dhat-heap")]
        let _profiler = dhat::Profiler::new_heap();
    // x = 1, a= 2, b = 3, c = 4, d = 5, e = 6, f = 7
    // -x or a
    // -x or b
    // -a or c
    // -b or -c or d
    // -c or -d or e or f
    // c or d or -e
    let mut formula = Formula {
        clauses: vec![
            // -x or a
            Clause {
                satisfiable: false,
                satisfied_by_variable: 0,
                literals: vec![-1, 2],
                number_of_active_literals: 2,
            },
            // -x or b
            Clause {
                satisfiable: false,
                satisfied_by_variable: 0,
                literals: vec![-1, 3],
                number_of_active_literals: 2,
            },
            // -a or c
            Clause {
                satisfiable: false,
                satisfied_by_variable: 0,
                literals: vec![-2, 4],
                number_of_active_literals: 2,
            },
            // -b or -c or d
            Clause {
                satisfiable: false,
                satisfied_by_variable: 0,
                literals: vec![-3, -4, 5],
                number_of_active_literals: 3,
            },
            // -c or -d or e or f
            Clause {
                satisfiable: false,
                satisfied_by_variable: 0,
                literals: vec![-4, -5, 6, 7],
                number_of_active_literals: 4,
            },
            // -c or -d or -e
            Clause {
                satisfiable: false,
                satisfied_by_variable: 0,
                literals: vec![-4, -5, -6],
                number_of_active_literals: 3,
            },
        ],
        variables: vec![
            // X
            Variable {
                value: Value::Null,
                positive_occurrences: vec![],
                negative_occurrences: vec![0, 1],
            },
            // A
            Variable {
                value: Value::Null,
                positive_occurrences: vec![0],
                negative_occurrences: vec![2],
            },
            // B
            Variable {
                value: Value::Null,
                positive_occurrences: vec![1],
                negative_occurrences: vec![3],
            },
            // C
            Variable {
                value: Value::Null,
                positive_occurrences: vec![2],
                negative_occurrences: vec![3, 4, 5],
            },
            // D
            Variable {
                value: Value::Null,
                positive_occurrences: vec![3],
                negative_occurrences: vec![4, 5],
            },
            // E
            Variable {
                value: Value::Null,
                positive_occurrences: vec![4],
                negative_occurrences: vec![5],
            },
            // F
            Variable {
                value: Value::Null,
                positive_occurrences: vec![4],
                negative_occurrences: vec![],
            },
        ],
        units: VecDeque::new(),
        assigment_stack: vec![],
    };
    // a, b ,c
    // -a or -b or c
    // a or -b or c
    // a or -b or -c
    /*let mut formula = Formula{
        clauses: vec![
            // -a or -b or c
            Clause{
                satisfiable: false,
                satisfied_by_variable: 0,
                literals: vec![-1, -2, -3],
                number_of_active_literals: 3,
            },
            // a or -b or -c
            Clause{
                satisfiable: false,
                satisfied_by_variable: 0,
                literals: vec![1, -2, -3],
                number_of_active_literals: 3,
            },
            // a or -b or -c
            Clause{
                satisfiable: false,
                satisfied_by_variable: 0,
                literals: vec![1, -2, -3],
                number_of_active_literals: 3,
            },
        ],
        variables: vec![
            // A
            Variable{
                value: Value::Null,
                positive_occurrences: vec![1, 2],
                negative_occurrences: vec![0],
            },
            // B
            Variable{
                value: Value::Null,
                positive_occurrences: vec![],
                negative_occurrences: vec![0, 1, 2],
            },
            // C
            Variable{
                value: Value::Null,
                positive_occurrences: vec![],
                negative_occurrences: vec![0, 1,2],
            },
        ],
        units: VecDeque::new(),
        assigment_stack: vec![],
    };*/
    dpll(&mut formula);
    for clause in formula.clauses.iter() {
        println!("Clause: {:?}", clause);
        if !clause.satisfiable {
            panic!("Unsat")
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
                positive_occurrences: vec![0, 1],
                negative_occurrences: vec![],
            },
            Variable {
                value: Value::Null,
                positive_occurrences: vec![],
                negative_occurrences: vec![0],
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
        let variables_indexes: Vec<i16> = vec![1, 2, 3];
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
        };
        let unit = find_unit(&variables_indexes, &formula.variables);
        assert_eq!(unit, 3);
    }

    #[test]
    fn test_find_unit_more_clauses() {
        let variables_indexes:Vec<i16>= vec![1, 2, 3];
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
        };

        let _ = find_unit(&variables_indexes, &formula.variables);
    }

    #[test]
    #[should_panic]
    fn test_find_unit_no_clauses() {
        let variables_indexes:Vec<i16> = vec![1, 2, 3];
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
        let formula = Formula {
            clauses: vec![],
            variables,
            units: VecDeque::new(),
            assigment_stack: vec![],
        };
        let _ = find_unit(&variables_indexes, &formula.variables);
    }
}
