use std::collections::{HashMap, HashSet};
use std::default::Default;

use bimap::BiMap;
use petgraph::graph::*;
use petgraph::dot::{Dot, Config};

use crate::logics::*;
use crate::ltl::*;
use crate::utils::*;

pub fn break_conjs(init: &FormulaTy) -> FormulaSet {
    let mut res = FormulaSet::new();
    match init {
        FormulaTy::And(lhs, rhs) => {
            let l = break_conjs(&*lhs);
            let r = break_conjs(&*rhs);
            res = l.union(&r);
        },
        _ => {
            res.insert(init.clone());
        }
    }
    res
}