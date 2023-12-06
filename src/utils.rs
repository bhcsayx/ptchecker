// use pnets::standard;
// use pnets_pnml::pnml::Pnml;
// use serde_xml_rs::from_str;
// use quick_xml::Reader;
// use quick_xml::events::Event;
// use quick_xml::name::QName;
use std::convert::TryInto;
use std::path::Path;
use std::collections::{HashMap, HashSet};

// pub fn pnets_read_ptnets_from(path: &str) -> Result<Vec<standard::Net>, Box<dyn Error>> {
//     let raw_string = fs::read_to_string(path)?;
//     let pnml: Pnml = quick_xml::de::from_str(raw_string.as_str())?;
//     let nets: Vec<standard::Net> = (&pnml).try_into()?;
//     // let pnml: Pnml = (&nets).into();
//     for net in nets.iter() {
//         println!("net: {:?}", net);
//     }
//     // println!("{:?}", quick_xml::se::to_string(&pnml));
//     Ok(nets)
// }

pub fn validate_path(input: &str) -> bool {
    let path = Path::new(input);
    if !path.exists() {
        return false;
    }
    let model_path = path.join("model.pnml");
    if !model_path.exists() {
        return false;
    }
    let ltl_fire_path = path.join("LTLFireability.txt");
    if !ltl_fire_path.exists() {
        return false;
    }
    let ltl_card_path = path.join("LTLCardinality.xml");
    if !ltl_card_path.exists() {
        return false;
    }
    let ctl_fire_path = path.join("CTLFireability.txt");
    if !ctl_fire_path.exists() {
        return false;
    }
    let ctl_card_path = path.join("CTLCardinality.xml");
    if !ctl_card_path.exists() {
        return false;
    }
    return true;
}

#[derive(Debug, Clone)]
pub struct Automaton<S, A> {
    pub states: Vec<S>,
    pub init_states: Vec<S>,
    pub acc_states: Vec<S>,
    pub alphabet: Vec<A>,
    pub transitions: HashMap<S, Vec<(A, S)>>, // Vec of states as we may have non-deterministic usages
}

impl<S, A> Automaton<S, A> {
    pub fn new() -> Automaton<S, A> {
        Automaton::<S, A> {
            states: Vec::new(),
            init_states: Vec::new(),
            acc_states: Vec::new(),
            alphabet: Vec::new(),
            transitions: HashMap::new()
        }
    }
}

pub fn powerset<T: std::cmp::Eq + std::hash::Hash>(input: &HashSet<T>) -> Vec<HashSet<T>> where T: Clone {
    let mut res = Vec::new();
    let elems = Vec::from_iter(input.into_iter());
    let length = usize::pow(2, elems.len().try_into().unwrap());
    for i in 0..length {
        let mut set = HashSet::new();
        for j in 0..elems.len() {
            if (i & (1 << j)) != 0 {
                set.insert(elems[j].clone());
            }
        }
        res.push(set.clone());
    }
    res
}