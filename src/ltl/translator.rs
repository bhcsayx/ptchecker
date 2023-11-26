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

pub struct CAV01Translator {
    pub f: FormulaTy,
    pub sub_f: HashSet<FormulaTy>,
    pub atrans: HashMap<FormulaTy, Vec<(FormulaSet, FormulaTy)>>,
    pub gtrans: HashMap<FormulaSet, Vec<(FormulaSet, FormulaSet, FormulaSet)>>,
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
        // println!("atrans: {:#?}", self.atrans);
        (inits, finals)
    }

    pub fn gba_final(&self, state: &FormulaSet, action: &FormulaSet, dest: &FormulaSet, finals: &FormulaSet) -> FormulaSet {
        let mut res = FormulaSet::new();
        let mut destination = dest.clone();
        for f in finals.set.iter() {
            if !self.atrans.contains_key(f) {
                println!("Error of state {:?} not exist in atrans", f);
                break;
            }
            if !destination.contains(f) {
                // println!("marking final: {:?} {:?}", f, dest);
                res.insert(f.clone());
                continue;
            }
            else {
                destination.remove(f);
                for (a, d) in self.atrans[f].iter() {
                    if a.set.is_subset(&action.set) && (destination.contains(d) || *d == FormulaTy::True) {
                        res.insert(f.clone());
                        break;
                    }
                }
                destination.insert(f.clone());
            }
        }
        res
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
            // for (a, d) in trans.iter() {
            //     println!("trans from {:?} to {:?} under {:?}", state, d, a);
            // }

            for (a, d) in trans.iter() {
                let d_set = break_conjs(d);
                let mut a_set = a.clone();
                if a_set.len() > 1 {
                    a_set.remove(&FormulaTy::True);
                }
                // println!("tran broken: {:?} -> {:?} on {:?}", state, d_set, a_set);
                let f_set = self.gba_final(&state, &a_set, &d_set, finals);
                // println!("tran with final: {:?} -> {:?} on {:?}, final: {:?}", state, d_set, a_set, f_set);
                if !processed.contains(&d_set) {
                    if !unprocessed.contains(&d_set) {
                        unprocessed.push(d_set.clone());
                    }
                }
                if !self.gtrans.contains_key(&state) {
                    self.gtrans.insert(state.clone(), vec![(a_set.clone(), d_set.clone(), f_set.clone())]);
                }
                else {
                    let s_ref = self.gtrans.get_mut(&state).unwrap();
                    let mut insert_flag = true;
                    for (a, d, f) in s_ref.iter_mut() {
                        if a.set.is_subset(&a_set.set) && d.set.is_subset(&d_set.set) && *f == f_set {
                            insert_flag = false;
                        }
                        else if a_set.set.is_subset(&a.set.clone()) && d_set.set.is_subset(&d.set.clone()) && *f == f_set {
                            *a = a_set.clone();
                            *d = d_set.clone();
                            insert_flag = false;
                        }
                    }
                    if (insert_flag) {
                        s_ref.push((a_set.clone(), d_set.clone(), f_set.clone()));
                    }
                }
            }
        }
        // println!("gtrans: {:#?}", self.gtrans);
        // for (k, v) in self.gtrans.iter() {
        //     for (a, d, f) in v.iter() {
        //         println!("trans: {:?} -> {:?} under {:?}, final {:?}", k, d, a, f);
        //     }
        // }
    }

    pub fn run(&mut self) {
        let (inits, finals) = self.vwaa_build();
        self.gba_build(&inits, &finals);
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