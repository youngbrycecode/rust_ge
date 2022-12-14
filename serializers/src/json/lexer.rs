use std::{io::Error, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReserveCode {
    OpenBrace,
    CloseBrace, 
    OpenBracket, 
    CloseBracket, 
    Colon, 
    Comma,
    Undefined
}

impl Default for ReserveCode {
    fn default() -> Self {
        ReserveCode::Undefined
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Reserve {
        reserve_id: ReserveCode
    },
    Number { 
        value: i64
    },
    Float { 
        value: f64
    },
    Boolean {
        value: bool
    },
    String {
        value: String
    },
    Null,
    Undefined
}

impl Default for TokenType {
    fn default() -> Self {
        TokenType::Undefined
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Token {
    token_type: TokenType
}

impl Token {
    pub fn get_type(&self) -> TokenType {
        self.token_type.clone()
    }
}

pub struct JsonLexer {
    /// All the text in the json file.
    json_text: String,
    /// the current location we are lexing in the file.
    index: usize
}

/// A lexer for a json file which returns
/// the found tokens in order.
impl JsonLexer {
    /// Creates a new tokenizer given the name of the file.
    pub fn new(file_name: &str) -> Result<JsonLexer, Error> {
        Ok(JsonLexer {
            json_text: match std::fs::read_to_string(file_name) {
                Ok(file_text) => file_text,
                Err(e) => return Err(e)
            },
            index: 0
        })
    }

    /// Resets the lexer to the first token in the stream.
    pub fn reset(&mut self) {
        self.index = 0;
    }

    /// Creates a parser from raw string info.
    pub fn from_raw_json(raw_json: &str) -> Option<JsonLexer> {
        Some(JsonLexer { 
            json_text: match String::from_str(raw_json) {
                Ok(json_text) => json_text,
                Err(_) => return None
            },
            index: 0 
        })
    }

    pub fn next_token<'a>(&mut self, token: &'a mut Token) {
        let size = self.json_text.len();
        let json_text = self.json_text.as_bytes();

        skip_whitespace(json_text, &mut self.index, size);

        if is_eof(self.index, size) {
            token.token_type = TokenType::Undefined;
            return;
        }

        let ch = json_text[self.index] as char;

        if ch.is_numeric() || ch == '-' {
            // Load number
            if !load_number(json_text, &mut self.index, size, token) {
                token.token_type = TokenType::Undefined;
                return;
            }
        }
        else if is_str_start(ch as u8) {
            // Load string
            if !load_string(json_text, &mut self.index, size, token) {
                token.token_type = TokenType::Undefined;
                return;
            }
        }
        else if load_boolean_or_null(json_text, &mut self.index, size, token) {
            // load_boolean_or_null returns true if it successfully loaded a bool or null.
        }
        else {
            // If it's a reserve, add it, otherwise there is an error :D
            if !load_reserve(json_text, &mut self.index, token) {
                token.token_type = TokenType::Undefined;
            }
        }
    }
}

fn is_eof(index: usize, size: usize) -> bool {
    index >= size
}

fn skip_whitespace(json: &[u8], index: &mut usize, size: usize) {
    while !is_eof(*index, size) && (json[*index]).is_ascii_whitespace() {
        *index += 1;
    }
}

fn is_str_start(c: u8) -> bool {
    c == '"' as u8
}

fn get_integer_num(json: &[u8], index: &mut usize, size: usize) {
    let mut ch = json[*index];

    while (ch as char).is_numeric() && !is_eof(*index, size) {
        *index += 1;

        if is_eof(*index, size) {
            return;
        }

        ch = json[*index];
    }
}

fn load_number<'a>(json: &'a [u8], index: &mut usize, size: usize, new_token: &'a mut Token) -> bool {
    let mut ch;
    let mut flt = false;

    let token_start = *index;
    *index += 1;

    get_integer_num(json, index, size);

    if !is_eof(*index, size) {
        ch = json[*index];

        // If it has floating point notation, continue with parsing as a float.
        if ch == '.' as u8 {
            *index += 1;
            flt = true;

            get_integer_num(json, index, size);

            if !is_eof(*index, size) {
                ch = json[*index];

                if ch == 'e' as u8 || ch == 'E' as u8 && !is_eof(*index + 1, size) {
                    *index += 1;
                    ch = json[*index];

                    if ch == '-' as u8 || ch == '+' as u8 && !is_eof(*index + 1, size) {
                        *index += 1;
                        ch = json[*index];
                    }
                    
                    if !(ch as char).is_numeric() {
                        println!("ERROR: Exponential notation must have at least one digit");
                        return false;
                    }
                    else {
                        get_integer_num(json, index, size);
                    }
                }
            }
        }
    }

    if !flt {
        let as_str = String::from_utf8_lossy(&json[token_start..*index]);
        let value = i64::from_str_radix(&as_str, 10);

        if let Ok(value) = value {
            new_token.token_type = TokenType::Number { value };
            return true;
        }
        else {
            println!("Error parsing integer: {}", as_str);
            return false;
        }
    }
    else { 
        let as_str = String::from_utf8_lossy(&json[token_start..*index]);
        let value = as_str.parse::<f64>();

        if let Ok(value) = value {
            new_token.token_type = TokenType::Float { value };
            return true;
        }
        else {
            println!("Error parsing float: {}", as_str);
            return false;
        }
    }
}

fn load_string<'a>(json: &'a [u8], index: &mut usize, size: usize, new_token: &'a mut Token) -> bool {
    let mut ch;
    let ending_quote = json[*index];

    *index += 1;

    let token_start = *index;
    ch = json[*index];

    'str_contents: while ch != ending_quote && !is_eof(*index, size) {
        if ch == '\\' as u8 {
            *index += 1;
        }
        if ch != '\n' as u8 {
            *index += 1;
            ch = json[*index];
        }
        else {
            break 'str_contents;
        }
    }

    if ch != '\n' as u8 {
        *index += 1;

        new_token.token_type = TokenType::String { 
            value: match String::from_utf8_lossy(&json[token_start..*index - 1]) {
                std::borrow::Cow::Borrowed(lex) => String::from_str(lex).unwrap(),
                std::borrow::Cow::Owned(lex) => lex
            }
        };
        true
    }
    else {
        println!("Error: Strings cannot appear on multiple lines");
        false
    }
}

fn load_reserve<'a>(json: &'a [u8], index: &mut usize, 
    new_token: &'a mut Token) -> bool 
{
    let ch = json[*index] as char;
    *index += 1;
    
    let mut err = false;
    match ch {
        '{' => new_token.token_type = TokenType::Reserve { reserve_id: ReserveCode::OpenBrace },
        '}' => new_token.token_type = TokenType::Reserve { reserve_id: ReserveCode::CloseBrace },
        '[' => new_token.token_type = TokenType::Reserve { reserve_id: ReserveCode::OpenBracket },
        ']' => new_token.token_type = TokenType::Reserve { reserve_id: ReserveCode::CloseBracket },
        ':' => new_token.token_type = TokenType::Reserve { reserve_id: ReserveCode::Colon },
        ',' => new_token.token_type = TokenType::Reserve { reserve_id: ReserveCode::Comma },
        _ => {
            err = true;
        }
    }

    return !err;
}

fn load_boolean_or_null<'a>(json: &'a [u8], index: &mut usize, size: usize, new_token: &'a mut Token) -> bool {
    let true_token = "true".as_bytes();
    let false_token = "false".as_bytes();
    let null_token = "null".as_bytes();

    if !is_eof(*index + true_token.len(), size) {
        if json[*index..(*index + true_token.len())].eq(true_token) {
            *index += true_token.len();
            new_token.token_type = TokenType::Boolean { value: true };
            return true
        }
    }

    if !is_eof(*index + false_token.len(), size) {
        if json[*index..(*index + false_token.len())].eq(false_token) {
            *index += false_token.len();
            new_token.token_type = TokenType::Boolean { value: false };
            return true
        }
    }

    if !is_eof(*index + null_token.len(), size) {
        if json[*index..(*index + null_token.len())].eq(null_token) {
            *index += null_token.len();
            new_token.token_type = TokenType::Null;
            return true
        }
    }

    false
}