use bimap::BiMap;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Arc {
    pub id: String,
    pub ty: ArcTy,
}

#[derive(Debug)]
pub enum ArcTy {
    Plain(usize),
    Strct,
}

#[derive(Debug)]
pub enum Marking {
    Plain(usize),
    Colored,
}

#[derive(Debug)]
pub struct Place {
    pub name: String,
    pub page: String,
    pub init: Marking,
    pub producers: Vec<(usize, usize)>,
    pub consumers: Vec<(usize, usize)>,
}

#[derive(Debug)]
pub struct Transition {
    pub name: String,
    pub page: String,
    pub conditions: Vec<(usize, usize)>,
    pub effects: Vec<(usize, usize)>,
}

#[derive(Default, Debug)]
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
}