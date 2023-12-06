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
    pub trans: HashMap<(FormulaSet, usize), Vec<(FormulaSet, FormulaSet, usize)>>,
}

impl CAV01Translator {
    pub fn init(f: &FormulaTy) -> Self {
        CAV01Translator {
            f: f.clone(),
            sub_f: ltl_subformulas(f.clone()),
            atrans: HashMap::new(),
            gtrans: HashMap::new(),
            trans: HashMap::new(),
        }
    }

    pub fn vwaa_build(&mut self) -> (FormulaSet, FormulaSet) {
        // VWAA states
        let mut states = HashSet::new();
        for sub in self.sub_f.iter() {
            match sub {
                FormulaTy::True => {},
                FormulaTy::False => {},
                // FormulaTy::Prop(_) => {},
                // FormulaTy::Neg(_) => {},
                FormulaTy::And(_, _) => {},
                FormulaTy::Or(_, _) => {},
                _ => {
                    states.insert(sub.clone());
                }
            }
        }
        // println!("states: {:?}", states);

        // VWAA alphabet
        let mut alphabet = HashSet::new();
        for s in states.iter() {
            match s {
                FormulaTy::Prop(atom) => {alphabet.insert(atom.clone());},
                FormulaTy::Neg(atom) => {alphabet.insert(atom.clone());},
                _ => {},
            }
        }
        // println!("alphabet: {:?}", powerset(&alphabet));

        // VWAA initial states
        let inits = vwaa_bar(self.f.clone());
        // println!("initial state: {:?}", inits);

        // VWAA final states
        let mut finals = FormulaSet::new();
        for sub in self.sub_f.iter() {
            if let FormulaTy::Until(_, _) = sub {
                finals.insert(sub.clone());
            }
        }
        // println!("final state: {:?}", finals);

        let mut transitions = HashMap::<(FormulaTy, PTAtom), HashSet<FormulaTy>>::new();

        // VWAA transitions
        for s in states.iter() {
            // println!("trans for state {:?}: {:?}", s, vwaa_delta(s.clone()));
            for (action, dest) in vwaa_delta(s.clone()).iter() {
                let mut action = action.clone();
                if action.len() > 1 {
                    action.remove(&FormulaTy::True);
                }
                if *dest == FormulaTy::True || *dest == FormulaTy::False {
                    continue;
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
        // for (k, v) in self.atrans.iter() {
        //     for t in v.iter() {
        //         println!("atran: {:?} -- {:?} -> {:?}", k, t.0, t.1);
        //     }
        // }
        // println!("keys: {:?}", self.atrans.keys());
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
            if !state.contains(f) {
                // println!("marking final: {:?} {:?}", f, dest);
                res.insert(f.clone());
                continue;
            }
            else {
                let in_flag = destination.contains(&f);
                // destination.remove(f);
                for (a, d) in self.atrans[f].iter() {
                    let vwaa_d = break_conjs(d);
                    if a.set.is_subset(&action.set) && (vwaa_d.set.is_subset(&destination.set) && !vwaa_d.contains(&f)) {
                        res.insert(f.clone());
                        break;
                    }
                }
                if in_flag {
                    destination.insert(f.clone());
                }
            }
        }
        res
    }

    pub fn gba_build(&mut self, inits: &FormulaSet, finals: &FormulaSet) {
        let mut unprocessed = vec![];
        let mut processed = vec![];
        // for i in inits.set.iter() {
        //     let init_set = break_conjs(i);
        //     unprocessed.push(init_set.clone());
        // }
        unprocessed.push(inits.clone());
        
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
                // let d_set = FormulaSet::from_iter(vec![d.clone()]);
                let mut a_set = a.clone();
                // if a_set.len() > 1 {
                //     a_set.remove(&FormulaTy::True);
                // }
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
                        s_ref.insert(0, (a_set.clone(), d_set.clone(), f_set.clone()));
                    }
                }
            }
        }
        // println!("gtrans: {:#?}", self.gtrans);
        // for (k, v) in self.gtrans.iter() {
        //     for (a, d, f) in v.iter() {
        //         println!("gtrans: {:?} -> {:?} under {:?}, final {:?}", k, d, a, f);
        //     }
        // }
    }

    pub fn ba_final(&self, finals: &Vec<FormulaTy>, tran: &(FormulaSet, FormulaSet, FormulaSet), prev: usize) -> usize {
        // if prev != finals.len() && tran.2.contains(&finals[prev.clone()]) {
        //     return self.ba_final(finals, tran, prev + 1);
        // }
        // return prev;
        let mut ret = if prev == finals.len() {0} else {prev};
        for i in ret..finals.len() {
            if !tran.2.contains(&finals[i.clone()]) {
                return i;
            }
        }
        return finals.len();
    }

    pub fn ba_build(&mut self, inits: &FormulaSet, finals: &FormulaSet) -> (Automaton::<(FormulaSet, usize), FormulaSet>, usize) {
        let mut unprocessed = vec![];
        let mut processed = vec![];
        let mut tmp_trans = HashMap::new();
        let mut finals_vec: Vec<FormulaTy> = finals.set.iter().map(|f| f.clone()).collect();
        // println!("finals vec: {:?}", finals_vec);
        // for i in inits.set.iter() {
        //     let init_set = break_conjs(i);
        //     println!("init set: {:?}", init_set);
        //     for (k, v) in self.gtrans.iter() {
        //         if *k == init_set {
        //             // println!("Key found: {:#?}", self.gtrans[k]);
        //             for tran in self.gtrans[k].iter() {
        //                 let fin = self.ba_final(&finals_vec, tran, 0);
        //                 // println!("inserting tran: {:?} {:?} {:?}", tran.1, tran.2, fin);
        //                 unprocessed.insert(0, (tran.1.clone(), fin.clone()));
        //             }
        //             break;
        //         }
        //     }
        // }
        // for (k, v) in self.gtrans.iter() {
        //     if *k == inits {
        //         // println!("Key found: {:#?}", self.gtrans[k]);
        //         for tran in self.gtrans[k].iter() {
        //             let fin = self.ba_final(&finals_vec, tran, 0);
        //             // println!("inserting tran: {:?} {:?} {:?}", tran.1, tran.2, fin);
        //             unprocessed.insert(0, (tran.1.clone(), fin.clone()));
        //         }
        //         break;
        //     }            
        // }
        unprocessed.push((inits.clone(), 0));

        while unprocessed.len() != 0 {
            let (gstate, fin) = unprocessed.remove(0);
            processed.push((gstate.clone(), fin.clone()));
            // println!("Handling bstate: {:?} {:?}", gstate, fin);
            for (k, v) in self.gtrans.iter() {
                if *k == gstate {
                    for (a_set, d_set, f_set) in self.gtrans[k].iter() {
                        let new_fin = self.ba_final(&finals_vec, &(a_set.clone(), d_set.clone(), f_set.clone()), 
                            fin.clone());
                        let mut new_flag = true;
                        for (s, fin) in unprocessed.iter() {
                            if *s == d_set.clone() && *fin == new_fin {
                                new_flag = false;
                            }
                        }
                        for (s, fin) in processed.iter() {
                            if *s == d_set.clone() && *fin == new_fin {
                                new_flag = false;
                            }
                        }
                        if new_flag {
                            // println!("find new state: {:?}", (d_set.clone(), new_fin.clone()));
                            // println!("from transition: {:?} -- {:?} -> {:?}", gstate.clone(), f_set.clone(), d_set.clone());
                            unprocessed.insert(0, (d_set.clone(), new_fin.clone()));
                        }
                        // else {
                        //     // println!("find existing state: {:?}", (d.clone(), new_fin.clone()));
                        // }
                        if !tmp_trans.contains_key(&(gstate.clone(), fin)) {
                            tmp_trans.insert((gstate.clone(), fin.clone()), vec![(a_set.clone(), d_set.clone(), new_fin.clone())]);
                        }
                        else {
                            let s_ref = tmp_trans.get_mut(&(gstate.clone(), fin)).unwrap();
                            let mut insert_flag = true;
                            for (a, d, f) in s_ref.iter_mut() {
                                if a.set.is_subset(&a_set.set) && d.set.is_subset(&d_set.set) && *f == new_fin {
                                    insert_flag = false;
                                }
                                else if a_set.set.is_subset(&a.set.clone()) && d_set.set.is_subset(&d.set.clone()) && *f == new_fin {
                                    *a = a_set.clone();
                                    *d = d_set.clone();
                                    insert_flag = false;
                                }
                            }
                            if insert_flag {
                                s_ref.insert(0, (a_set.clone(), d_set.clone(), new_fin.clone()));
                            }
                        }
                    }
                    break;
                }
            }
        }
        // println!("trans: {:#?}", self.gtrans);
        let mut expelled = HashSet::new();
        for (k1, v1) in tmp_trans.iter() {
            for (k2, v2) in tmp_trans.iter() {
                if *k1 == *k2 || expelled.contains(k2) {
                    continue;
                }
                if !((k1.1 + k2.1) == 2 * finals_vec.len() || k1.1 != finals_vec.len() && k2.1 != finals_vec.len()) {
                    continue;
                }
                else {
                    let set1: HashSet<&(FormulaSet, FormulaSet, usize)> = HashSet::from_iter(v1);
                    let set2: HashSet<&(FormulaSet, FormulaSet, usize)> = HashSet::from_iter(v2);
                    if set1 == set2 {
                        expelled.insert(k1.clone());
                        break;
                    }
                }
            }
        }
        for (k, v) in tmp_trans.iter() {
            if !expelled.contains(k) {
                self.trans.insert(k.clone(), vec![]);
                for (a, d, f) in v.iter() {
                    if !expelled.contains(&(d.clone(), f.clone())) {
                        let s_ref = self.trans.get_mut(k).unwrap();
                        s_ref.push((a.clone(), d.clone(), f.clone()));
                    }
                }
            }
        }
        let mut auto = Automaton::<(FormulaSet, usize), FormulaSet>::new();
        for (k, v) in self.trans.iter() {
            if !auto.states.contains(k) {
                auto.states.push(k.clone());
            }
            if k.0 == *inits && k.1 == 0 {
                // println!("buchi init state: {:?}", k);
                if !auto.init_states.contains(k) {
                    auto.init_states.push(k.clone());
                }
            }
            // if k.1 == finals_vec.len() {
            //     println!("buchi acc state: {:?}", k);
            // }
            for (a, d, f) in v.iter() {
                // println!("trans: {:?} -> {:?} under {:?}", k, (d, f), a);
                if !auto.transitions.contains_key(&k.clone()) {
                    auto.transitions.insert(k.clone(), vec![(a.clone(), (d.clone(), f.clone()))]);
                }
                else {
                    let s_ref = auto.transitions.get_mut(&k.clone()).unwrap();
                    s_ref.push((a.clone(), (d.clone(), f.clone())));
                }
            }
        }
        (auto, finals_vec.len())
        // println!("init: {:?}", inits);
        // println!("final: {:?}", finals);
    }

    pub fn run(&mut self) -> (Automaton::<(FormulaSet, usize), FormulaSet>, usize) {
        let (inits, finals) = self.vwaa_build();
        self.gba_build(&inits, &finals);
        return self.ba_build(&inits, &finals);
    }
}

pub fn build_automaton_cav01(f: &Formula) -> Option<(Automaton::<(FormulaSet, usize), FormulaSet>, usize)> {
    // println!("Original f: {:?}", f.ty.clone());
    // println!("Negated f: {:?}", ltl_negate(f.ty.clone()));
    let preprocessed = ltl_simplify(ltl_negate(f.ty.clone()));
    if let FormulaTy::Forall(inner) = &preprocessed { // Make sure this is an LTL formula
        // println!("Simplified f: {:?}", inner);
        let mut translator = CAV01Translator::init(&*inner);
        // println!("subs: {:?}", translator.sub_f);
        let res = translator.run();
        return Some(res);
        // translator.print_automaton();
    }
    return None;
}