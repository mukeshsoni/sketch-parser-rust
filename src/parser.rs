use std::collections::HashMap;

mod tokenizer;
use tokenizer::*;

#[derive(Debug, PartialEq, Eq)]
enum StateType {
    AtomicState,
    CompoundState,
    FinalState,
    ParallelState
}

#[derive(Debug, PartialEq, Eq)]
pub struct TransitionNode<'a> {
    target: &'a str,
    cond: Option<&'a str>,
    action: Option<&'a str>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StateNode<'a> {
    id: &'a str,
    typ: StateType,
    initial: Option<&'a str>,
    is_initial: bool,
    on: HashMap<&'a str, TransitionNode<'a>>,
    states: HashMap<&'a str, StateNode<'a>>,
}

// TODO: This return value is not enough. We need to consume the token, which 
// means updating the offset. Each parser can change the offset by different
// amount. We can either return a tuple (new_offset, Option<&'a str>) or we
// can define these methods as Type methods for a Parser type.
// struct Parser<'a> {
//     offset: usize,
//     tokens: Vec<Token<'a>>,
//     input_str: &'a str,
// }
//
// impl Parser {
//   fn new(input_str: &str) {
//      Parser {
//          offset: 0,
//          input_str
//      }
//   }
//
//   fn parse(&self) {
//      let tokens = tokenize(self.input_str);
//
//      // if we store the token iterator then we won't have to store the offset
//      // at each stage. We also want the ability to peek into the tokens in 
//      // case we want to backtrack.
//      self.tokens = tokens.iter();
//   }
//
//   fn condition(&self) {
//      if let Token::Condition(text) == self.tokens.peek() {
//          self.tokens.next();
//          Some(text)
//      }
//
//      None;
//   }
//
//   OR
//   
//   // This one does not use the token iterator. Just uses offset to keep track
//   // of next token to consume. And consume updates the offset internally.
//   fn condition(&self) {
//      if let Token::Condition(text) == self.tokens[self.offset] {
//          self.consume();
//          Some(text)
//      }
//
//      None;
//   }
// }

// But the above will not help us with the offset, will it? How will a parser
// know when to stop peeking and start advancing the iterator? It might do so 
// wrong and the whole chain becomes buggy from that point.
// What if each parser returns Option<(new_offset, ParserResult)>? Then each 
// parser has the responsibility of adjusting the offset after calling other
// parsers internally. That's not good. Instead only the higher level parser
// combinators (like oneOrAnother or zero_or_more) should know about the 
// concept of offset or index.
// Ok, so individual parsers just consume tokens if they see it fit to do so
// And only parser combinators worry about backtracking, which involves putting
// the offset/index back to some previous position.

fn identifier<'a>(tokens: &Vec<Token<'a>>, offset: usize) -> Option<&'a str> {
    if let Token::Identifier(text) = tokens[offset] {
       return Some(text);
    }

    None
}

fn parallel_state<'a>(tokens: &Vec<Token<'a>>, offset: usize) -> bool {
    tokens[offset] == Token::ParallelState
}

// All our parsers will return an Option. If parsing was successful, return
// Some<SomeData> else return None. We can probably write generic functions
// which can handle these Option<T> return values. Functions like zero_or_more
// one_or_more etc.
fn state_parser<'a>(tokens: &Vec<Token<'a>>, offset: usize) -> Option<StateNode<'a>> {
    let id = identifier(tokens, offset).expect("Expected a state name");

    println!("id {:?}", id);
    Some(StateNode {
        id: "1",
        typ: StateType::AtomicState,
        initial: Some("abc"),
        is_initial: false,
        on: HashMap::new(),
        states: HashMap::new()
    })
}

// Our parser returns a Result type. Which means it returns an error if the 
// parsing fails. 
// TODO: Define a custom error struct
pub fn parse<'a>(input: &'a str) -> Result<StateNode<'a>, &'a str> {
    let tokens = tokenize(input);

    if let Some(ast) = state_parser(&tokens, 0) {
        return Ok(ast);
    }

    Err("Error parsing string")
}

#[cfg(test)]
mod tests {
    use super::*;

    static INPUT: &str = "abc
    % some comment
      def -> lmn
      pasta -> noodles %more comment
      ast&*
        opq -> rst; ifyes
        uvw -> #abc.lastState
        nestedstate1
        nestedstate2*
      tried -> that > andDoThis
      lastState
        % trying out transient state
        -> ast; ifyes
        -> lastState; ifno";

    #[test]
    fn test_parser() {
        let ast = parse(INPUT).unwrap();

        let expected_ast: StateNode = StateNode {
            id: "1",
            typ: StateType::AtomicState,
            initial: Some("abc"),
            is_initial: false,
            on: HashMap::new(),
            states: HashMap::new()
        };

        assert_eq!(expected_ast, ast);
    }
}
