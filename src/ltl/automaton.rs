use std::collections::HashMap;
use std::default::Default;

// #[derive(Default)]
pub struct Automaton<S, A> {
    pub states: Vec<S>,
    pub init_states: Vec<S>,
    pub acc_states: Vec<S>,
    pub alphabet: Vec<A>,
    pub transitions: HashMap<(S, A), S>,
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