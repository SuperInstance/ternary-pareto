use crate::{Direction, Objective, ScoredStrategy, Strategy};

/// Determines dominance relationships between strategies.
#[derive(Debug, Clone)]
pub struct DominanceRelation {
    objectives: Vec<Objective>,
}

impl DominanceRelation {
    /// Create a new dominance relation with the given objectives.
    pub fn new(objectives: Vec<Objective>) -> Self {
        Self { objectives }
    }

    /// Get the objectives.
    pub fn objectives(&self) -> &[Objective] {
        &self.objectives
    }

    /// Returns true if `a` dominates `b`.
    ///
    /// Strategy a dominates strategy b if:
    /// - a is at least as good as b in all objectives
    /// - a is strictly better than b in at least one objective
    pub fn dominates(&self, a: &Strategy, b: &Strategy) -> bool {
        let mut at_least_one_better = false;

        for obj in &self.objectives {
            let va = match a.get(&obj.name) {
                Some(&v) => v,
                None => return false,
            };
            let vb = match b.get(&obj.name) {
                Some(&v) => v,
                None => return false,
            };

            match obj.direction {
                Direction::Minimize => {
                    if va > vb {
                        return false;
                    }
                    if va < vb {
                        at_least_one_better = true;
                    }
                }
                Direction::Maximize => {
                    if va < vb {
                        return false;
                    }
                    if va > vb {
                        at_least_one_better = true;
                    }
                }
            }
        }

        at_least_one_better
    }

    /// Returns the dominance relationship between two scored strategies.
    pub fn compare(&self, a: &ScoredStrategy, b: &ScoredStrategy) -> Dominance {
        if self.dominates(&a.values, &b.values) {
            Dominance::ADominatesB
        } else if self.dominates(&b.values, &a.values) {
            Dominance::BDominatesA
        } else {
            Dominance::Incomparable
        }
    }

    /// Returns true if two strategies are incomparable (neither dominates the other).
    pub fn is_incomparable(&self, a: &Strategy, b: &Strategy) -> bool {
        !self.dominates(a, b) && !self.dominates(b, a)
    }
}

/// Result of comparing two strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dominance {
    /// The first strategy dominates the second.
    ADominatesB,
    /// The second strategy dominates the first.
    BDominatesA,
    /// Neither dominates the other.
    Incomparable,
}
