mod tokenizer;
use tokenizer::*;

pub fn parse(input: &str) {
    let tokens = tokenize(input);

    println!("got tokens {:#?}", tokens);
}
