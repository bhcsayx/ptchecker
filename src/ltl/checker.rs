use std::collections::{HashMap, HashSet};
use std::default::Default;

use bimap::BiMap;
use petgraph::graph::*;
use petgraph::dot::{Dot};

use crate::logics::*;
use crate::logics::transys::*;
use crate::ltl::*;
use crate::ltl::vwaa::*;
use crate::ltl::gba::*;
use crate::ltl::translator::*;
use crate::petri::*;
use crate::utils::*;

pub struct LTLChecker {
    pub auto: Automaton::<(FormulaSet, usize), FormulaSet>,
    pub fin_size: usize,
    pub net: PTNet,
    pub init: Config,
    pub visited_1: Vec<((FormulaSet, usize), Config)>,
    pub visited_2: Vec<((FormulaSet, usize), Config)>,
    pub stack_1: Vec<((FormulaSet, usize), Config)>,
    pub stack_2: Vec<((FormulaSet, usize), Config)>,
}

impl LTLChecker {
    pub fn new(net: &PTNet, auto: Automaton::<(FormulaSet, usize), FormulaSet>, fin: usize) -> Self {
        let mut root = Config::new();
        for (ind, p) in net.places.iter() {
            root.insert(*ind, p.get_tokens());
        }
        LTLChecker {
            auto: auto.clone(),
            fin_size: fin,
            net: net.clone(),
            init: root,
            visited_1: Vec::new(),
            visited_2: Vec::new(),
            stack_1: Vec::new(),
            stack_2: Vec::new(),
        }
    }

    fn fireable_set(&self, config: Config) -> Vec<Config> {
        let mut fireable = vec![];
        for (_, transition) in self.net.transitions.iter() {
            let mut fire = true;
            for (ind_p, capacity) in transition.conditions.iter() {
                let tokens =
                    if let Some(n) = config.get(&ind_p) { *n } else { 0 };
                if tokens < *capacity {
                    fire = false;
                    break;
                }
            }
            if fire {
                let mut new_config = config.clone();
                for (ind_p, capacity) in transition.conditions.iter() {
                    let tokens =
                        if let Some(n) = new_config.get(&ind_p) { *n } else { 0 };
                    new_config.insert(ind_p.clone(), if tokens == usize::MAX {tokens} else { tokens-*capacity });
                }
                for (ind_p, capacity) in transition.effects.iter() {
                    let tokens =
                        if let Some(n) = new_config.get(&ind_p) { *n } else { 0 };
                            new_config.insert(ind_p.clone(), if tokens == usize::MAX {tokens} else { tokens+*capacity });
                }
                fireable.push(new_config.clone());
            }
        }
        fireable
    }

    fn is_fireable(&self, config: Config, tran: &Transition) -> bool {
        let mut fire = true;
        for (ind_p, capacity) in tran.conditions.iter() {
            let tokens =
                if let Some(n) = config.get(&ind_p) { *n } else { 0 };
            if tokens < *capacity {
                fire = false;
                break;
            }
        }
        fire
    }

    pub fn filter_marks(&self, action: &FormulaSet, dest: &(FormulaSet, usize), marks: &Vec<Config>) -> Vec<Config> {
        let mut filtered = marks.clone();
        if *action == FormulaSet::from_iter(vec![FormulaTy::True]) {
            return marks.clone();
        }
        for m in marks.iter() {
            let mut flag = true;
            for a in action.set.iter() {
                if let FormulaTy::Prop(ap) = a.clone() {
                    if let PTAtom::Fireability(name) = ap.clone() {
                        let tran_idx = self.net.index_map.get_by_left(&name).unwrap();
                        let tran = self.net.transitions.get(&tran_idx).unwrap();
                        // println!("tran: {:?}", tran);
                        if !self.is_fireable(m.clone(), &tran) {
                            flag = false;
                            break;
                        }
                    }
                }
                else if let FormulaTy::Neg(ap) = a.clone() {
                    if let PTAtom::Fireability(name) = ap.clone() {
                        let tran_idx = self.net.index_map.get_by_left(&name).unwrap();
                        let tran = self.net.transitions.get(&tran_idx).unwrap();
                        // println!("tran: {:?}", tran);
                        if self.is_fireable(m.clone(), &tran) {
                            flag = false;
                            break;
                        }
                    }
                }
            }
            if flag {
                filtered.push(m.clone());
            }
        }
        filtered
    }

    pub fn dfs1(&mut self, spec: &(FormulaSet, usize), marking: Config) -> bool {
        self.visited_1.push((spec.clone(), marking.clone()));
        self.stack_1.push((spec.clone(), marking.clone()));
        let markings = self.fireable_set(marking.clone());
        // println!("markings: {:?}", markings);
        if !self.auto.transitions.contains_key(spec) {
            // println!("invalid key: {:?}", spec);
            return false;
        }
        for (a, d) in self.auto.transitions[spec].clone().iter() {
            // println!("tran: {:?} {:?}", a, d);
            let filtered = self.filter_marks(a, d, &markings);
            // println!("filtered: {:?}", filtered);
            for m in filtered.iter() {
                if !self.visited_1.contains(&(d.clone(), m.clone())) {
                    let dest = d.clone();
                    if self.dfs1(&dest, m.clone()) {
                        return true;
                    }
                }
            }
        }
        if spec.1 == self.fin_size {
            // accept state, start inner dfs
            if self.dfs2(spec, marking.clone(), spec, marking.clone()) {
                return true;
            }
        }
        false
    }

    pub fn dfs2(&mut self, spec: &(FormulaSet, usize), marking: Config, spec_start: &(FormulaSet, usize), marking_start: Config) -> bool {
        self.visited_2.push((spec.clone(), marking.clone()));
        self.stack_2.push((spec.clone(), marking.clone()));
        let markings = self.fireable_set(marking.clone());
        for (a, d) in self.auto.transitions[spec].clone().iter() {
            let filtered = self.filter_marks(a, d, &markings);
            for m in filtered.iter() {
                if self.visited_1.contains(&(d.clone(), m.clone())) {
                    return true;
                }
                else if !self.visited_2.contains(&(d.clone(), m.clone())) {
                    let dest = d.clone();
                    if self.dfs2(&dest, m.clone(), spec_start, marking_start.clone()) {
                        return true;
                    }
                }
            }
        }
        return false;
    }

    pub fn check(&mut self) -> bool {
        for init in self.auto.init_states.clone().iter() {
            println!("init state: {:?}", init);
            if self.dfs1(init, self.init.clone()) {
                return false;
            }
        }
        true
    }
}