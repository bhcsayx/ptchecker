use crate::logics::*;
use crate::petri::*;

use petgraph::graph::*;
use petgraph::dot::{Dot};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;

pub mod checker;
pub mod vwaa;
pub mod gba;
pub mod translator;

use crate::ltl::translator::*;

pub fn ltl_subf_recur(set: &mut HashSet<FormulaTy>, input: FormulaTy) {
    set.insert(input.clone());
    match input {
        FormulaTy::True | FormulaTy::False => {},
        FormulaTy::Prop(atom) => {},
        FormulaTy::Neg(atom) => {},
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

pub fn ltl_subformulas(input: FormulaTy) -> HashSet<FormulaTy> {
    // if let FormulaTy::Forall(inner) = input {
    //     let mut set = HashSet::new();
    //     ltl_subf_recur(&mut set, *inner.clone());
    //     return set;
    // }
    // else {
    //     println!("Malformed LTL formula");
    //     return HashSet::new();
    // }
    let mut set = HashSet::new();
    ltl_subf_recur(&mut set, input.clone());
    return set;
}

pub fn ltl_negate(input: FormulaTy) -> FormulaTy {
    // Negate an LTL formula
    match input {
        FormulaTy::True => FormulaTy::False,
        FormulaTy::False => FormulaTy::True,
        FormulaTy::Prop(atom) => {
            FormulaTy::Neg(atom.clone())
        },
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
        _ => {
            println!("Strange formular: {:?}", input.clone());
            input.clone()
        },
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

// fn expand(nodes: &mut HashSet<usize>, index: &mut usize, now_table: &mut HashMap<usize, FormulaSet>, next_table: &mut HashMap<usize, FormulaSet>, incoming_table: &mut 
//     HashMap<usize, HashSet<usize>>, curr: FormulaSet, old: FormulaSet, next: FormulaSet, incoming: HashSet<usize>) {

//     if curr.len() == 0 {
//         let mut found = false;
//         for q in nodes.iter() {
//             // println!("getting q: {:?}", q);
//             if now_table.get(q).unwrap().clone() == old && next_table.get(q).unwrap().clone() == next {
//                 found = true;
//                 for i in incoming.iter() {
//                     let incomings = incoming_table.get_mut(q).unwrap();
//                     incomings.insert(i.clone());
//                 }
//                 break;
//             }
//         }
//         if !found {
//             nodes.insert(index.clone());
//             incoming_table.insert(index.clone(), incoming.clone());
//             now_table.insert(index.clone(), old.clone());
//             next_table.insert(index.clone(), next.clone());
//             let mut new_incoming = HashSet::new();
//             new_incoming.insert(index.clone());
//             *index += 1;

//             expand(nodes, index, now_table, next_table, incoming_table, next.clone(), HashSet::new(), HashSet::new(), new_incoming);
//         }
//     }
//     else {
//         for f in curr.iter() {
//             let mut new_curr = curr.clone();
//             new_curr.remove(f);
//             let mut new_old = old.clone();
//             new_old.insert(f.clone());
//             match f {
//                 FormulaTy::True => {
//                     expand(nodes, index, now_table, next_table, incoming_table, new_curr.clone(), new_old.clone(), next.clone(), incoming.clone());
//                 },
//                 FormulaTy::False => {},
//                 FormulaTy::Neg(atom) => {
//                     if !new_old.contains(&FormulaTy::Prop(atom.clone())) {
//                         expand(nodes, index, now_table, next_table, incoming_table, new_curr.clone(), new_old.clone(), next.clone(), incoming.clone());
//                     }
//                 },
//                 FormulaTy::Prop(atom) => {
//                     if !new_old.contains(&FormulaTy::Neg(atom.clone())) {
//                         expand(nodes, index, now_table, next_table, incoming_table, new_curr.clone(), new_old.clone(), next.clone(), incoming.clone());
//                     }
//                 },
//                 FormulaTy::And(lhs, rhs) => {
//                     // println!("Entering and");
//                     let mut and_curr = new_curr.clone();
//                     let mut and_curr_right = HashSet::new();
//                     and_curr_right.insert(*lhs.clone());
//                     and_curr_right.insert(*rhs.clone());
//                     and_curr_right = and_curr_right.difference(&new_old).map(|item| item.clone()).collect();
//                     and_curr = and_curr.union(&and_curr_right).map(|item| item.clone()).collect();
//                     expand(nodes, index, now_table, next_table, incoming_table, and_curr.clone(), new_old.clone(), next.clone(), incoming.clone());
//                 },
//                 FormulaTy::Next(inner) => {
//                     let mut new_next = next.clone();
//                     let mut next_rhs = HashSet::new();
//                     next_rhs.insert(*inner.clone());
//                     new_next = new_next.union(&next_rhs).map(|item| item.clone()).collect();
//                     expand(nodes, index, now_table, next_table, incoming_table, new_curr.clone(), new_old.clone(), new_next.clone(), incoming.clone());
//                 },
//                 FormulaTy::Or(lhs, rhs) => {
//                     let mut or_curr1 = new_curr.clone();
//                     let mut or_curr_right1 = HashSet::new();
//                     or_curr_right1.insert(*rhs.clone());
//                     or_curr_right1 = or_curr_right1.difference(&new_old).map(|item| item.clone()).collect();
//                     or_curr1 = or_curr1.union(&or_curr_right1).map(|item| item.clone()).collect();
//                     let mut or_curr2 = new_curr.clone();
//                     let mut or_curr_right2 = HashSet::new();
//                     or_curr_right2.insert(*lhs.clone());
//                     or_curr_right2 = or_curr_right2.difference(&new_old).map(|item| item.clone()).collect();
//                     or_curr2 = or_curr2.union(&or_curr_right2).map(|item| item.clone()).collect();
//                     expand(nodes, index, now_table, next_table, incoming_table, or_curr1.clone(), new_old.clone(), next.clone(), incoming.clone());
//                     expand(nodes, index, now_table, next_table, incoming_table, or_curr2.clone(), new_old.clone(), next.clone(), incoming.clone());
//                 },
//                 FormulaTy::Until(lhs, rhs) => {
//                     let mut or_curr1 = new_curr.clone();
//                     let mut or_curr_right1 = HashSet::new();
//                     or_curr_right1.insert(*lhs.clone());
//                     or_curr_right1 = or_curr_right1.difference(&new_old).map(|item| item.clone()).collect();
//                     or_curr1 = or_curr1.union(&or_curr_right1).map(|item| item.clone()).collect();
//                     let mut next_right = HashSet::new();
//                     next_right.insert(f.clone());
//                     let mut new_next = next.clone();
//                     new_next = new_next.union(&next_right).map(|item| item.clone()).collect();
//                     let mut or_curr2 = new_curr.clone();
//                     let mut or_curr_right2 = HashSet::new();
//                     or_curr_right2.insert(*rhs.clone());
//                     or_curr_right2 = or_curr_right2.difference(&new_old).map(|item| item.clone()).collect();
//                     or_curr2 = or_curr2.union(&or_curr_right2).map(|item| item.clone()).collect();
//                     expand(nodes, index, now_table, next_table, incoming_table, or_curr1.clone(), new_old.clone(), new_next.clone(), incoming.clone());
//                     expand(nodes, index, now_table, next_table, incoming_table, or_curr2.clone(), new_old.clone(), next.clone(), incoming.clone());
//                 },
//                 FormulaTy::Release(lhs, rhs) => {
//                     let mut or_curr1 = new_curr.clone();
//                     let mut or_curr_right1 = HashSet::new();
//                     or_curr_right1.insert(*rhs.clone());
//                     or_curr_right1 = or_curr_right1.difference(&new_old).map(|item| item.clone()).collect();
//                     or_curr1 = or_curr1.union(&or_curr_right1).map(|item| item.clone()).collect();
//                     let mut next_right = HashSet::new();
//                     next_right.insert(f.clone());
//                     let mut new_next = next.clone();
//                     new_next = new_next.union(&next_right).map(|item| item.clone()).collect();
//                     let mut or_curr2 = new_curr.clone();
//                     let mut or_curr_right2 = HashSet::new();
//                     or_curr_right2.insert(*rhs.clone());
//                     or_curr_right2.insert(*lhs.clone());
//                     or_curr_right2 = or_curr_right2.difference(&new_old).map(|item| item.clone()).collect();
//                     or_curr2 = or_curr2.union(&or_curr_right2).map(|item| item.clone()).collect();
//                     expand(nodes, index, now_table, next_table, incoming_table, or_curr1.clone(), new_old.clone(), new_next.clone(), incoming.clone());
//                     expand(nodes, index, now_table, next_table, incoming_table, or_curr2.clone(), new_old.clone(), next.clone(), incoming.clone());
//                 },
//                 _ => {},
//             }
//             break;
//         }
//     }
// }

// pub fn build_graph(input: FormulaTy) -> (HashSet<usize>, HashMap<usize, FormulaSet>, HashMap<usize, HashSet<usize>>) {
//     let mut nodes = HashSet::new(); // 0 is for init node
//     let mut now_table: HashMap<usize, FormulaSet> = HashMap::new();
//     let mut next_table: HashMap<usize, FormulaSet> = HashMap::new();
//     let mut incoming_table: HashMap<usize, HashSet<usize>> = HashMap::new();
//     let mut index = 1;

//     nodes.insert(0); // Insert init as 0;
//     now_table.insert(0, HashSet::new());
//     next_table.insert(0, HashSet::new());
//     incoming_table.insert(0, HashSet::new());

//     let mut curr = HashSet::new();
//     curr.insert(input.clone());

//     let mut old = HashSet::new();
//     let mut next = HashSet::new();
//     let mut incoming = HashSet::new();
//     incoming.insert(0);
    
//     expand(&mut nodes, &mut index, &mut now_table, &mut next_table, &mut incoming_table, curr, old, next, incoming);
//     // println!("{:?} {:?} {:?}", nodes, now_table, incoming_table);
//     return (nodes, now_table, incoming_table);
// }

// pub fn print_automaton(graph: (HashSet<usize>, HashMap<usize, FormulaSet>, HashMap<usize, HashSet<usize>>), subs: FormulaSet) {
//     let (nodes, now_table, incoming_table) = graph;
//     for (k, n) in now_table.iter() {
//         println!("node {:?}: {:?}", k, n);
//     }
//     for (k, i)  in incoming_table.iter() {
//         if i.contains(&0) {
//             println!("{:?} is an initial state", k);
//         }
//     }
//     let mut fi = HashSet::new();
//     for sub in subs.iter() {
//         if let FormulaTy::Until(lhs, rhs) = sub {
//             for (k, n) in now_table.iter() {
//                 if (n.contains(&(*rhs.clone())) || !n.contains(sub)) && k.clone() != 0 {
//                     fi.insert(k.clone());
//                 }
//             }
//         }
//     }
//     for f in fi.iter() {
//         println!("{:?} is a final state", f);
//     }
//     let mut automaton_graph = Graph::<FormulaSet, FormulaSet>::new();
//     let mut idx_map: HashMap<usize, NodeIndex> = HashMap::new();

//     for (k, n) in now_table.iter() {
//         let g_idx = automaton_graph.add_node(n.clone());
//         idx_map.insert(k.clone(), g_idx.clone());
//     }
//     for (k, s) in incoming_table.iter() {
//         let dst_idx = idx_map.get(k).unwrap().clone();
//         for i in s.iter() {
//             let src_idx = idx_map.get(i).unwrap().clone();
//             let mut weight = HashSet::new();
//             for f in now_table.get(k).unwrap().clone() {
//                 if let FormulaTy::Prop(_) = f {
//                     weight.insert(f.clone());
//                 }
//             }
//             automaton_graph.add_edge(src_idx, dst_idx, weight);
//         }
//     }
//     let output = format!("{:?}", Dot::with_config(&automaton_graph, &[Config::GraphContentOnly]));
//     std::fs::write("automaton.dot", output.as_str());
//     // let mut res = Automaton::<FormulaSet, PTAtom>::new();
//     // res
// }