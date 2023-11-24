use std::collections::{HashMap, HashSet};
use std::default::Default;

use bimap::BiMap;
use petgraph::graph::*;
use petgraph::dot::{Dot, Config};

use crate::logics::*;
use crate::ltl::*;
use crate::ltl::vwaa::*;
use crate::ltl::gba::*;
use crate::utils::*;

pub struct PSTV95Translator {
    pub f: FormulaTy,
    // pub subf: FormulaSet,
    pub nodes_num: usize,
    pub incoming: HashMap<usize, HashSet<usize>>,
    pub now: BiMap<usize, FormulaSet>,
    pub next: BiMap<usize, FormulaSet>,
}

impl PSTV95Translator {

    pub fn init(f: &FormulaTy) -> Self {
        let mut incoming = HashMap::new();
        let mut now: BiMap<usize, FormulaSet> = BiMap::new();
        let mut next: BiMap<usize, FormulaSet> = BiMap::new();

        // Initial node
        incoming.insert(0, HashSet::new());
        now.insert(0, FormulaSet::new());
        next.insert(0, FormulaSet::new());

        PSTV95Translator {
            f: f.clone(),
            nodes_num: 1,
            incoming: incoming,
            now: now,
            next: next,
        }
    }

    pub fn run(&mut self) {
        println!("f: {:?}", self.f);
        let init_curr = FormulaSet::from_iter(vec![self.f.clone()]);
        let init_old = FormulaSet::new();
        let init_next = FormulaSet::new();
        let init_incoming = HashSet::from_iter(vec![0]);
        self.expand(init_curr, init_old, init_next, init_incoming);
        println!("now: {:?}", self.now);
        println!("next: {:?}", self.next);
        println!("incoming: {:?}", self.incoming);
    }

    pub fn expand(&mut self, curr: FormulaSet, old: FormulaSet, next: FormulaSet, incoming: HashSet<usize>) {
        if curr.len() == 0 {
            if self.now.contains_right(&old) && self.next.contains_right(&next) {
                let now_idx = self.now.get_by_right(&old).unwrap().clone();
                let next_idx = self.next.get_by_right(&next).unwrap().clone();
                if now_idx == next_idx {
                    let mut incoming_set = self.incoming.get(&now_idx).unwrap().clone();
                    incoming_set = incoming_set.union(&incoming).map(|i| i.clone()).collect();
                    let incoming_ref = self.incoming.get_mut(&now_idx).unwrap();
                    *incoming_ref = incoming_set;
                }
                else {
                    let new_node = self.nodes_num.clone();
                    self.incoming.insert(new_node.clone(), incoming.clone());
                    self.next.insert(new_node.clone(), next.clone());
                    self.now.insert(new_node.clone(), old.clone());
                    self.nodes_num += 1;
                    let curr_incoming = HashSet::from_iter(vec![new_node.clone()]);
                    self.expand(next.clone(), FormulaSet::new(), FormulaSet::new(), curr_incoming);
                }
            }
            else {
                let new_node = self.nodes_num.clone();
                self.incoming.insert(new_node.clone(), incoming.clone());
                self.next.insert(new_node.clone(), next.clone());
                self.now.insert(new_node.clone(), old.clone());
                self.nodes_num += 1;
                let curr_incoming = HashSet::from_iter(vec![new_node.clone()]);
                self.expand(next.clone(), FormulaSet::new(), FormulaSet::new(), curr_incoming);
            }
        }
        else {
            let elem = curr.set.clone().into_iter().collect::<Vec<_>>()[0].clone();
            let mut new_curr = curr.clone();
            new_curr.remove(&elem);
            let mut new_old = old.clone();
            new_old.insert(elem.clone());
            match elem.clone() {
                FormulaTy::False => {},
                FormulaTy::True => {
                    self.expand(new_curr, new_old, next, incoming);
                },
                FormulaTy::Prop(atom) => {
                    if !old.contains(&FormulaTy::Neg(atom)) {
                        self.expand(new_curr, new_old, next, incoming);
                    }
                },
                FormulaTy::Neg(atom) => {
                    if !old.contains(&FormulaTy::Prop(atom)) {
                        self.expand(new_curr, new_old, next, incoming);
                    }
                },
                FormulaTy::And(lhs, rhs) => {
                    let mut and_curr = FormulaSet::from_iter(vec![*lhs.clone(), *rhs.clone()]);
                    and_curr = and_curr.difference(&new_old);
                    and_curr = and_curr.union(&new_curr);
                    self.expand(and_curr, new_old, next, incoming);
                },
                FormulaTy::Next(inner) => {
                    let mut next_next = next.clone();
                    next_next.insert(*inner.clone());
                    self.expand(new_curr, new_old, next_next, incoming);
                },
                FormulaTy::Or(lhs, rhs) => {
                    // First expand
                    let mut or_curr1 = FormulaSet::from_iter(vec![*rhs.clone()]);
                    or_curr1 = or_curr1.difference(&new_old);
                    or_curr1 = or_curr1.union(&new_curr);
                    let mut or_next1 = next.clone();
                    self.expand(or_curr1, new_old.clone(), or_next1, incoming.clone());

                    // Second expand
                    let mut or_curr2 = FormulaSet::from_iter(vec![*lhs.clone()]);
                    or_curr2 = or_curr2.difference(&new_old);
                    or_curr2 = or_curr2.union(&new_curr);
                    self.expand(or_curr2, new_old, next, incoming);
                },
                FormulaTy::Until(lhs, rhs) => {
                    // First expand
                    let mut until_curr1 = FormulaSet::from_iter(vec![*lhs.clone()]);
                    until_curr1 = until_curr1.difference(&new_old);
                    until_curr1 = until_curr1.union(&new_curr);
                    let mut until_next1 = next.clone();
                    until_next1.insert(elem.clone());
                    self.expand(until_curr1, new_old.clone(), until_next1, incoming.clone());

                    // Second expand
                    let mut until_curr2 = FormulaSet::from_iter(vec![*rhs.clone()]);
                    until_curr2 = until_curr2.difference(&new_old);
                    until_curr2 = until_curr2.union(&new_curr);
                    self.expand(until_curr2, new_old, next, incoming);
                },
                FormulaTy::Release(lhs, rhs) => {
                    // First expand
                    let mut release_curr1 = FormulaSet::from_iter(vec![*rhs.clone()]);
                    release_curr1 = release_curr1.difference(&new_old);
                    release_curr1 = release_curr1.union(&new_curr);
                    let mut release_next1 = next.clone();
                    release_next1.insert(elem.clone());
                    self.expand(release_curr1, new_old.clone(), release_next1, incoming.clone());

                    // Second expand
                    let mut release_curr2 = FormulaSet::from_iter(vec![*rhs.clone(), *rhs.clone()]);
                    release_curr2 = release_curr2.difference(&new_old);
                    release_curr2 = release_curr2.union(&new_curr);
                    self.expand(release_curr2, new_old, next, incoming);
                },
                _ => {
                    println!("Unhandled formula detected: {:?}", elem);
                }
            }
        }
        return;
    }

    pub fn print_automaton(&self) {

    }
}

pub struct CAV01Translator {
    pub f: FormulaTy,
    pub sub_f: HashSet<FormulaTy>,
    pub atrans: HashMap<FormulaTy, Vec<(FormulaSet, FormulaTy)>>,
    pub gtrans: HashMap<FormulaSet, Vec<(FormulaSet, FormulaSet)>>,
}

impl CAV01Translator {
    pub fn init(f: &FormulaTy) -> Self {
        CAV01Translator {
            f: f.clone(),
            sub_f: ltl_subformulas(f.clone()),
            atrans: HashMap::new(),
            gtrans: HashMap::new(),
        }
    }

    pub fn vwaa_build(&mut self) -> (FormulaSet, FormulaSet) {
        // VWAA states
        let mut states = HashSet::new();
        for sub in self.sub_f.iter() {
            match sub {
                FormulaTy::True => {},
                FormulaTy::False => {},
                FormulaTy::Prop(_) => {},
                FormulaTy::Neg(_) => {},
                FormulaTy::And(_, _) => {},
                FormulaTy::Or(_, _) => {},
                _ => {
                    states.insert(sub.clone());
                }
            }
        }
        println!("states: {:?}", states);

        // VWAA alphabet
        let mut alphabet = HashSet::new();
        for s in states.iter() {
            match s {
                FormulaTy::Prop(atom) => {alphabet.insert(atom.clone());},
                FormulaTy::Neg(atom) => {alphabet.insert(atom.clone());},
                _ => {},
            }
        }
        println!("alphabet: {:?}", powerset(&alphabet));

        // VWAA initial states
        let inits = vwaa_bar(self.f.clone());
        println!("initial state: {:?}", inits);

        // VWAA final states
        let mut finals = FormulaSet::new();
        for sub in self.sub_f.iter() {
            if let FormulaTy::Until(_, _) = sub {
                finals.insert(sub.clone());
            }
        }
        println!("final state: {:?}", finals);

        let mut transitions = HashMap::<(FormulaTy, PTAtom), HashSet<FormulaTy>>::new();

        // VWAA transitions
        for s in states.iter() {
            // println!("trans for state {:?}: {:?}", s, vwaa_delta(s.clone()));
            for (action, dest) in vwaa_delta(s.clone()).iter() {
                let mut action = action.clone();
                if action.len() > 1 {
                    action.remove(&FormulaTy::True);
                }
                if self.atrans.contains_key(s) {
                    let trans_ref = self.atrans.get_mut(s).unwrap();
                    trans_ref.push((action.clone(), dest.clone()));
                }
                else {
                    self.atrans.insert(s.clone(), vec![(action.clone(), dest.clone())]);
                }
            }
        }
        // println!("trans: {:#?}", self.atrans);
        (inits, finals)
    }

    pub fn gba_build(&mut self, inits: &FormulaSet, finals: &FormulaSet) {
        let mut unprocessed = vec![];
        let mut processed = vec![];
        for i in inits.set.iter() {
            let init_set = break_conjs(i);
            unprocessed.push(init_set.clone());
        }
        
        while unprocessed.len() != 0 {
            let state = unprocessed.remove(0);
            // println!("state: {:?}", state);
            processed.push(state.clone());
            if state.len() == 0 {
                continue;
            }
            let mut trans = HashSet::new();
            trans.insert((FormulaSet::from_iter(vec![FormulaTy::True]), FormulaTy::True));
            for s in state.set.iter() {
                trans = vwaa_product(trans.clone(), vwaa_delta(s.clone()));
            }

            for (a, d) in trans.iter() {
                let d_set = break_conjs(d);
                let mut a_set = a.clone();
                if a_set.len() > 1 {
                    a_set.remove(&FormulaTy::True);
                }
                // println!("tran: {:?} to: {:?}", a_set, d_set);
                if self.gtrans.contains_key(&state) {
                    let s_ref = self.gtrans.get_mut(&state).unwrap();
                    s_ref.push((a_set.clone(), d_set.clone()));
                }
                else {
                    self.gtrans.insert(state.clone(), vec![(a_set.clone(), d_set.clone())]);
                }
                if !processed.contains(&d_set) {
                    unprocessed.push(d_set.clone());
                }
            }
        }
        println!("trans: {:#?}", self.gtrans);
    }

    pub fn run(&mut self) {
        let (inits, finals) = self.vwaa_build();
        self.gba_build(&inits, &finals);
    }
}

pub fn build_automaton_pstv95(f: &Formula) {
    println!("Original f: {:?}", f.ty.clone());
    println!("Negated f: {:?}", ltl_negate(f.ty.clone()));
    let preprocessed = ltl_simplify(ltl_negate(f.ty.clone()));
    if let FormulaTy::Forall(inner) = &preprocessed { // Make sure this is an LTL formula
        let mut translator = PSTV95Translator::init(&*inner);
        translator.run();
        translator.print_automaton();
    }
}

pub fn build_automaton_cav01(f: &Formula) {
    println!("Original f: {:?}", f.ty.clone());
    println!("Negated f: {:?}", ltl_negate(f.ty.clone()));
    let preprocessed = ltl_simplify(ltl_negate(f.ty.clone()));
    if let FormulaTy::Forall(inner) = &preprocessed { // Make sure this is an LTL formula
        println!("Simplified f: {:?}", inner);
        let mut translator = CAV01Translator::init(&*inner);
        println!("subs: {:?}", translator.sub_f);
        translator.run();
        // translator.print_automaton();
    }
}