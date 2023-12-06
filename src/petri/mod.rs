use bimap::BiMap;
use petgraph::Graph;
use petgraph::graph::*;
use std::collections::HashMap;

pub mod parser;

#[derive(Debug, Clone)]
pub struct Arc {
    pub id: String,
    pub ty: ArcTy,
}

#[derive(Debug, Clone)]
pub enum ArcTy {
    Plain(usize),
    Strct,
}

#[derive(Debug, Clone)]
pub enum Marking {
    Plain(usize),
    Colored,
}

#[derive(Debug, Clone)]
pub struct Place {
    pub name: String,
    pub page: String,
    pub init: Marking,
    pub producers: Vec<(usize, usize)>,
    pub consumers: Vec<(usize, usize)>,
}

#[derive(Debug, Clone)]
pub struct Transition {
    pub name: String,
    pub page: String,
    pub conditions: Vec<(usize, usize)>,
    pub effects: Vec<(usize, usize)>,
}

#[derive(Default, Debug, Clone)]
pub struct PTNet {
    pub name: String,
    pub pages: Vec<String>,
    pub index_map: BiMap<String, usize>,
    pub places: HashMap<usize, Place>,
    pub transitions: HashMap<usize, Transition>,
    pub arcs: HashMap<(usize, usize), Vec<Arc>>,
    pub arcs_cnt: usize,
}

impl PTNet {
    pub fn insert_place(&mut self, place: Place) {
        self.index_map.insert(place.name.clone(), self.index_map.len());
        self.places.insert(self.index_map.len() - 1, place);
    }

    pub fn insert_transition(&mut self, transition: Transition) {
        self.index_map.insert(transition.name.clone(), self.index_map.len());
        self.transitions.insert(self.index_map.len() - 1, transition);
    }

    pub fn insert_arc(&mut self, arc: Arc, src: String, dst: String) {
        let src_idx = self.index_map.get_by_left(&src).unwrap().clone();
        let dst_idx = self.index_map.get_by_left(&dst).unwrap().clone();
        if self.is_place(&src_idx) {
            // Arc from place to transition
            let mut place = self.places.get_mut(&src_idx).unwrap();
            let mut transition = self.transitions.get_mut(&dst_idx).unwrap();
            let threshold = if let ArcTy::Plain(th) = &arc.ty {th.clone()} else {1};
            place.consumers.push((dst_idx.clone(), threshold.clone()));
            transition.conditions.push((src_idx.clone(), threshold.clone()));
        }
        else {
            // Arc from transition to place
            let mut place = self.places.get_mut(&dst_idx).unwrap();
            let mut transition = self.transitions.get_mut(&src_idx).unwrap();
            let threshold = if let ArcTy::Plain(th) = &arc.ty {th.clone()} else {1};
            place.producers.push((src_idx.clone(), threshold.clone()));
            transition.effects.push((dst_idx.clone(), threshold.clone()));
        }
        if !self.arcs.contains_key(&(src_idx.clone(), dst_idx.clone())) {
            self.arcs.insert((src_idx.clone(), dst_idx.clone()), vec![arc]);
        }
        else {
            let mut key = self.arcs.get_mut(&(src_idx.clone(), dst_idx.clone())).unwrap();
            key.push(arc);
        }
        self.arcs_cnt += 1;
    }

    fn is_place(&self, idx: &usize) -> bool {
        self.places.contains_key(idx)
    }

    pub fn reachability_graph(&self) -> Graph::<HashMap<usize, usize>, String> {
        // Construct reachability graph, nodes for markings and edges for transition name
        let mut res = Graph::<HashMap<usize, usize>, String>::new();
        let mut node_map = HashMap::<NodeIndex, HashMap<usize, usize>>::new();
        // Fetch initial marking
        let mut init_marking = HashMap::new();
        for (k, p) in self.places.iter() {
            let val = if let Marking::Plain(size) = p.init {
                size
            }
            else { 0 };
            init_marking.insert(k.clone(), val.clone());
        }
        // println!("init: {:?}", init_marking);
        let mut queue = Vec::new();
        let idx = res.add_node(init_marking.clone());
        queue.push(idx.clone());
        node_map.insert(idx, init_marking);
        while queue.len() != 0 {
            let src_idx = queue.remove(0);
            let marking = node_map.get(&src_idx).unwrap().clone();
            for (k, t) in self.transitions.iter() {
                let mut fireable = true;
                for (place, size) in t.conditions.iter() {
                    if marking[place] < *size {
                        fireable = false;
                        break;
                    }
                }
                if fireable {
                    println!("transition {:?} is fireable", t.name);
                    println!("old marking: {:?}", marking);
                    let mut new_marking = marking.clone();
                    for (k, num) in t.conditions.iter() {
                        let place = new_marking.get_mut(k).unwrap();
                        *place -= num;
                    }
                    for (k, num) in t.effects.iter() {
                        let place = new_marking.get_mut(k).unwrap();
                        *place += num;
                    }
                    println!("new marking: {:?}", new_marking);
                    let mut added = false;
                    for (k, m) in node_map.iter() {
                        if *m == new_marking {
                            // println!("repeated marking: {:?}", new_marking);
                            added = true;
                            res.add_edge(src_idx.clone(), k.clone(), format!("Fireability({:?})", t.name));
                            break;
                        }
                    }
                    if !added {
                        // println!("new marking: {:?}", new_marking);
                        let dst_idx = res.add_node(new_marking.clone());
                        println!("new node: {:?} {:?}", dst_idx, new_marking);
                        node_map.insert(dst_idx.clone(), new_marking.clone());
                        res.add_edge(src_idx.clone(), dst_idx.clone(), format!("Fireability({:?})", t.name));
                        queue.push(dst_idx);
                        break;
                    }
                }
            }
        }
        println!("res: {:?}", res);
        res
    }
}