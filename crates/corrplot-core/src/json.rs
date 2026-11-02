//! Small dependency-free JSON reader used at the shared-core data boundary.

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParseError {
    pub offset: usize,
    pub message: &'static str,
}

pub(crate) fn parse(source: &str) -> Result<Value, ParseError> {
    let mut parser = Parser { bytes: source.as_bytes(), offset: 0 };
    parser.whitespace();
    let value = parser.value()?;
    parser.whitespace();
    if parser.offset != parser.bytes.len() {
        return Err(parser.error("unexpected trailing input"));
    }
    Ok(value)
}

struct Parser<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl Parser<'_> {
    fn error(&self, message: &'static str) -> ParseError {
        ParseError { offset: self.offset, message }
    }

    fn whitespace(&mut self) {
        while matches!(self.bytes.get(self.offset), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.offset += 1;
        }
    }

    fn value(&mut self) -> Result<Value, ParseError> {
        self.whitespace();
        match self.bytes.get(self.offset).copied() {
            Some(b'n') => { self.keyword(b"null")?; Ok(Value::Null) }
            Some(b't') => { self.keyword(b"true")?; Ok(Value::Bool(true)) }
            Some(b'f') => { self.keyword(b"false")?; Ok(Value::Bool(false)) }
            Some(b'"') => self.string().map(Value::String),
            Some(b'[') => self.array(),
            Some(b'{') => self.object(),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err(self.error("expected a JSON value")),
        }
    }

    fn keyword(&mut self, keyword: &[u8]) -> Result<(), ParseError> {
        if self.bytes.get(self.offset..self.offset + keyword.len()) == Some(keyword) {
            self.offset += keyword.len();
            Ok(())
        } else {
            Err(self.error("invalid JSON keyword"))
        }
    }

    fn string(&mut self) -> Result<String, ParseError> {
        self.offset += 1;
        let mut result = String::new();
        loop {
            let byte = self.bytes.get(self.offset).copied().ok_or_else(|| self.error("unterminated JSON string"))?;
            match byte {
                b'"' => { self.offset += 1; return Ok(result); }
                b'\\' => {
                    self.offset += 1;
                    let escaped = self.bytes.get(self.offset).copied().ok_or_else(|| self.error("incomplete JSON escape"))?;
                    self.offset += 1;
                    match escaped {
                        b'"' => result.push('"'),
                        b'\\' => result.push('\\'),
                        b'/' => result.push('/'),
                        b'b' => result.push('\u{0008}'),
                        b'f' => result.push('\u{000c}'),
                        b'n' => result.push('\n'),
                        b'r' => result.push('\r'),
                        b't' => result.push('\t'),
                        b'u' => {
                            let first = self.hex_quad()?;
                            let scalar = if (0xD800..=0xDBFF).contains(&first) {
                                if self.bytes.get(self.offset..self.offset + 2) != Some(b"\\u") {
                                    return Err(self.error("high surrogate must be followed by a low surrogate"));
                                }
                                self.offset += 2;
                                let second = self.hex_quad()?;
                                if !(0xDC00..=0xDFFF).contains(&second) {
                                    return Err(self.error("invalid low surrogate"));
                                }
                                0x10000 + (((first - 0xD800) as u32) << 10) + (second - 0xDC00) as u32
                            } else if (0xDC00..=0xDFFF).contains(&first) {
                                return Err(self.error("unexpected low surrogate"));
                            } else { first as u32 };
                            result.push(char::from_u32(scalar).ok_or_else(|| self.error("invalid Unicode scalar"))?);
                        }
                        _ => return Err(self.error("invalid JSON escape")),
                    }
                }
                0x00..=0x1f => return Err(self.error("unescaped control character in JSON string")),
                0x20..=0x7f => { result.push(byte as char); self.offset += 1; }
                _ => {
                    let remaining = std::str::from_utf8(&self.bytes[self.offset..])
                        .map_err(|_| self.error("invalid UTF-8 in JSON string"))?;
                    let character = remaining.chars().next().ok_or_else(|| self.error("invalid UTF-8 in JSON string"))?;
                    result.push(character);
                    self.offset += character.len_utf8();
                }
            }
        }
    }

    fn hex_quad(&mut self) -> Result<u16, ParseError> {
        let mut value = 0_u16;
        for _ in 0..4 {
            let digit = self.bytes.get(self.offset).copied().ok_or_else(|| self.error("incomplete Unicode escape"))?;
            self.offset += 1;
            value = value * 16 + match digit {
                b'0'..=b'9' => (digit - b'0') as u16,
                b'a'..=b'f' => (digit - b'a' + 10) as u16,
                b'A'..=b'F' => (digit - b'A' + 10) as u16,
                _ => return Err(self.error("invalid hexadecimal digit in Unicode escape")),
            };
        }
        Ok(value)
    }

    fn array(&mut self) -> Result<Value, ParseError> {
        self.offset += 1;
        self.whitespace();
        let mut values = Vec::new();
        if self.bytes.get(self.offset) == Some(&b']') { self.offset += 1; return Ok(Value::Array(values)); }
        loop {
            values.push(self.value()?);
            self.whitespace();
            match self.bytes.get(self.offset) {
                Some(b',') => { self.offset += 1; self.whitespace(); }
                Some(b']') => { self.offset += 1; return Ok(Value::Array(values)); }
                _ => return Err(self.error("expected ',' or ']' in JSON array")),
            }
        }
    }

    fn object(&mut self) -> Result<Value, ParseError> {
        self.offset += 1;
        self.whitespace();
        let mut values = BTreeMap::new();
        if self.bytes.get(self.offset) == Some(&b'}') { self.offset += 1; return Ok(Value::Object(values)); }
        loop {
            if self.bytes.get(self.offset) != Some(&b'"') { return Err(self.error("expected a string key in JSON object")); }
            let key = self.string()?;
            self.whitespace();
            if self.bytes.get(self.offset) != Some(&b':') { return Err(self.error("expected ':' after JSON object key")); }
            self.offset += 1;
            let value = self.value()?;
            if values.insert(key, value).is_some() { return Err(self.error("duplicate JSON object key")); }
            self.whitespace();
            match self.bytes.get(self.offset) {
                Some(b',') => { self.offset += 1; self.whitespace(); }
                Some(b'}') => { self.offset += 1; return Ok(Value::Object(values)); }
                _ => return Err(self.error("expected ',' or '}' in JSON object")),
            }
        }
    }

    fn number(&mut self) -> Result<Value, ParseError> {
        let start = self.offset;
        if self.bytes.get(self.offset) == Some(&b'-') { self.offset += 1; }
        match self.bytes.get(self.offset) {
            Some(b'0') => self.offset += 1,
            Some(b'1'..=b'9') => {
                self.offset += 1;
                while matches!(self.bytes.get(self.offset), Some(b'0'..=b'9')) { self.offset += 1; }
            }
            _ => return Err(self.error("invalid JSON number")),
        }
        if self.bytes.get(self.offset) == Some(&b'.') {
            self.offset += 1;
            let digits = self.offset;
            while matches!(self.bytes.get(self.offset), Some(b'0'..=b'9')) { self.offset += 1; }
            if self.offset == digits { return Err(self.error("fraction requires at least one digit")); }
        }
        if matches!(self.bytes.get(self.offset), Some(b'e' | b'E')) {
            self.offset += 1;
            if matches!(self.bytes.get(self.offset), Some(b'+' | b'-')) { self.offset += 1; }
            let digits = self.offset;
            while matches!(self.bytes.get(self.offset), Some(b'0'..=b'9')) { self.offset += 1; }
            if self.offset == digits { return Err(self.error("exponent requires at least one digit")); }
        }
        let raw = std::str::from_utf8(&self.bytes[start..self.offset]).map_err(|_| self.error("invalid JSON number"))?;
        let value = raw.parse::<f64>().map_err(|_| self.error("invalid JSON number"))?;
        if !value.is_finite() { return Err(self.error("JSON number is outside the finite range")); }
        Ok(Value::Number(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_json_and_unicode_surrogate_pairs() {
        let value = parse(r#"{"label":"A\uD83D\uDE80","cells":[null,true,-1.25e2]}"#).unwrap();
        let Value::Object(object) = value else { panic!("expected object") };
        assert_eq!(object.get("label"), Some(&Value::String("A🚀".into())));
        assert_eq!(object.get("cells"), Some(&Value::Array(vec![Value::Null, Value::Bool(true), Value::Number(-125.0)])));
    }

    #[test]
    fn rejects_duplicate_keys_invalid_numbers_and_trailing_input() {
        assert!(parse(r#"{"x":1,"x":2}"#).is_err());
        assert!(parse("01").is_err());
        assert!(parse("1 trailing").is_err());
    }
}
