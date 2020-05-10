use regex::Regex;

// How do i print my structs and enums?
// There are 2 ways
// 1. We can implement the Debug trait
// 2. We can use the derive attribute. An attribute is used like in the format
// below and is used to add some meta data to the program for the compiler.
// TODO: Use tuple where required to store the text along with token type
// E.g. Identifier(String);
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token<'a> {
    Identifier(&'a str),
    Condition(&'a str),
    Indent,
    Dedent,
    Unknown(&'a str),
    Comment(&'a str),
    Action(&'a str),
    ParallelState,
    FinalState,
    InitialState,
    TransitionArrow,
}

// Instead of having a Token type with line and col, maybe it's better to rename
// TokenType to Token, convert the lexer to an iterator where the parser keeps
// asking for the next token. And when the parser needs it, most probably during
// an error, the parser can ask the lexer for the current line and column
// It will also make our lexer more performant because it will not go through 
// the whole text and get all tokens. It will do so lazily. In most cases when 
// there's an error in the initial parts of the string or in the middle, it 
// won't waste time parsing the rest of the string.


fn comment_token(offset: usize, input: &str) -> Token {
    let text = &input[offset..];

    Token::Comment(text)
}

fn condition_token(mut offset: usize, input: &str) -> (usize, Token) {
    let input_as_chars: Vec<char> = input.chars().collect();

    let mut c = input_as_chars[offset];

    while !is_identifier_start(c) {
        c = input_as_chars[offset];
        offset += 1;
    }
    offset -= 1;
    let identifier = identifier_token(offset, input);

    let text = match identifier {
        Token::Identifier(t) => t,
        _ => " ",
    };

    (offset + text.len(), Token::Condition(text))
}

fn action_token(mut offset: usize, input: &str) -> (usize, Token) {
    let input_as_chars: Vec<char> = input.chars().collect();

    let mut c = input_as_chars[offset];

    while !is_identifier_start(c) {
        c = input_as_chars[offset];
        offset += 1;
    }
    offset -= 1;
    let identifier = identifier_token(offset, input);

    let text = match identifier {
        Token::Identifier(t) => t,
        _ => " ",
    };

    (offset + text.len(), Token::Action(text))
}

fn identifier_token(offset: usize, input: &str) -> Token {
    let text = &input[offset..].split(|c| is_identifier_start(c) == false).collect::<Vec<&str>>()[0];

    Token::Identifier(text)
}

fn is_identifier_start(c: char) -> bool {
    // How do i use regex in rust?
    // rust does not support regular expressions (regex) out of the box
    // We have to use an external library
    let re = Regex::new(r"^[#a-zA-Z0-9_\.]").unwrap();

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
fn indent_dedent_tokens<'a>(
    indent_stack: &mut Vec<usize>,
    line: &Vec<char>,
) -> (usize, Vec<Token<'a>>) {
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
                tokens.push(Token::Indent);
            }
            Some(&prev_indent_level) => {
                if prev_indent_level < current_indent_level {
                    indent_stack.push(current_indent_level);
                    tokens.push(Token::Indent);
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
                            tokens.push(Token::Dedent);
                        } else {
                            indent_stack.push(prev_indent);
                            break;
                        }
                    }
                }
            }
        }
    }
    let s: String = line.into_iter().collect();
    println!("line {:?} {:?} {:?} {:?}", s, indent_stack, current_indent_level, tokens);

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
    // let mut line_number = 0;
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
            indent_dedent_tokens(&mut indent_stack, &char_vec);
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
                    tokens.push(comment_token(offset, line));
                    break;
                }
                '&' => {
                    tokens.push(Token::ParallelState);
                    offset += 1;
                }
                '$' => {
                    tokens.push(Token::FinalState);
                    offset += 1;
                }
                '*' => {
                    tokens.push(Token::InitialState);
                    offset += 1;
                }
                ';' => {
                    let (new_offset, condition) = condition_token(offset, line);
                    offset  = new_offset;
                    tokens.push(condition);
                }
                '-' => {
                    if offset < line.len() - 1 && char_vec[offset + 1] == '>' {
                        tokens.push(Token::TransitionArrow);
                        offset += 2;
                    } else {
                        tokens.push(Token::Unknown("unknown"));
                        offset += 1;
                    }
                }
                '>' => {
                    let (new_offset, condition) = action_token(offset, line);
                    offset  = new_offset;
                    tokens.push(condition);
                }
                c if is_identifier_start(c) => {
                    let identifier = identifier_token(offset, line);
                    let text = match identifier.clone() {
                        Token::Identifier(t) => t,
                        _ => " ",
                    };
                    offset += text.len();
                    tokens.push(identifier);
                }
                c if c.is_whitespace() => offset += 1,
                _ => {
                    tokens.push(Token::Unknown("unknown"));
                    offset += 1;
                }
            }
        }

        // line_number += 1;
    }

    // pop out all the Dedents
    while indent_stack.len() > 0 {
        indent_stack.pop();
        tokens.push(Token::Dedent);
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

    // static INVALID_INPUT_STR: &str = "abc
  // def -> lmn
      // pqr
    // stm";

    #[test]
    fn it_works() {
        // how to write multiline string literal? You just write it.
        // println! or print! does not work for successful tests. rust test
        // clears all stdout output from the program if the test passes.
        // 2 ways to check our println! statements
        // 1. Fail the test manually. E.g. assert_eq!(1, 0)
        // 2. use the --nocapture flag while running the tests
        let tokens = tokenize(INPUT);
        let expected_tokens = vec![
            Token::Identifier("abc"),
            Token::Comment("% some comment"),
            Token::Indent,
            Token::Identifier("def"),
            Token::TransitionArrow,
            Token::Identifier("lmn"),
            Token::Identifier("pasta"),
            Token::TransitionArrow,
            Token::Identifier("noodles"),
            Token::Comment("%more comment"),
            Token::Identifier("ast"),
            Token::ParallelState,
            Token::InitialState,
            Token::Indent,
            Token::Identifier("opq"),
            Token::TransitionArrow,
            Token::Identifier("rst"),
            Token::Condition("ifyes"),
            Token::Identifier("uvw"),
            Token::TransitionArrow,
            Token::Identifier("#abc.lastState"),
            Token::Identifier("nestedstate1"),
            Token::Identifier("nestedstate2"),
            Token::InitialState,
            Token::Dedent,
            Token::Identifier("tried"),
            Token::TransitionArrow,
            Token::Identifier("that"),
            Token::Action("andDoThis"),
            Token::Identifier("lastState"),
            Token::Indent,
            Token::Comment("% trying out transient state"),
            Token::TransitionArrow,
            Token::Identifier("ast"),
            Token::Condition("ifyes"),
            Token::TransitionArrow,
            Token::Identifier("lastState"),
            Token::Condition("ifno"),
            Token::Dedent,
            Token::Dedent,
        ];

        // println!("tokens {:#?}", tokens);
        //
        assert_eq!(tokens.len(), 40);
        assert_eq!(expected_tokens, tokens);

        // another way to test the same thing. Good for debugging.
        let mut i = 0;

        while i < expected_tokens.len() {
            assert_eq!(expected_tokens[i], tokens[i]);
            i += 1;
        }
    }
}
