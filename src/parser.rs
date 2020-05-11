use std::collections::HashMap;

mod tokenizer;
use tokenizer::*;

#[derive(Debug, PartialEq, Eq)]
enum StateType {
    AtomicState,
    CompoundState,
    FinalState,
    ParallelState,
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
    tokens: Vec<Token<'a>>,
}

// looks like i can't write this method zero_or_one in rust
// It needs a mutable reference to it's self type. But the function it takes
// which parses the current token also needs mutable reference to self. That
// is not allowed in rust. 2 things can't have mutable reference to the same
// thing
// The only solution seems to be not to mutate offset but instead return
// new offset from each parser.
// The return type sends the offset as a return value in both success and fail
// case since both are actually success for zero_or_one. No match is also what
// this parser is supposed to treat as a success.
fn zero_or_one<T, F>(offset: usize, mut f: F) -> (usize, Option<(usize, T)>)
where
    F: Fn(usize) -> Option<(usize, T)>,
{
    if let Some(x) = f(offset) {
        let (new_offset, _) = x;
        return (new_offset, Some(x));
    }

    (offset, None)
}

// TODO: these parser combinators are not using self at all. We can move
// them out of the impl methods
fn zero_or_more<T, F>(offset: usize, mut f: F) -> (usize, Option<(usize, Vec<T>)>)
where
    F: Fn(usize) -> Option<(usize, T)>,
{
    let mut new_offset = offset;
    let mut parsed_values = vec![];

    while let Some(x) = f(new_offset) {
        let (newer_offset, v) = x;
        new_offset = newer_offset;
        parsed_values.push(v);
    }

    if (parsed_values.len() > 0) {
        return (new_offset, Some((new_offset, parsed_values)));
    } else {
        return (offset, None);
    }
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
        Parser { tokens: vec![] }
    }

    fn identifier(&self, offset: usize) -> Option<(usize, &'a str)> {
        if let TokenType::Identifier(text) = self.tokens[offset].typ {
            return Some((offset + 1, text));
        }

        None
    }

    fn parallel_state(&self, offset: usize) -> Option<(usize, bool)> {
        if self.tokens[offset].typ == TokenType::ParallelState {
            return Some((offset + 1, true));
        }

        None
    }

    fn final_state(&self, offset: usize) -> Option<(usize, bool)> {
        if self.tokens[offset].typ == TokenType::FinalState {
            return Some((offset + 1, true));
        }

        None
    }

    fn initial_state(&self, offset: usize) -> Option<(usize, bool)> {
        if self.tokens[offset].typ == TokenType::InitialState {
            return Some((offset + 1, true));
        }

        None
    }

    fn indent(&self, offset: usize) -> Option<(usize, bool)> {
        if self.tokens[offset].typ == TokenType::Indent {
            return Some((offset + 1, true));
        }

        None
    }

    fn dedent(&self, offset: usize) -> Option<(usize, bool)> {
        if self.tokens[offset].typ == TokenType::Dedent {
            return Some((offset + 1, true));
        }

        None
    }

    // All our parsers will return an Option. If parsing was successful, return
    // Some<SomeData> else return None. We can probably write generic functions
    // which can handle these Option<T> return values. Functions like zero_or_more
    // one_or_more etc.
    // We can use the question mark (?) operator
    // self.identifier()?;
    fn state_parser(&mut self, offset: usize) -> Option<StateNode<'a>> {
        println!("offset {:?}", offset);
        let mut new_offset = offset;
        // we have to find a better way of passing on the None values from
        // one parser to another. Panicing will not do.
        let (offset, id) = self.identifier(offset)?;
        let mut is_parallel_state = false;
        let (offset, is_parallel_state_option) =
            zero_or_one(offset, |offset| self.parallel_state(offset));

        if let Some(_) = is_parallel_state_option {
            is_parallel_state = true;
        }

        let mut is_final_state = false;
        let (offset, is_final_state_option) =
            zero_or_one(offset, |offset| self.final_state(offset));

        if let Some(_) = is_final_state_option {
            is_final_state = true;
        }

        let mut is_initial_state = false;
        let (offset, is_initial_state_option) =
            zero_or_one(offset, |offset| self.initial_state(offset));

        if let Some(_) = is_initial_state_option {
            is_initial_state = true;
        }

        let mut is_indent_there = false;
        let (offset, is_indent_there_option) = zero_or_one(offset, |offset| self.indent(offset));

        if let Some(_) = is_indent_there_option {
            is_indent_there = true;
        }

        if (is_indent_there) {
            zero_or_more(offset, |offset| self.dedent(offset));
        }

        Some(StateNode {
            id: "1",
            typ: StateType::AtomicState,
            initial: Some("abc"),
            is_initial: false,
            on: HashMap::new(),
            states: HashMap::new(),
        })
    }

    // Our parser returns a Result type. Which means it returns an error if the
    // parsing fails.
    // TODO: Define a custom error struct
    pub fn parse(&mut self, input_str: &'a str) -> Result<StateNode<'a>, &'a str> {
        self.tokens = tokenize(input_str)
            .into_iter()
            .filter(|t| !matches!(t.typ, TokenType::Comment(_)))
            .collect();

        if let Some(ast) = self.state_parser(0) {
            return Ok(ast);
        }

        Err("MyParser: Error parsing string")
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
            states: HashMap::new(),
        };

        assert_eq!(expected_ast, ast);
    }
}
