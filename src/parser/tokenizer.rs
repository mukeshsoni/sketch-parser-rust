extern crate regex;

use regex::Regex;

// How do i print my structs and enums?
// There are 2 ways
// 1. We can implement the Debug trait
// 2. We can use the derive attribute. An attribute is used like in the format
// below and is used to add some meta data to the program for the compiler.
#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    Identifier,
    Condition,
    Indent,
    Dedent,
    Unknown,
    Comment,
    ParallelState,
    FinalState,
    InitialState,
    Actions,
    TransitionArrow,
}

#[derive(Debug)]
pub struct Token {
    pub text: String,
    pub line: usize,
    pub col: usize,
    pub token_type: TokenType,
}

fn token_without_text(line: usize, col: usize, token_type: TokenType) -> Token {
    Token {
        token_type,
        line,
        col,
        text: "".to_string(),
    }
}

fn parallel_state_token(line: usize, col: usize) -> Token {
    token_without_text(line, col, TokenType::ParallelState)
}

fn final_state_token(line: usize, col: usize) -> Token {
    token_without_text(line, col, TokenType::FinalState)
}

fn initial_state_token(line: usize, col: usize) -> Token {
    token_without_text(line, col, TokenType::InitialState)
}

fn arrow_token(line: usize, col: usize) -> Token {
    token_without_text(line, col, TokenType::TransitionArrow)
}

fn unknown_token(line: usize, col: usize) -> Token {
    Token {
        // Why do i have to use the to_string method here?
        text: "unknown".to_string(),
        token_type: TokenType::Unknown,
        line,
        col,
    }
}

fn comment_token(line: usize, offset: usize, input: &str) -> Token {
    Token {
        token_type: TokenType::Comment,
        text: input[offset..].to_string(),
        line,
        col: offset,
    }
}

fn condition_token(line: usize, mut offset: usize, input: &str) -> Token {
    let input_as_chars: Vec<char> = input.chars().collect();

    let mut c = input_as_chars[offset];

    while !is_identifier_start(c) {
        c = input_as_chars[offset];
        offset += 1;
    }
    offset -= 1;

    identifier_token(line, offset, input)
}

fn action_token(line: usize, offset: usize, input: &str) -> Token {
    Token {
        token_type: TokenType::Actions,
        line,
        col: offset,
        text: condition_token(line, offset, input).text,
    }
}

fn identifier_token(line: usize, offset: usize, input: &str) -> Token {
    let text = input[offset..].split(' ').collect::<Vec<&str>>()[0].to_string();

    // println!("{:?} {:?}", input, text);

    Token {
        token_type: TokenType::Identifier,
        text,
        line,
        col: offset,
    }
}

fn is_identifier_start(c: char) -> bool {
    // How do i use regex in rust?
    // rust does not support regular expressions (regex) out of the box
    // We have to use an external library
    let re = Regex::new(r"[#a-zA-Z0-9_\.]").unwrap();

    // How do i convert character to string?
    // is_match method expects a string
    // Ans -> use `to_string()`
    // But to_string returns a String, but we want a &str
    // How do i convert String to &str?
    re.is_match(&c.to_string()[..])
}

fn indent_dedent_tokens(
    line_number: usize,
    indent_stack: &mut Vec<usize>,
    line: &Vec<char>,
) -> (usize, Vec<Token>) {
    let re = Regex::new(r" ").unwrap();
    let mut offset = 0;
    let mut current_indent_level: usize = 0;
    let mut tokens: Vec<Token> = Vec::new();

    while re.is_match(&line[offset].to_string()[..]) {
        current_indent_level += 1;
        offset += 1;
    }

    if current_indent_level > 0 {
        match indent_stack.last() {
            None => {
                // it's the first indent we have encountered
                // or - all indents have been deindented
                indent_stack.push(current_indent_level);
                tokens.push(token_without_text(line_number, 0, TokenType::Indent));
            }
            Some(&prev_indent_level) => {
                if prev_indent_level < current_indent_level {
                    indent_stack.push(current_indent_level);
                    tokens.push(token_without_text(line_number, 0, TokenType::Indent));
                } else if prev_indent_level > current_indent_level {
                    println!("indent stack {:?}", indent_stack);
                    while indent_stack.len() > 0 {
                        let prev_indent = indent_stack.pop().unwrap();
                        if prev_indent > current_indent_level {
                            tokens.push(token_without_text(line_number, 0, TokenType::Dedent));
                        }
                    }
                }
            }
        }
    }

    (offset, tokens)
}

pub fn tokenize(input: &str) -> Vec<Token> {
    // How to write a comment in rust. Like we do in javascript.
    // Rust comments are more than comments though. We can write whole tests
    // inside a comment for a function.

    // How to split a string into lines? using split function. But it's not
    // that simple. `split` return an iterator and if we want the lines as
    // as vector or array, we have to use the collect method of the iterator
    // The syntax for collect gets weird when we want to tell it the type to
    // be returned
    // we can avoid specifying it in that weird way in collect by specifying
    // type of lines
    // let lines: Vec<&str> = input.split("\n").collect();
    let lines = input.split("\n").collect::<Vec<&str>>();
    // How to create an empty vector?
    let mut tokens: Vec<Token> = Vec::new();
    // line and col keep track of the current line and col number
    let mut line_number = 0;
    // offset keeps track of the current character position in the line
    let mut offset;
    let mut indent_stack: Vec<usize> = Vec::new();

    // TODO: can we write it as input.split("\n").map().flatten().collect()?
    // The map function returns the list of tokens in one line

    // writing for line in lines would mean moving lines inside the for block
    // and hence not being available outside it
    for line in &lines {
        // how to convert a string into a list of characters? Use chars method
        // on string. Again, chars returns an iterator instead of a vector.
        // This seems to be a common pattern. Whenever a javascript programmer
        // expects an array of something, rust functions/methods return an
        // iterator.
        // Probably my tokenize function should also return an iterator of
        // Tokens instead of a Vector of tokens
        let char_vec: Vec<char> = line.chars().collect();

        let (new_offset, indent_tokens) =
            indent_dedent_tokens(line_number, &mut indent_stack, &char_vec);
        offset = new_offset;

        tokens.extend(indent_tokens);

        while offset < char_vec.len() {
            let c = char_vec[offset];
            match c {
                // How to create new values of a struct?
                '%' => {
                    tokens.push(comment_token(line_number, offset, line));
                    break;
                }
                '&' => {
                    tokens.push(parallel_state_token(line_number, offset));
                    offset += 1;
                }
                '$' => {
                    tokens.push(final_state_token(line_number, offset));
                    offset += 1;
                }
                '*' => {
                    tokens.push(initial_state_token(line_number, offset));
                    offset += 1;
                }
                ';' => {
                    let condition = condition_token(line_number, offset, line);
                    offset += condition.text.len();
                    tokens.push(condition);
                }
                '-' => {
                    if offset < line.len() - 1 && char_vec[offset + 1] == '>' {
                        tokens.push(arrow_token(line_number, offset));
                        offset += 2;
                    } else {
                        tokens.push(unknown_token(line_number, offset));
                        offset += 1;
                    }
                }
                '>' => {
                    let actions = action_token(line_number, offset, line);
                    println!("action {:?}", actions.text);
                    offset += actions.text.len();
                    tokens.push(actions);
                }
                c if is_identifier_start(c) => {
                    let identifier = identifier_token(line_number, offset, line);
                    offset += identifier.text.len();
                    tokens.push(identifier);
                }
                c if c.is_whitespace() => offset += 1,
                _ => {
                    tokens.push(unknown_token(line_number, offset));
                    offset += 1;
                }
            }
        }

        line_number += 1;
    }

    // pop out all the Dedents
    while indent_stack.len() > 0 {
        indent_stack.pop();
        tokens.push(token_without_text(line_number, 0, TokenType::Dedent));
    }

    // println!("tokens: {:?}", tokens.len());
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);

        // how to write multiline string literal? You just write it.
        let input = "abc
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

        // println! or print! does not work for successful tests. rust test
        // clears all stdout output from the program if the test passes.
        // 2 ways to check our println! statements
        // 1. Fail the test manually. E.g. assert_eq!(1, 0)
        // 2. use the --nocapture flag while running the tests
        let tokens = tokenize(input);

        println!(
            "{:#?} {:#?}",
            tokens.len(),
            // how do i find an element in a vector?
            tokens
                .iter()
                .find(|&t| t.token_type == TokenType::Condition)
        );

        //
        assert_eq!(tokens.len(), 39);
    }
}