use crate::schemas::{Formula, Variable, Value};

mod schemas;

fn find_unit(variables_indexes: Vec<usize>, variables: &Vec<Variable>) -> usize {
    // Find the unit in the clause that is not yet set
    let mut counter = 0;
    let mut current_index = 0;
    for index in variables_indexes {
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
fn set_variable_true(variable: &mut Variable, formular: &mut Formula) {
    variable.value = Value::True;
    for index in variable.negative_occurrences.iter() {
        let clause = &mut formular.clauses[*index as usize];
        // decrease the number of active literals in the clause
        clause.active_literals -= 1;
        if clause.active_literals == 1 {
            // Add units to the queue for propagation
            let x: Vec<usize> = clause.literals.iter().map(|x| x.abs() as usize).collect();
            formular.units.push_back(find_unit(x, &formular.variables));
        }
    }
    for index in variable.positive_occurrences.iter() {
        let clause = &mut formular.clauses[*index as usize];
        // set the clause to satisfied
        clause.satisfiable = true;
    }
}

fn main() {
}
