use crate::dpll::schemas::{Formula, Value, Variable};

impl Variable {
    pub(crate) fn dlis(&self) -> f32 {
        (if self.positive_occurrences.len() > self.negative_occurrences.len() {
            self.positive_occurrences.len()
        } else {
            self.negative_occurrences.len()
        }) as f32
    }

    pub(crate) fn dlcs(&self) -> f32 {
        (self.positive_occurrences.len() + self.negative_occurrences.len()) as f32
    }
}

impl Formula {
    pub fn dlis(&mut self) {
        let mut variables_index = self
            .variables
            .iter()
            .enumerate()
            .map(|(index, var)| (index, var.dlis()))
            .collect::<Vec<(usize, f32)>>();
        variables_index.sort_by(|a, b| b.1.total_cmp(&a.1));
        self.variables_index = variables_index;
    }

    pub fn dlcs(&mut self) {
        let mut variables_index = self
            .variables
            .iter()
            .enumerate()
            .map(|(index, var)| (index, var.dlcs()))
            .collect::<Vec<(usize, f32)>>();
        variables_index.sort_by(|a, b| b.1.total_cmp(&a.1));
        self.variables_index = variables_index;
    }

    pub fn mom(&mut self) {
        let mut variables_index = self
            .variables
            .iter()
            .enumerate()
            .map(|(index, var)| (index, var.dlcs()))
            .collect::<Vec<(usize, f32)>>();
        variables_index.sort_by(|a, b| b.1.total_cmp(&a.1));
        variables_index.reverse();
        self.variables_index = variables_index;
    }

    pub fn jeroslow_wang_score(&mut self) {
        self.variables_index = self
            .variables
            .iter()
            .enumerate()
            .map(|(index, var)| {
                let score;
                if var.value == Value::Null {
                    score = f32::MIN;
                } else {
                    score = var
                        .positive_occurrences
                        .iter()
                        .chain(var.negative_occurrences.iter())
                        .map(|clause_index| {
                            2.0f32
                                .powi(
                                    -(self.clauses[*clause_index].number_of_active_literals as i32),
                                )
                                .round()
                        })
                        .sum::<f32>();
                }
                (index, score)
            })
            .collect();

        self.variables_index.sort_by(|a, b| b.1.total_cmp(&a.1));
    }

    pub fn vsids_score(&mut self, variables_index: usize) {
        let decay_factor: f32 = 0.95;

        let pos = &self.variables[variables_index].positive_occurrences.clone();
        let neg = &self.variables[variables_index].negative_occurrences.clone();

        for clause_index in pos {
            let lits: Vec<usize> = self.clauses[*clause_index]
                .literals
                .iter()
                .map(|x| (x.abs() - 1) as usize)
                .collect();
            for lit in lits {
                self.variables[lit].score += 1.0;
            }
        }

        for clause_index in neg {
            let lits: Vec<usize> = self.clauses[*clause_index]
                .literals
                .iter()
                .map(|x| (x.abs() - 1) as usize)
                .collect();
            for lit in lits {
                self.variables[lit].score += 1.0;
            }
        }

        for variable_index in 0..self.variables.len() {
            self.variables[variable_index].score *= decay_factor;
            self.variables_index[variable_index] =
                (variable_index, self.variables[variable_index].score)
        }
        self.variables_index.sort_by(|a, b| b.1.total_cmp(&a.1));
    }
}
