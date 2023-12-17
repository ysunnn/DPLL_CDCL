use crate::schemas::{Formula, Variable, Value};

mod schemas;

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
        return current_index;
    }
    if counter > 1 {
        panic!("More than one unit found");
    }
    panic!("No unit found");
}

fn set_variable_true(variable: usize, formular: &mut Formula) {
    if variable == 0 {
        panic!("Variable cannot be 0");
    }
    let variable_index = variable - 1;
    formular.variables[variable_index].value = Value::True;
    for index in formular.variables[variable_index].negative_occurrences.iter() {
        let clause = &mut formular.clauses[(index - 1) as usize];
        // decrease the number of active literals in the clause
        clause.active_literals -= 1;
        if clause.active_literals == 1 {
            // Add units to the queue for propagation
            let x: Vec<usize> = clause.literals.iter().map(|x| x.abs() as usize).collect();
            formular.units.push_back(find_unit(x, &formular.variables));
        }
        if clause.active_literals == 0 {
            // set the clause to unsatisfiable
            // report conflict
        }
    }

    for index in formular.variables[variable_index].positive_occurrences.iter() {
        let clause = &mut formular.clauses[(index - 1) as usize];
        // set the clause to satisfied
        clause.satisfiable = true;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use crate::{find_unit, set_variable_true};
    use crate::schemas::{Clause, Formula, Value, Variable};

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
                    literals: vec![1, -2],
                    active_literals: 2,
                },
                Clause {
                    satisfiable: false,
                    literals: vec![1],
                    active_literals: 1,
                },
            ],
            variables,
            units: VecDeque::new(),
        };
        set_variable_true(1, &mut formular);
        assert_eq!(formular.variables[0].value, Value::True);
        assert_eq!(formular.variables[1].value, Value::Null);
        assert_eq!(formular.clauses[0].active_literals, 2);
        assert_eq!(formular.clauses[1].active_literals, 1);
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
        assert_eq!(unit, 2);
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
