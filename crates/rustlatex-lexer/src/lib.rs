//! `rustlatex-lexer` — LaTeX tokenizer
//!
//! This crate implements the first stage of the LaTeX compilation pipeline:
//! tokenizing raw LaTeX source into a stream of tokens. It handles TeX's
//! category code mechanism, control sequences, grouping, math mode, and more.

/// A token produced by the LaTeX lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// A control sequence such as `\section` or `\textbf`.
    ControlSequence(String),
    /// A single character with its category.
    Character(char, Category),
    /// End of input.
    EndOfInput,
}

/// TeX category codes assigned to characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    /// Category 0: escape character (usually `\`)
    Escape,
    /// Category 1: begin group (usually `{`)
    BeginGroup,
    /// Category 2: end group (usually `}`)
    EndGroup,
    /// Category 3: math shift (usually `$`)
    MathShift,
    /// Category 4: alignment tab (usually `&`)
    AlignmentTab,
    /// Category 5: end of line (usually `\n`)
    EndOfLine,
    /// Category 6: parameter (usually `#`)
    Parameter,
    /// Category 7: superscript (usually `^`)
    Superscript,
    /// Category 8: subscript (usually `_`)
    Subscript,
    /// Category 9: ignored character
    Ignored,
    /// Category 10: space (usually space and tab)
    Space,
    /// Category 11: letter (a-z, A-Z)
    Letter,
    /// Category 12: other (anything else)
    Other,
    /// Category 13: active character (usually `~`)
    Active,
    /// Category 14: comment (usually `%`)
    Comment,
    /// Category 15: invalid character
    Invalid,
}

/// The lexer tokenizes a LaTeX source string into a sequence of [`Token`]s.
pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    /// Create a new lexer for the given input string.
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    /// Return the default TeX category for a character.
    pub fn category(ch: char) -> Category {
        match ch {
            '\\' => Category::Escape,
            '{' => Category::BeginGroup,
            '}' => Category::EndGroup,
            '$' => Category::MathShift,
            '&' => Category::AlignmentTab,
            '\n' => Category::EndOfLine,
            '#' => Category::Parameter,
            '^' => Category::Superscript,
            '_' => Category::Subscript,
            '\x00' => Category::Ignored,
            ' ' | '\t' => Category::Space,
            'a'..='z' | 'A'..='Z' => Category::Letter,
            '~' => Category::Active,
            '%' => Category::Comment,
            '\x7f' => Category::Invalid,
            _ => Category::Other,
        }
    }

    /// Peek at the current character without advancing.
    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    /// Advance and return the current character.
    fn advance(&mut self) -> Option<char> {
        let ch = self.input.get(self.pos).copied();
        self.pos += 1;
        ch
    }

    /// Read the next token from the input.
    pub fn next_token(&mut self) -> Token {
        // Skip comments
        loop {
            match self.peek() {
                None => return Token::EndOfInput,
                Some('%') => {
                    // Skip until end of line
                    while let Some(ch) = self.advance() {
                        if ch == '\n' {
                            break;
                        }
                    }
                }
                _ => break,
            }
        }

        match self.advance() {
            None => Token::EndOfInput,
            Some('\\') => {
                // Control sequence
                match self.peek() {
                    None => Token::ControlSequence(String::new()),
                    Some(ch) if Self::category(ch) == Category::Letter => {
                        let mut name = String::new();
                        while let Some(c) = self.peek() {
                            if Self::category(c) == Category::Letter {
                                name.push(c);
                                self.advance();
                            } else {
                                break;
                            }
                        }
                        // Skip trailing spaces after a word control sequence
                        while self.peek() == Some(' ') || self.peek() == Some('\t') {
                            self.advance();
                        }
                        Token::ControlSequence(name)
                    }
                    Some(ch) => {
                        self.advance();
                        Token::ControlSequence(ch.to_string())
                    }
                }
            }
            Some(ch) => Token::Character(ch, Self::category(ch)),
        }
    }

    /// Collect all tokens into a vector.
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            if tok == Token::EndOfInput {
                tokens.push(tok);
                break;
            }
            tokens.push(tok);
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_sequence() {
        let mut lexer = Lexer::new(r"\hello world");
        assert_eq!(
            lexer.next_token(),
            Token::ControlSequence("hello".to_string())
        );
        assert_eq!(lexer.next_token(), Token::Character('w', Category::Letter));
    }

    #[test]
    fn test_character_categories() {
        assert_eq!(Lexer::category('{'), Category::BeginGroup);
        assert_eq!(Lexer::category('}'), Category::EndGroup);
        assert_eq!(Lexer::category('$'), Category::MathShift);
        assert_eq!(Lexer::category('a'), Category::Letter);
        assert_eq!(Lexer::category(' '), Category::Space);
    }

    #[test]
    fn test_comment_skipping() {
        let mut lexer = Lexer::new("a% this is a comment\nb");
        assert_eq!(lexer.next_token(), Token::Character('a', Category::Letter));
        assert_eq!(lexer.next_token(), Token::Character('b', Category::Letter));
    }

    #[test]
    fn test_tokenize_simple_document() {
        let src = r"\documentclass{article}";
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize();
        assert_eq!(
            tokens[0],
            Token::ControlSequence("documentclass".to_string())
        );
        assert_eq!(tokens[1], Token::Character('{', Category::BeginGroup));
    }

    #[test]
    fn test_end_of_input() {
        let mut lexer = Lexer::new("");
        assert_eq!(lexer.next_token(), Token::EndOfInput);
    }
}
