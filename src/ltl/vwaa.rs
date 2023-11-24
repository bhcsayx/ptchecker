use std::collections::{HashMap, HashSet};
use std::default::Default;

use bimap::BiMap;
use petgraph::graph::*;
use petgraph::dot::{Dot, Config};

use crate::logics::*;
use crate::ltl::*;
use crate::utils::*;

pub fn vwaa_bar(f: FormulaTy) -> FormulaSet {
    let mut res = FormulaSet::new();
    match f {
        FormulaTy::Or(lhs, rhs) => {
            let mut lset = vwaa_bar(*lhs.clone());
            let mut rset = vwaa_bar(*rhs.clone());
            res = lset.union(&rset);
        },
        FormulaTy::And(lhs, rhs) => {
            let lset = vwaa_bar(*lhs.clone());
            let rset = vwaa_bar(*rhs.clone());
            for l in lset.set.iter() {
                for r in rset.set.iter() {
                    res.insert(FormulaTy::And(Box::new(l.clone()), Box::new(r.clone())));
                }
            }
        }
        _ => {
            res.insert(f.clone());
        }
    }
    res
}

pub fn vwaa_product(lhs: HashSet<(FormulaSet, FormulaTy)>,
    rhs: HashSet<(FormulaSet, FormulaTy)>) -> HashSet<(FormulaSet, FormulaTy)> {
    let mut res = HashSet::new();
    for (l_key, l_val) in lhs.iter() {
        for (r_key, r_val) in rhs.iter() {
            let key = l_key.union(&r_key);
            let val = if *l_val == FormulaTy::True {
                r_val.clone()
            } else if *r_val == FormulaTy::True {
                l_val.clone()
            } else {
                FormulaTy::And(Box::new(l_val.clone()), Box::new(r_val.clone()))
            };
            res.insert((key, val));
        }
    }
    res
}

pub fn vwaa_cap_delta(f: FormulaTy) -> HashSet<(FormulaSet, FormulaTy)> {
    match f {
        FormulaTy::Or(lhs, rhs) => {
            return vwaa_cap_delta(*lhs.clone()).union(&vwaa_cap_delta(*rhs.clone())).map(|i| i.clone()).collect();
        }
        FormulaTy::And(lhs, rhs) => {
            return vwaa_product(vwaa_cap_delta(*lhs.clone()), vwaa_cap_delta(*rhs.clone()));
        }
        _ => {
            return vwaa_delta(f);
        }
    }
}

pub fn vwaa_delta(f: FormulaTy) -> HashSet<(FormulaSet, FormulaTy)> {
    // Returns transition set where element (sigma, q), sigma is actually set of letters that include formula sigma
    let mut res = HashSet::new();
    let tt = FormulaSet::from_iter(vec![FormulaTy::True]);
    let p = FormulaSet::from_iter(vec![f.clone()]);

    match f.clone() {
        FormulaTy::True => {
            res.insert((tt.clone(), FormulaTy::True));
            return res;
        },
        FormulaTy::False => {
            return res;
        },
        FormulaTy::Prop(atom) => {
            res.insert((p.clone(), FormulaTy::True));
            return res;
        },
        FormulaTy::Neg(atom) => {
            res.insert((p.clone(), FormulaTy::True));
            return res;
        },
        FormulaTy::Next(inner) => {
            let bar = vwaa_bar(*inner.clone());
            for b in bar.set.iter() {
                res.insert((tt.clone(), b.clone()));
            }
            return res;
        },
        FormulaTy::Until(lhs, rhs) => {
            res.insert((tt.clone(), f.clone()));
            let res_left = vwaa_cap_delta(*rhs.clone());
            let res_right = vwaa_product(vwaa_cap_delta(*lhs.clone()), res);
            return res_left.union(&res_right).map(|i| i.clone()).collect();
        },
        FormulaTy::Release(lhs, rhs) => {
            res.insert((tt.clone(), f.clone()));
            let res_left = vwaa_cap_delta(*rhs.clone());
            let res_right = vwaa_cap_delta(*lhs.clone()).union(&res).map(|i| i.clone()).collect();
            return vwaa_product(res_left, res_right);
        },
        FormulaTy::Or(lhs, rhs) => {
            return vwaa_cap_delta(f.clone());
        }
        FormulaTy::And(lhs, rhs) => {
            return vwaa_cap_delta(f.clone());
        } 
        _ => {return res;},
    }
}