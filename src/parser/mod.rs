mod tokenizer;
use tokenizer::*;

pub fn parse(input: &str) {
    let tokens = tokenize(input);

    println!("got tokens {:#?}", tokens);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
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

        parse(input);
        assert_eq!(1, 0);
    }
}
