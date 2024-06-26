use std::{iter::Peekable, str::Chars, vec::IntoIter};

use crate::{parser::ast::Identifier, parser::parse_errors::ParseError};

use super::token::{HasInfix, ParsedMultipartToken, ParsedToken, Precedence, Token};

#[derive(Debug)]
pub struct LexedTokens {
    token_iter: Peekable<IntoIter<Token>>,
}

impl From<&str> for LexedTokens {
    fn from(source_code: &str) -> Self {
        let mut code_iter = source_code.chars().peekable();

        let mut tokens: Vec<Token> = Vec::new();
        while let Some(current_char) = code_iter.next() {
            if current_char.is_whitespace() {
                continue;
            }

            let lexed_token: Token = match Token::from(current_char) {
                ParsedToken::CompleteToken(token) => token,
                ParsedToken::PossibleMultipart(first_part) => {
                    match Token::lex_second_part(first_part, code_iter.peek().cloned()) {
                        ParsedMultipartToken::Multipart(token) => {
                            code_iter.next();
                            token
                        }
                        ParsedMultipartToken::OnlyOnePart(token) => token,
                    }
                }
                ParsedToken::AlphabeticStart => {
                    let literal: String = read_literal(&mut code_iter, current_char, |char| {
                        char.is_alphabetic() && char != &','
                    });

                    Token::parse_keyword(&literal)
                }
                ParsedToken::NumericStart => {
                    let literal: String =
                        read_literal(&mut code_iter, current_char, |char| char.is_numeric());

                    Token::Int(literal)
                }
            };

            tokens.push(lexed_token);
        }

        LexedTokens {
            token_iter: tokens.into_iter().peekable(),
        }
    }
}

impl LexedTokens {
    pub fn consume(&mut self) -> Option<Token> {
        self.token_iter.next()
    }

    pub fn expect(&mut self) -> Result<Token, ParseError> {
        match self.consume() {
            Some(token) => Ok(token),
            None => Err(ParseError::ExpectedToken),
        }
    }

    pub fn peek(&mut self) -> Option<&Token> {
        self.token_iter.peek()
    }

    pub fn next_token_has_infix(&mut self) -> bool {
        match self.token_iter.peek() {
            Some(token) => !matches!(token.has_infix(), HasInfix::No(_)),
            None => false,
        }
    }

    pub fn next_token_is(&mut self, is_token: &Token) -> bool {
        match self.token_iter.peek() {
            Some(token) => is_token == token,
            None => false,
        }
    }

    pub fn iterate_to_next_statement(&mut self) {
        for token in self.token_iter.by_ref() {
            if token == Token::Period {
                break;
            }
        }
    }

    pub fn expect_token(&mut self, expected_token_type: Token) -> Result<Token, ParseError> {
        match self.token_iter.next_if_eq(&expected_token_type) {
            Some(token) => Ok(token),
            None => Err(ParseError::single_unexpected(
                &expected_token_type,
                self.token_iter.peek(),
            )),
        }
    }

    pub fn expect_optional_token(&mut self, expected_token_type: Token) {
        self.token_iter.next_if_eq(&expected_token_type);
    }

    pub fn expected_identifier(&mut self) -> Result<Identifier, ParseError> {
        match self.token_iter.peek() {
            Some(peeked_token) => {
                let parsed_identifier = Identifier::parse_from_token(peeked_token)?;
                self.consume();
                Ok(parsed_identifier)
            }
            None => Err(ParseError::ExpectedToken),
        }
    }

    pub fn next_token_precedence(&mut self) -> Precedence {
        match self.peek() {
            Some(token) => token.get_precedence(),
            None => Precedence::Lowest,
        }
    }
}

fn read_literal<F>(iterator: &mut Peekable<Chars>, first_char: char, read_until: F) -> String
where
    F: Fn(&char) -> bool,
{
    let mut literal = String::from(first_char);

    while let Some(c) = iterator.peek().cloned().filter(|c| read_until(c)) {
        literal.push(c);
        iterator.next();
    }

    literal
}

#[cfg(test)]
mod tests {
    use crate::parser::lexer::{lexedtokens::LexedTokens, token::Token};

    #[test]
    fn parse_sympols() {
        let source_code = "
            !+:}{)(][~
        ";

        let expected_tokens = vec![
            Token::Bang,
            Token::Add,
            Token::Assign,
            Token::RBrace,
            Token::LBrace,
            Token::RParen,
            Token::LParen,
            Token::RBracket,
            Token::LBracket,
            Token::Lasagna,
        ];

        let mut found_tokens: LexedTokens = LexedTokens::from(source_code);
        let mut expected_iter = expected_tokens.into_iter();
        while let Some(token) = found_tokens.consume() {
            let expected_token = expected_iter.next().unwrap();
            assert_eq!(token, expected_token);
        }
    }

    #[test]
    fn parse_identifier() {
        let source_code = "
            foo
        ";

        let expected_tokens = [Token::Ident("foo".to_string())];

        let mut found_tokens: LexedTokens = LexedTokens::from(source_code);

        assert_eq!(
            found_tokens.token_iter.len(),
            expected_tokens.len(),
            "List of expected tokens should be the same as found tokens"
        );
        expected_tokens.iter().enumerate().for_each(|(idx, token)| {
            assert_eq!(
                token,
                &found_tokens.token_iter.nth(idx).expect("Should have token"),
                "Token in position {idx} was not parsed"
            )
        });
    }

    #[test]
    fn parse_comma_seperated_identifier() {
        let source_code = "
(foo, bar)
        ";

        let expected_tokens = [
            Token::LParen,
            Token::Ident("foo".to_string()),
            Token::Comma,
            Token::Ident("bar".to_string()),
            Token::RParen,
        ];

        let mut found_tokens: LexedTokens = LexedTokens::from(source_code);

        assert_eq!(
            found_tokens.token_iter.len(),
            expected_tokens.len(),
            "List of expected tokens should be the same as found tokens"
        );

        expected_tokens.iter().for_each(|token| {
            let found = &found_tokens.consume().unwrap();
            assert_eq!(token, found, "Expected {token:?}, but got {found:?}")
        });
    }

    #[test]
    fn parse_code() {
        let source_code = "
            let foo: 5~
            foo + 6
            ~fooFunc(x, y):
                ~res x+y
                return res
            ~

            ~if(5 < 10):
                5 + 6
                6 + 7
                return true
            ~else:
!
                return false
            ~
            ==
            !=
        ";

        let expected_tokens = vec![
            Token::Let,
            Token::Ident("foo".to_string()),
            Token::Assign,
            Token::Int("5".to_string()),
            Token::Lasagna,
            Token::Ident("foo".to_string()),
            Token::Add,
            Token::Int("6".to_string()),
            Token::Lasagna,
            Token::Ident("fooFunc".to_string()),
            Token::LParen,
            Token::Ident("x".to_string()),
            Token::Comma,
            Token::Ident("y".to_string()),
            Token::RParen,
            Token::Assign,
            Token::Lasagna,
            Token::Ident("res".to_string()),
            Token::Ident("x".to_string()),
            Token::Add,
            Token::Ident("y".to_string()),
            Token::Return,
            Token::Ident("res".to_string()),
            Token::Lasagna,
            Token::Lasagna,
            Token::If,
            Token::LParen,
            Token::Int("5".to_string()),
            Token::LessThan,
            Token::Int("10".to_string()),
            Token::RParen,
            Token::Assign,
            Token::Int("5".to_string()),
            Token::Add,
            Token::Int("6".to_string()),
            Token::Int("6".to_string()),
            Token::Add,
            Token::Int("7".to_string()),
            Token::Return,
            Token::True,
            Token::Lasagna,
            Token::Else,
            Token::Assign,
            Token::Bang,
            Token::Return,
            Token::False,
            Token::Lasagna,
            Token::Equal,
            Token::NotEqual,
        ];

        let mut found_tokens: LexedTokens = LexedTokens::from(source_code);
        let mut expected_iter = expected_tokens.into_iter();
        while let Some(token) = found_tokens.consume() {
            let expected_token = expected_iter.next().unwrap();
            assert_eq!(token, expected_token);
        }
    }
}
