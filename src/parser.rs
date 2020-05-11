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
//      if let TokenType::Condition(text) == self.tokens.peek() {
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
//      if let TokenType::Condition(text) == self.tokens[self.offset] {
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

struct Parser<'a> {
    offset: usize,
    tokens: Vec<Token<'a>>,
}

impl<'a> Parser<'a> {
    // Question: Should the input str be sent when creating a new parser or 
    // during the call to parse? If we send it during Parser creation, we have
    // to keep creating new parsers for every new parse.
    // If we send it during parse call, what is even the point of having a 
    // new method? We can directly call Parser::parse(input_str). Also, we
    // won't have to store input_str in the struct any more. We need it to
    // only get the tokens. The problem is, these methods are defined on Parser
    // So we need an instance of Parser to call these methods. So either the
    // user has to create an instance of the Parser themselves, or we provide a
    // new method to do it for them
    // Sigh. Let's go ahead with passing input_str to parse as argument for now
    // At least we won't have to 
    // 1. Store the input_str inside the parser
    // 2. Won't have to create a new instance of Parser for every new parse
    fn new() -> Parser<'a> {
        Parser {
            offset: 0,
            tokens: vec![]
        }
    }

    fn consume(&mut self) {
        self.offset += 1;
    }

    fn identifier(&mut self) -> Option<&'a str> {
        if let TokenType::Identifier(text) = self.tokens[self.offset].typ {
            self.consume();
            return Some(text);
        }

        None
    }

    fn parallel_state(&mut self) -> Option<bool> {
        if self.tokens[self.offset].typ == TokenType::ParallelState {
            self.consume();
            return Some(true);
        }

        None
    }

    // All our parsers will return an Option. If parsing was successful, return
    // Some<SomeData> else return None. We can probably write generic functions
    // which can handle these Option<T> return values. Functions like zero_or_more
    // one_or_more etc.
    // We can use the question mark (?) operator
    // self.identifier()?;
    fn state_parser(&mut self) -> Option<StateNode<'a>> {
        // we have to find a better way of passing on the None values from 
        // one parser to another. Panicing will not do.
        let id = self.identifier()?;

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
    pub fn parse(&mut self, input_str: &'a str) -> Result<StateNode<'a>, &'a str> {
        self.offset = 0;
        self.tokens = tokenize(input_str);

        if let Some(ast) = self.state_parser() {
            return Ok(ast);
        }

        Err("Error parsing string")
    }
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
        let mut parser = Parser::new();
        let ast = parser.parse(INPUT).unwrap();

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
