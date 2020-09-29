use bigdecimal::BigDecimal;
use std::iter::Peekable;
use thiserror::Error;

pub fn eval(input: &str) -> Result<BigDecimal, EvalError> {
    let tokens = lex(input)?;
    let mut token_iter = tokens.into_iter().peekable();
    let parse_tree = parse_expr(&mut token_iter)?;
    Ok(eval_tree(&parse_tree))
}

#[derive(Error, Debug)]
pub enum EvalError {
    #[error("Unable to lex the provided expression")]
    LexError(#[from] LexError),
    #[error("Unable to parse the provided expression")]
    ParseError(#[from] ParseError),
    #[error("Can not evaluate empty input")]
    EmptyInput,
}

#[derive(Error, Debug)]
pub enum LexError {
    #[error("The token \"{0}\" is not allowed")]
    IllegalToken(String),
    #[error("\"{0}\" is not a number")]
    IllegalNumber(String),
    #[error("The input must be ASCII")]
    NonAsciiInput,
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Input contains unmatched parenthesis")]
    UnmatchedParens,
    #[error("Input contains unmatched token")]
    UnmatchedToken,
    #[error("Expected binary operator")]
    ExpectedBinaryOperator,
    #[error("Can not parse empty input")]
    EmptyInput,
}

#[derive(Debug)]
enum Token {
    Number(BigDecimal),
    ParenStart,
    ParenClose,
    Plus,
    Minus,
    Mul,
    Div,
}

impl Token {
    fn op_precedence(&self) -> Option<usize> {
        match self {
            Token::Plus | Token::Minus => Some(0),
            Token::Mul | Token::Div => Some(1),
            _ => None,
        }
    }
}

#[derive(Debug)]
enum ParseTree {
    Number(BigDecimal),
    Neg(Box<ParseTree>),
    Plus(Box<ParseTree>, Box<ParseTree>),
    Sub(Box<ParseTree>, Box<ParseTree>),
    Mul(Box<ParseTree>, Box<ParseTree>),
    Div(Box<ParseTree>, Box<ParseTree>),
}

impl ParseTree {
    fn apply(self: Box<Self>, op: Token, other: Box<Self>) -> Result<Box<ParseTree>, ParseError> {
        let applied = match op {
            Token::Plus => Self::Plus(self, other),
            Token::Minus => Self::Sub(self, other),
            Token::Mul => Self::Mul(self, other),
            Token::Div => Self::Div(self, other),
            _ => return Err(ParseError::ExpectedBinaryOperator),
        };
        Ok(Box::new(applied))
    }
}

fn lex(input: &str) -> Result<Vec<Token>, LexError> {
    if !input.is_ascii() {
        return Err(LexError::NonAsciiInput);
    }
    let mut result = vec![];

    let mut byte_iter = input.bytes().enumerate().peekable();

    while let Some((idx, byte)) = byte_iter.next() {
        if byte.is_ascii_whitespace() {
            continue;
        }
        let token = match byte {
            b'(' => Token::ParenStart,
            b')' => Token::ParenClose,
            b'+' => Token::Plus,
            b'-' => Token::Minus,
            b'*' => Token::Mul,
            b'/' => Token::Div,
            b'0'..=b'9' | b'.' => Token::Number(parse_number(idx, &mut byte_iter, input)?),
            unknown => {
                return Err(LexError::IllegalToken(
                    String::from_utf8_lossy(&[unknown]).into_owned(),
                ))
            }
        };
        result.push(token);
    }

    Ok(result)
}

fn parse_number(
    start_idx: usize,
    byte_iter: &mut Peekable<impl Iterator<Item = (usize, u8)>>,
    input: &str,
) -> Result<BigDecimal, LexError> {
    let mut end_idx = start_idx + 1;
    while let Some((_, byte)) = byte_iter.peek() {
        if matches!(byte, b'0'..=b'9' | b'.') {
            byte_iter.next();
            end_idx += 1;
        } else {
            break;
        }
    }
    let number = input[start_idx..end_idx]
        .parse()
        .map_err(|_| LexError::IllegalNumber(input[start_idx..end_idx].to_string()))?;
    Ok(number)
}

fn parse_expr(
    input: &mut Peekable<impl DoubleEndedIterator<Item = Token>>,
) -> Result<Box<ParseTree>, ParseError> {
    parse_expr_rec(parse_primary(input)?, input, 0)
}

fn parse_expr_rec(
    mut lhs: Box<ParseTree>,
    input: &mut Peekable<impl DoubleEndedIterator<Item = Token>>,
    min_precedence: usize,
) -> Result<Box<ParseTree>, ParseError> {
    let mut lookahead = input.peek();
    while lookahead.map(Token::op_precedence).flatten() >= Some(min_precedence) {
        let op = input.next().unwrap();
        let mut rhs = parse_primary(input)?;
        lookahead = input.peek();
        while lookahead.map(Token::op_precedence).flatten() >= op.op_precedence() {
            let lookahead_prec = lookahead.unwrap().op_precedence().unwrap();
            rhs = parse_expr_rec(rhs, input, lookahead_prec)?;
            lookahead = input.peek();
        }
        lhs = lhs.apply(op, rhs)?;
    }
    Ok(lhs)
}

fn parse_primary(
    input: &mut Peekable<impl DoubleEndedIterator<Item = Token>>,
) -> Result<Box<ParseTree>, ParseError> {
    match input.next() {
        Some(Token::ParenStart) => {
            if let Some(Token::ParenClose) = input.next_back() {
                parse_expr(input)
            } else {
                Err(ParseError::UnmatchedParens)
            }
        }
        Some(Token::Number(num)) => Ok(Box::new(ParseTree::Number(num))),
        Some(Token::Minus) => Ok(Box::new(ParseTree::Neg(parse_primary(input)?))),
        Some(_) => Err(ParseError::UnmatchedToken),
        None => Err(ParseError::EmptyInput),
    }
}

fn eval_tree(parse_tree: &ParseTree) -> BigDecimal {
    match parse_tree {
        ParseTree::Number(num) => num.clone(),
        ParseTree::Neg(tree) => -eval_tree(tree),
        ParseTree::Plus(left, right) => eval_tree(left) + eval_tree(right),
        ParseTree::Sub(left, right) => eval_tree(left) - eval_tree(right),
        ParseTree::Mul(left, right) => eval_tree(left) * eval_tree(right),
        ParseTree::Div(left, right) => eval_tree(left) / eval_tree(right),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn test_parse_number() {
        let input = "42";
        let mut byte_iter = input.bytes().enumerate().peekable();
        byte_iter.next();
        let expected: BigDecimal = 42.try_into().unwrap();
        assert_eq!(expected, parse_number(0, &mut byte_iter, input).unwrap());
    }

    #[test]
    fn test_parse_float_number() {
        let input = "0.3";
        let mut byte_iter = input.bytes().enumerate().peekable();
        byte_iter.next();
        let expected: BigDecimal = 0.3.try_into().unwrap();
        assert_eq!(expected, parse_number(0, &mut byte_iter, input).unwrap());
    }

    #[test]
    fn test_parse_number_trailing_input() {
        let input = "0.1256+510";
        let mut byte_iter = input.bytes().enumerate().peekable();
        byte_iter.next();
        let expected: BigDecimal = 0.1256.try_into().unwrap();
        assert_eq!(expected, parse_number(0, &mut byte_iter, input).unwrap());
    }

    #[test]
    fn test_parse_number_no_decimal_point() {
        let input = "1256";
        let mut byte_iter = input.bytes().enumerate().peekable();
        byte_iter.next();
        let expected: BigDecimal = 1256.try_into().unwrap();
        assert_eq!(expected, parse_number(0, &mut byte_iter, input).unwrap());
    }

    #[test]
    fn test_parse_number_leading_decimal_point() {
        let input = ".1256";
        let mut byte_iter = input.bytes().enumerate().peekable();
        byte_iter.next();
        let expected: BigDecimal = 0.1256.try_into().unwrap();

        assert_eq!(expected, parse_number(0, &mut byte_iter, input).unwrap());
    }

    #[test]
    fn parse_simple_expr() {
        let input = lex("42").unwrap();
        let mut iter = input.into_iter().peekable();
        parse_expr(&mut iter).unwrap();
    }

    #[test]
    fn parse_addition() {
        let input = lex("42 + 50").unwrap();
        let mut iter = input.into_iter().peekable();
        parse_expr(&mut iter).unwrap();
    }

    #[test]
    fn compute_sub() -> Result<(), EvalError> {
        let expected: BigDecimal = 0.try_into().unwrap();
        assert_eq!(expected, eval("42 - 42")?);
        Ok(())
    }

    #[test]
    fn compute_mul() -> Result<(), EvalError> {
        let expected: BigDecimal = 84.0.try_into().unwrap();
        assert_eq!(expected, eval("2 * 42")?);
        Ok(())
    }

    #[test]
    fn compute_div() -> Result<(), EvalError> {
        let expected: BigDecimal = 21.0.try_into().unwrap();
        assert_eq!(expected, eval("42 / 2")?);
        Ok(())
    }

    #[test]
    fn compute_with_precedence() -> Result<(), EvalError> {
        let expected: BigDecimal = 25.0.try_into().unwrap();
        assert_eq!(expected, eval("5 + 10 * 2")?);
        Ok(())
    }

    #[test]
    fn compute_with_braces() -> Result<(), EvalError> {
        let expected: BigDecimal = 42.0.try_into().unwrap();
        assert_eq!(expected, eval("2 * (10 + 11)")?);
        Ok(())
    }

    #[test]
    fn compute_negation() -> Result<(), EvalError> {
        let expected: BigDecimal = (-42.0).try_into().unwrap();
        assert_eq!(expected, eval("-2 * (10 + 11)")?);
        Ok(())
    }

    #[test]
    fn compute_check_precision() -> Result<(), EvalError> {
        let expected: BigDecimal = 0.3.try_into().unwrap();
        assert_eq!(expected, eval("0.1 + 0.2")?);
        Ok(())
    }
}
