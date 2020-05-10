// use std::collections::HashMap;

mod tokenizer;
use tokenizer::*;

// enum StateType {
    // AtomicState,
    // CompoundState,
    // FinalState,
    // ParallelState
// }

// struct TransitionNode {
    // target: String,
    // cond: Option<String>,
    // action: Option<String>,
// }

// struct StateNode {
    // id: String,
    // typ: StateType,
    // initial: Option<String>,
    // is_initial: bool,
    // on: HashMap<String, TransitionNode>,
    // states: HashMap<String, StateNode>,
// }

pub fn parse(input: &str) {
    let tokens = tokenize(input);

    println!("got tokens {:#?}", tokens);
}
