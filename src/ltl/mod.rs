use crate::logics::*;
use crate::petri::*;

use petgraph::graph::*;
use petgraph::dot::{Dot, Config};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;

pub mod automaton;

use crate::ltl::automaton::*;

pub fn ltl_subf_recur(set: &mut FormulaSet, input: FormulaTy) {
    if let FormulaTy::Forall(inner) = input.clone() {}
    else {
        set.insert(input.clone());
    }
    match input {
        FormulaTy::True | FormulaTy::False => {},
        FormulaTy::Prop(atom) => {},
        FormulaTy::Neg(atom) => {
            set.insert(FormulaTy::Prop(atom.clone()));
        },
        FormulaTy::Not(inner) => {
            ltl_subf_recur(set, *inner.clone());
        },
        FormulaTy::Or(lhs, rhs) => {
            ltl_subf_recur(set, *lhs.clone());
            ltl_subf_recur(set, *rhs.clone());
        },
        FormulaTy::And(lhs, rhs) => {
            ltl_subf_recur(set, *lhs.clone());
            ltl_subf_recur(set, *rhs.clone());
        },
        FormulaTy::Next(inner) => {
            ltl_subf_recur(set, *inner.clone());
        },
        FormulaTy::Until(lhs, rhs) => {
            ltl_subf_recur(set, *lhs.clone());
            ltl_subf_recur(set, *rhs.clone());
        },
        FormulaTy::Release(lhs, rhs) => {
            ltl_subf_recur(set, *lhs.clone());
            ltl_subf_recur(set, *rhs.clone());
        },
        FormulaTy::Forall(inner) => {
            ltl_subf_recur(set, *inner.clone());
        },
        _ => {},
    }
}

pub fn ltl_subformulas(input: FormulaTy) -> FormulaSet {
    if let FormulaTy::Forall(inner) = input {
        let mut set = HashSet::new();
        ltl_subf_recur(&mut set, *inner.clone());
        return set;
    }
    else {
        println!("Malformed LTL formula");
        return HashSet::new();
    }
}

pub fn ltl_maximal_csubsets(closure: FormulaSet) -> HashSet<FormulaSet> {
    let mut res = Vec::new();

}

pub fn ltl_negate(input: FormulaTy) -> FormulaTy {
    // Negate an LTL formula
    match input {
        FormulaTy::True => FormulaTy::False,
        FormulaTy::False => FormulaTy::True,
        FormulaTy::Prop(atom) => FormulaTy::Neg(atom.clone()),
        FormulaTy::Neg(atom) => FormulaTy::Prop(atom.clone()),
        FormulaTy::Not(inner) => *inner.clone(),
        FormulaTy::Or(lhs, rhs) => FormulaTy::And(
            Box::new(ltl_negate(*lhs.clone())),
            Box::new(ltl_negate(*rhs.clone())),
        ),
        FormulaTy::And(lhs, rhs) => FormulaTy::Or(
            Box::new(ltl_negate(*lhs.clone())),
            Box::new(ltl_negate(*rhs.clone())),
        ),
        FormulaTy::Next(inner) => FormulaTy::Next(Box::new(ltl_negate(*inner.clone()))),
        FormulaTy::Global(inner) => FormulaTy::Finally(Box::new(ltl_negate(*inner.clone()))),
        FormulaTy::Finally(inner) => FormulaTy::Global(Box::new(ltl_negate(*inner.clone()))),
        FormulaTy::Until(lhs, rhs) => FormulaTy::Release(
            Box::new(ltl_negate(*lhs.clone())),
            Box::new(ltl_negate(*rhs.clone())),
        ),
        FormulaTy::Release(lhs, rhs) => FormulaTy::Until(
            Box::new(ltl_negate(*lhs.clone())),
            Box::new(ltl_negate(*rhs.clone())),
        ),
        FormulaTy::Forall(inner) => FormulaTy::Forall(Box::new(ltl_negate(*inner.clone()))),
        _ => input.clone(),
    }
}

pub fn ltl_simplify(input: FormulaTy) -> FormulaTy {
    match input {
        FormulaTy::Global(inner) => FormulaTy::Release(
            Box::new(FormulaTy::False),
            Box::new(ltl_simplify(*inner.clone())),
        ),
        FormulaTy::Finally(inner) => FormulaTy::Until(
            Box::new(FormulaTy::True),
            Box::new(ltl_simplify(*inner.clone())),
        ),
        FormulaTy::Not(inner) => FormulaTy::Not(Box::new(*inner.clone())),
        FormulaTy::Or(lhs, rhs) => FormulaTy::Or(
            Box::new(ltl_simplify(*lhs.clone())),
            Box::new(ltl_simplify(*rhs.clone())),
        ),
        FormulaTy::And(lhs, rhs) => FormulaTy::And(
            Box::new(ltl_simplify(*lhs.clone())),
            Box::new(ltl_simplify(*rhs.clone())),
        ),
        FormulaTy::Next(inner) => FormulaTy::Next(Box::new(ltl_simplify(*inner.clone()))),
        FormulaTy::Until(lhs, rhs) => FormulaTy::Until(
            Box::new(ltl_simplify(*lhs.clone())),
            Box::new(ltl_simplify(*rhs.clone())),
        ),
        FormulaTy::Release(lhs, rhs) => FormulaTy::Release(
            Box::new(ltl_simplify(*lhs.clone())),
            Box::new(ltl_simplify(*rhs.clone())),
        ),
        FormulaTy::Forall(inner) => FormulaTy::Forall(Box::new(ltl_simplify(*inner.clone()))),
        _ => input.clone(),
    }
}

pub fn build_local_automaton(input: FormulaTy) -> Automaton::<FormulaSet, FormulaSet> {
    let mut res = Automaton::<FormulaSet, FormulaSet>::new();
    res
}

pub fn build_epsilon_graph(input: FormulaTy) -> Graph::<FormulaSet, FormulaSet> {
    let mut res = Graph::<FormulaSet, FormulaSet>::new();
    let mut node_map = HashMap::<NodeIndex, FormulaSet>::new();
    // Initial node with {input}
    let mut init = HashSet::new();
    init.insert(input.clone());
    let init_idx = res.add_node(init.clone());
    node_map.insert(init_idx.clone(), init);

    let mut new_nodes = vec![init_idx];
    while new_nodes.len() != 0 {
        let mut tmp_nodes = Vec::new();
        for idx in new_nodes.iter() {
            let subformulas = node_map.get(idx).unwrap().clone();
            let mut reduced_formulas = subformulas.clone();
            for f in subformulas.iter() {
                match f {
                    FormulaTy::And(lhs, rhs) => {
                        reduced_formulas.remove(f);
                        reduced_formulas.insert(*lhs.clone());
                        reduced_formulas.insert(*rhs.clone());
                        let and_idx = res.add_node(reduced_formulas.clone());
                        res.add_edge(idx.clone(), and_idx.clone(), HashSet::new());
                        tmp_nodes.push(and_idx.clone());
                        node_map.insert(and_idx, reduced_formulas);
                        break;
                    },
                    FormulaTy::Or(lhs, rhs) => {
                        reduced_formulas.remove(f);
                        reduced_formulas.insert(*lhs.clone());
                        let or_idx1 = res.add_node(reduced_formulas.clone());
                        res.add_edge(idx.clone(), or_idx1.clone(), HashSet::new());
                        tmp_nodes.push(or_idx1.clone());
                        node_map.insert(or_idx1, reduced_formulas.clone());

                        reduced_formulas.remove(&(*lhs.clone()));
                        reduced_formulas.insert(*rhs.clone());
                        let or_idx2 = res.add_node(reduced_formulas.clone());
                        res.add_edge(idx.clone(), or_idx2.clone(), HashSet::new());
                        tmp_nodes.push(or_idx2.clone());
                        node_map.insert(or_idx2, reduced_formulas);
                        break;
                    },
                    FormulaTy::Until(lhs, rhs) => {
                        reduced_formulas.remove(f);
                        reduced_formulas.insert(*rhs.clone());
                        let until_idx1 = res.add_node(reduced_formulas.clone());
                        res.add_edge(idx.clone(), until_idx1.clone(), HashSet::new());
                        tmp_nodes.push(until_idx1.clone());
                        node_map.insert(until_idx1, reduced_formulas.clone());

                        reduced_formulas.remove(&(*rhs.clone()));
                        reduced_formulas.insert(*lhs.clone());
                        reduced_formulas.insert(FormulaTy::Next(Box::new(f.clone())));
                        let until_idx2 = res.add_node(reduced_formulas.clone());
                        let mut weight = HashSet::new();
                        weight.insert(FormulaTy::Not(Box::new(f.clone())));
                        res.add_edge(idx.clone(), until_idx2.clone(), weight);
                        tmp_nodes.push(until_idx2.clone());
                        node_map.insert(until_idx2, reduced_formulas.clone());
                        break;
                    },
                    FormulaTy::Release(lhs, rhs) => {
                        reduced_formulas.remove(f);
                        reduced_formulas.insert(*lhs.clone());
                        reduced_formulas.insert(*rhs.clone());
                        let release_idx1 = res.add_node(reduced_formulas.clone());
                        res.add_edge(idx.clone(), release_idx1.clone(), HashSet::new());
                        tmp_nodes.push(release_idx1.clone());
                        node_map.insert(release_idx1, reduced_formulas.clone());

                        reduced_formulas.remove(&(*lhs.clone()));
                        reduced_formulas.insert(FormulaTy::Next(Box::new(f.clone())));
                        let release_idx2 = res.add_node(reduced_formulas.clone());
                        let mut weight = HashSet::new();
                        weight.insert(FormulaTy::Not(Box::new(f.clone())));
                        res.add_edge(idx.clone(), release_idx2.clone(), weight);
                        tmp_nodes.push(release_idx2.clone());
                        node_map.insert(release_idx2, reduced_formulas.clone());
                        break;
                    },
                    _ => {},
                }
            }
        }
        new_nodes = tmp_nodes;
    }
    // println!("epsilon graph: {:?}", res);
    // let dot = format!("{:?}", Dot::with_config(&res, &[Config::GraphContentOnly]));
    // std::fs::write("./epsilon.dot", dot.as_str());
    return res;
}