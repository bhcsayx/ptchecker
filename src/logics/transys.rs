use std::collections::{HashMap, HashSet};
use std::hash::{Hash};
use crate::logics::{FormulaTy, PTAtom};
use crate::petri::{Marking, Place, PTNet};

pub(crate) type State = usize;

type Config = HashMap<usize, usize>;

pub struct TranSys {
    states: HashSet<State>,
    state2conf: HashMap<State, Config>,
    transitions: HashMap<State, HashSet<State>>,
    labels: HashMap<String, HashSet<State>>,
}

impl TranSys {
    pub fn new() -> TranSys {
        TranSys {
            states: HashSet::new(),
            state2conf: HashMap::new(),
            transitions: HashMap::new(),
            labels: HashMap::new(),
        }
    }

    pub fn insert_state(&mut self, s: State) {
        self.states.insert(s);
    }

    pub fn insert_mapping(&mut self, s: State, c: Config) {
        if ! self.states.contains(&s) {
            self.insert_state(s);
        }
        self.state2conf.insert(s, c);
    }

    pub fn insert_transition(&mut self, s1: State, s2: State) {
        self.transitions.entry(s1).or_insert_with(|| HashSet::new()).insert(s2);
        // if let Some(set) = self.transitions.get(&s1) {
        //     set.insert(s2);
        // } else {
        //     let mut set = HashSet::new();
        //     set.insert(s2);
        //     self.transitions.insert(s1, set);
        // }
    }

    pub fn insert_fireable(&mut self, state: State, names: Vec<String>) {
        for name in names {
            self.labels.entry(name).or_insert_with(|| HashSet::new()).insert(state);
        }
    }

    pub fn states_from(&self, source: &State) -> Option<&HashSet<State>> {
        self.transitions.get(source)
    }

    pub fn label_of(&self, f: &String) -> Option<&HashSet<State>> {
        self.labels.get(f)
    }

    pub fn config_of(&self, state: State) -> Option<&Config> {
        self.state2conf.get(&state)
    }

    fn duplicate_config(&self, new_config: &Config) -> Option<usize> {
        for (k, v) in self.state2conf.iter() {
            if *v == *new_config {
                return Some(k.clone());
            }
        }
        return None;
    }

    fn merge_pair(&mut self, s1: State, s2: State) {
        self.states.remove(&s2);
        self.state2conf.remove(&s2);
        self.transitions.remove(&s1);
        for set in self.transitions.values_mut() {
            if set.contains(&s2) {
                set.remove(&s2);
                set.insert(s1);
            }
        }
    }

    fn merge_graph(&mut self) {
        let mut tmp = vec![];
        for i in 0..self.states.len() {
            let ith = self.states.get(&i).unwrap();
            for j in i + 1..self.states.len() {
                let jth = self.states.get(&j).unwrap();
                if let Some(&ref config1) = self.state2conf.get(ith) {
                    if let Some(&ref config2) = self.state2conf.get(jth) {
                        if config1 == config2 {
                            tmp.push((*ith, *jth));
                        }
                    }
                }
            }
        }
        for (ith, jth) in tmp {
            self.merge_pair(ith, jth);
        }
    }

    pub fn from_petri(petri: &PTNet) -> TranSys {
        let mut tran = TranSys::new();
        let mut root = Config::new();
        // tran.insert_mapping(0, root);
        for (ind, p) in petri.places.iter() {
            root.insert(*ind, p.get_tokens());
        }

        let mut path_map: HashMap<usize, usize> = HashMap::new();
        path_map.insert(0, 0);

        let mut all = Vec::new();
        let mut index_all = 0;
        all.push((index_all, root.clone()));
        tran.insert_mapping(index_all, root);
        while !all.is_empty() {
            let (index_old, config) = all.pop().unwrap();
            let path = find_path(&path_map, index_old);
            if has_identical(&tran.state2conf, &path, &config) {
                continue
            }

            let mut fireable = vec![];
            for (_, transition) in petri.transitions.iter() {
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
                    fireable.push(transition.name.clone());
                    let mut new_config = config.clone();
                    for (ind_p, capacity) in transition.conditions.iter() {
                        let tokens =
                            if let Some(n) = new_config.get(&ind_p) { *n } else { 0 };
                        new_config.insert(*ind_p, if tokens == usize::MAX {tokens} else { tokens-*capacity });
                    }
                    for (ind_p, capacity) in transition.effects.iter() {
                        let tokens =
                            if let Some(n) = new_config.get(&ind_p) { *n } else { 0 };
                        new_config.insert(*ind_p, if tokens == usize::MAX {tokens} else { tokens+*capacity });
                    }
                    if let Some(covers) = has_cover(&tran.state2conf, &path, &new_config) {
                        if covers.is_empty() {
                            continue;
                        }
                        println!("{:?}", covers);
                        for (c) in covers {
                            new_config.insert(c, usize::MAX);
                        }
                    }
                    if let Some(index_older) = tran.duplicate_config(&new_config) {
                        println!("{:?} linking to previous state: {:?}", index_old, index_older);
                        if index_old != index_older {
                            tran.insert_transition(index_old, index_older);
                            continue;
                        }
                    }
                    index_all += 1;
                    path_map.insert(index_all, index_old);
                    all.push((index_all, new_config.clone()));
                    tran.insert_mapping(index_all, new_config.clone());
                    tran.insert_transition(index_old, index_all);
                    println!("{:?}", (index_old, index_all, new_config))
                }
            }
            tran.insert_fireable(index_old, fireable);
        }
        tran.merge_graph();
        tran
    }
}

fn find_path(path_map: &HashMap<usize, usize>, end: usize) -> Vec<usize> {
    if end == 0 {
        return vec![0];
    }
    let mut res = Vec::new();
    let mut cur = end;

    while let Some(&last) = path_map.get(&cur) {
        res.push(cur);
        if last == 0 {
            break;
        }
        cur = last;
    }
    res.push(0);
    res
}

fn is_covered(m1: &Config, m2: &Config) -> Option<Vec<usize>> {
    let mut res = Vec::new();

    for (&k, &v1) in m1 {
        match m2.get(&k) {
            Some(&v2) if v2 >= v1 => {
                if v2 > v1 {
                    res.push(k);
                }
            }
            _ => return None, // Either key not found in m2, or m2[k] < v
        }
    }

    Some(res)
}

fn has_identical(state2conf: &HashMap<usize, Config>, path: &Vec<usize>, config_end: &Config) -> bool {
    for (state) in path.iter().skip(1) {
        if let Some(&ref config)  = state2conf.get(&state) {
            if *config == *config_end {
                return true
            }
        }
    }
    false
}

fn has_cover(state2conf: &HashMap<usize, Config>, path: &Vec<usize>, config_end: &Config) -> Option<Vec<usize>> {
    for (state) in path {
        if let Some(&ref config)  = state2conf.get(&state) {
            if let Some(cover) = is_covered(&config, &config_end) {
                return Some(cover);
            }
        }
    }
    None
}

fn sum(map: HashMap<usize, usize>) -> usize {
    let mut res = 0;
    for (_, v) in map {
        res += v;
    }
    res
}


impl Place {
    pub fn get_tokens(&self) -> usize {
        match self.init {
            Marking::Plain(n) => {n}
            Marking::Colored => {0}
        }
    }
}

impl PTNet {
    // pub fn get_by_index(&self, ind: usize) {
    //     self.index_map.get_by_left(ind)
    // }
}