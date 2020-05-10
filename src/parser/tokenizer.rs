use regex::Regex;

// How do i print my structs and enums?
// There are 2 ways
// 1. We can implement the Debug trait
// 2. We can use the derive attribute. An attribute is used like in the format
// below and is used to add some meta data to the program for the compiler.
// TODO: Use tuple where required to store the text along with token type
// E.g. Identifier(String);
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenType {
    Identifier(String),
    Condition(String),
    Indent,
    Dedent,
    Unknown(String),
    Comment(String),
    ParallelState,
    FinalState,
    InitialState,
    Actions,
    TransitionArrow,
}

#[derive(Debug)]
pub struct Token {
    pub line: usize,
    pub col: usize,
    pub token_type: TokenType,
}

fn token_without_text(line: usize, col: usize, token_type: TokenType) -> Token {
    Token {
        token_type,
        line,
        col,
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
        token_type: TokenType::Unknown("unknown".to_string()),
        line,
        col,
    }
}

fn comment_token(line: usize, offset: usize, input: &str) -> Token {
    let text = input[offset..].to_string();

    Token {
        token_type: TokenType::Comment(text.clone()),
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
    let text = input[offset..].split(' ').collect::<Vec<&str>>()[0].to_string();

    return Token {
        token_type: TokenType::Condition(text.clone()),
        ..identifier_token(line, offset, input)
    }
}

fn action_token(line: usize, offset: usize) -> Token {
    Token {
        token_type: TokenType::Actions,
        line,
        col: offset,
    }
}

fn identifier_token(line: usize, offset: usize, input: &str) -> Token {
    let text = input[offset..].split(' ').collect::<Vec<&str>>()[0].to_string();

    // println!("{:?} {:?}", input, text);

    Token {
        token_type: TokenType::Identifier(text.clone()),
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

// this is the key function in the tokenizer
// because our language is indent based. Parsing it is very tricky.
// This is the whole reason i had to write a tokenizer in a recursive descent 
// parser.
// This step in the tokenizer makes life much simpler for the parser.
fn indent_dedent_tokens(
    line_number: usize,
    indent_stack: &mut Vec<usize>,
    line: &Vec<char>,
) -> (usize, Vec<Token>) {
    let mut offset = 0;
    let mut current_indent_level: usize = 0;
    let mut tokens: Vec<Token> = Vec::new();

    while line[offset] == ' ' {
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

                    // TODO: we should implement some syntax error checking 
                    // in this part. E.g. previous indent level is 2 and the 
                    // current one is 6. It's too much.
                    // Or the one below
                    // const dedentLevelInStack = indentStack.find(
                      // (n) => n === currentIndentLevel,
                    // );

                    // // any dedent/outdent must match some previous indentation level.
                    // // otherwise it's a syntax error
                    // if (dedentLevelInStack === undefined) {
                      // throw new Error('Invalid indentation');
                    // }


                    while indent_stack.len() > 0 {
                        let prev_indent = indent_stack.pop().unwrap();
                        // keep popping indentation levels from indent dedentLevelInStack
                        // until we reach the current indent level
                        // push those many dedent tokens to tokenizer
                        if prev_indent > current_indent_level {
                            tokens.push(token_without_text(line_number, 0, TokenType::Dedent));
                        } else {
                            break;
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
    // be returned.
    // let lines =  input.split("\n").collect<Vec<&str>>();
    // Or we can annotate the variable to which the value of
    // we can avoid specifying it in that weird way in collect by specifying
    // type of lines
    let lines: Vec<&str> = input.split("\n").collect();
    // How to create an empty vector?
    let mut tokens: Vec<Token> = Vec::new();
    // line and col keep track of the current line and col number
    let mut line_number = 0;
    // offset keeps track of the current character position in the line
    let mut offset;
    let mut indent_stack: Vec<usize> = Vec::new();

    // TODO: can we write it as input.split("\n").map().flatten().collect()?
    // The map function returns the list of tokens in one line

    // writing `for line in lines` would mean moving lines inside the for block
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

        // extend extends a collection with contents of an iterator
        tokens.extend(indent_tokens);

        // why can we split the char_vec at offset and then iterate on the line
        // from that point?
        // Because on every loop the offset changes by more than or equal to 1
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
                    // let TokenType::Condition(text) = condition.token_type;
                    let text = match condition.token_type.clone() {
                        TokenType::Condition(t) => t,
                        _ => " ".to_string(),
                    };
                    offset += text.len();
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
                    let actions = action_token(line_number, offset);
                    offset += 1;
                    tokens.push(actions);
                }
                c if is_identifier_start(c) => {
                    let identifier = identifier_token(line_number, offset, line);
                    let text = match identifier.token_type.clone() {
                        TokenType::Identifier(t) => t,
                        _ => " ".to_string(),
                    };
                    offset += text.len();
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

    static INVALID_INPUT_STR: &str = "abc
  def -> lmn
      pqr
    stm";

    #[test]
    fn it_works() {
        // how to write multiline string literal? You just write it.
        // println! or print! does not work for successful tests. rust test
        // clears all stdout output from the program if the test passes.
        // 2 ways to check our println! statements
        // 1. Fail the test manually. E.g. assert_eq!(1, 0)
        // 2. use the --nocapture flag while running the tests
        let tokens = tokenize(INPUT);

        println!(
            "{:#?} {:#?}",
            tokens.len(),
            "placeholder",
            // how do i find an element in a vector?
            // tokens
                // .iter()
                // .find(|&t| t.token_type == TokenType::Condition(_))
        );

        //
        assert_eq!(tokens.len(), 39);
    }

    #[test]
    fn test_token_type() {
        let tokens = tokenize(INPUT);

        // using {:?} prints structures other than the basic ones
        // using {:#?} pretty prints
        println!("tokens {:#?}", tokens);
        println!("token {:?}", tokens[21]);

        assert_eq!(tokens[1].token_type, TokenType::Comment("% some comment".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::Indent);
        assert_eq!(tokens[11].token_type, TokenType::Indent);
        assert_eq!(tokens[21].token_type, TokenType::Dedent);
    }

}
