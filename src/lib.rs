use anyhow::{Context, Result, bail};
use std::borrow::Cow;
use std::fs;
use std::path::Path;

pub fn sort_json_string(original: &str) -> Result<String> {
    let mut parser = Parser::new(original);
    let document = parser
        .parse_document()
        .context("Parsing the JSON was not possible")?;
    Ok(document.render())
}

pub fn sort_json_file<P: AsRef<Path>>(path: P) -> Result<bool> {
    let path = path.as_ref();
    let original =
        fs::read_to_string(path).with_context(|| format!("Failed to read file {:?}", path))?;

    let sorted = sort_json_string(&original)
        .with_context(|| format!("Failed to sort JSON in {:?}", path))?;

    if original == sorted {
        return Ok(false);
    }

    fs::write(path, sorted).with_context(|| format!("Failed to write to file {:?}", path))?;

    Ok(true)
}

struct Document<'a> {
    source_len: usize,
    leading: &'a str,
    root: JsonNode<'a>,
    trailing: &'a str,
}

impl<'a> Document<'a> {
    fn render(&self) -> String {
        let mut rendered = String::with_capacity(self.source_len);
        rendered.push_str(self.leading);
        self.root.render(&mut rendered);
        rendered.push_str(self.trailing);
        rendered
    }
}

enum JsonNode<'a> {
    Object(ObjectNode<'a>),
    Array(ArrayNode<'a>),
    Primitive(&'a str),
}

impl<'a> JsonNode<'a> {
    fn render(&self, out: &mut String) {
        match self {
            JsonNode::Object(object) => object.render(out),
            JsonNode::Array(array) => array.render(out),
            JsonNode::Primitive(slice) => out.push_str(slice),
        }
    }
}

struct ObjectNode<'a> {
    body: ObjectBody<'a>,
}

enum ObjectBody<'a> {
    Empty(&'a str),
    Entries(Vec<ObjectEntry<'a>>),
}

impl<'a> ObjectNode<'a> {
    fn render(&self, out: &mut String) {
        out.push('{');

        match &self.body {
            ObjectBody::Empty(trivia) => out.push_str(trivia),
            ObjectBody::Entries(entries) => {
                for (index, entry) in entries.iter().enumerate() {
                    out.push_str(entry.leading);
                    out.push_str(entry.key.raw_span);
                    out.push_str(entry.between);
                    entry.value.render(out);
                    out.push_str(entry.after);

                    if index + 1 < entries.len() {
                        out.push(',');
                    }
                }
            }
        }

        out.push('}');
    }
}

struct ParsedKey<'a> {
    raw_span: &'a str,
    decoded: Cow<'a, str>,
}

impl<'a> ParsedKey<'a> {
    fn from_raw(raw: RawString<'a>) -> Result<Self> {
        let decoded = if let Some(decoded) = raw.decoded {
            Cow::Owned(decoded)
        } else {
            Cow::Borrowed(&raw.raw[1..raw.raw.len() - 1])
        };

        Ok(Self {
            raw_span: raw.raw,
            decoded,
        })
    }
}

struct ObjectEntry<'a> {
    leading: &'a str,
    key: ParsedKey<'a>,
    between: &'a str,
    value: JsonNode<'a>,
    after: &'a str,
}

struct ArrayNode<'a> {
    body: ArrayBody<'a>,
}

enum ArrayBody<'a> {
    Empty(&'a str),
    Items(Vec<ArrayItem<'a>>),
}

impl<'a> ArrayNode<'a> {
    fn render(&self, out: &mut String) {
        out.push('[');

        match &self.body {
            ArrayBody::Empty(trivia) => out.push_str(trivia),
            ArrayBody::Items(items) => {
                for (index, item) in items.iter().enumerate() {
                    out.push_str(item.leading);
                    item.value.render(out);
                    out.push_str(item.after);

                    if index + 1 < items.len() {
                        out.push(',');
                    }
                }
            }
        }

        out.push(']');
    }
}

struct ArrayItem<'a> {
    leading: &'a str,
    value: JsonNode<'a>,
    after: &'a str,
}

struct RawString<'a> {
    raw: &'a str,
    decoded: Option<String>,
}

struct Parser<'a> {
    source: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            pos: 0,
        }
    }

    fn parse_document(&mut self) -> Result<Document<'a>> {
        let leading = self.parse_trivia()?;
        let root = self.parse_value()?;
        let trailing = self.parse_trivia()?;

        if self.pos != self.bytes.len() {
            bail!("Unexpected content at byte {}", self.pos);
        }

        Ok(Document {
            source_len: self.source.len(),
            leading,
            root,
            trailing,
        })
    }

    fn parse_value(&mut self) -> Result<JsonNode<'a>> {
        match self.peek_byte() {
            Some(b'{') => self.parse_object().map(JsonNode::Object),
            Some(b'[') => self.parse_array().map(JsonNode::Array),
            Some(b'"') => self
                .lex_string()
                .map(|string| JsonNode::Primitive(string.raw)),
            Some(b'-' | b'0'..=b'9') => self.parse_number().map(JsonNode::Primitive),
            Some(b't') => self.parse_literal("true").map(JsonNode::Primitive),
            Some(b'f') => self.parse_literal("false").map(JsonNode::Primitive),
            Some(b'n') => self.parse_literal("null").map(JsonNode::Primitive),
            Some(byte) => bail!("Unexpected byte '{}' at byte {}", byte as char, self.pos),
            None => bail!("Unexpected end of input"),
        }
    }

    fn parse_object(&mut self) -> Result<ObjectNode<'a>> {
        self.expect_byte(b'{')?;

        let mut next_leading = self.parse_trivia()?;
        if self.peek_byte() == Some(b'}') {
            self.pos += 1;
            return Ok(ObjectNode {
                body: ObjectBody::Empty(next_leading),
            });
        }

        let mut entries = Vec::new();
        loop {
            let raw_key = self.lex_string()?;
            let parsed_key = ParsedKey::from_raw(raw_key)?;

            let between_start = self.pos;
            self.parse_trivia()?;
            self.expect_byte(b':')?;
            self.parse_trivia()?;
            let between = &self.source[between_start..self.pos];

            let value = self.parse_value()?;
            let after = self.parse_trivia()?;

            entries.push(ObjectEntry {
                leading: next_leading,
                key: parsed_key,
                between,
                value,
                after,
            });

            match self.peek_byte() {
                Some(b',') => {
                    self.pos += 1;
                    next_leading = self.parse_trivia()?;
                }
                Some(b'}') => {
                    self.pos += 1;
                    break;
                }
                _ => bail!("Expected ',' or '}}' at byte {}", self.pos),
            }
        }

        sort_entries_preserving_trivia_slots(&mut entries);

        Ok(ObjectNode {
            body: ObjectBody::Entries(entries),
        })
    }

    fn parse_array(&mut self) -> Result<ArrayNode<'a>> {
        self.expect_byte(b'[')?;

        let mut next_leading = self.parse_trivia()?;
        if self.peek_byte() == Some(b']') {
            self.pos += 1;
            return Ok(ArrayNode {
                body: ArrayBody::Empty(next_leading),
            });
        }

        let mut items = Vec::new();
        loop {
            let value = self.parse_value()?;
            let after = self.parse_trivia()?;

            items.push(ArrayItem {
                leading: next_leading,
                value,
                after,
            });

            match self.peek_byte() {
                Some(b',') => {
                    self.pos += 1;
                    next_leading = self.parse_trivia()?;
                }
                Some(b']') => {
                    self.pos += 1;
                    break;
                }
                _ => bail!("Expected ',' or ']' at byte {}", self.pos),
            }
        }

        Ok(ArrayNode {
            body: ArrayBody::Items(items),
        })
    }

    fn lex_string(&mut self) -> Result<RawString<'a>> {
        self.expect_byte(b'"')?;
        let start = self.pos - 1;
        let mut has_escapes = false;

        while let Some(byte) = self.peek_byte() {
            match byte {
                b'"' => {
                    self.pos += 1;
                    let slice = &self.source[start..self.pos];
                    let decoded =
                        if has_escapes {
                            // I delegate the full validation of escape sequence syntax to serde_json.
                            // Although a manual "zero-allocation" validator would be faster,
                            // I prefer to rely on this proven library to guarantee 100% compliance
                            // with the JSON standard without having to maintain my own validation code.
                            Some(serde_json::from_str::<String>(slice).with_context(|| {
                                format!("Invalid string literal at byte {}", start)
                            })?)
                        } else {
                            None
                        };
                    return Ok(RawString {
                        raw: slice,
                        decoded,
                    });
                }
                b'\\' => {
                    has_escapes = true;
                    self.pos += 1;
                    if self.peek_byte().is_none() {
                        bail!("Unterminated escape sequence at byte {}", self.pos);
                    }
                    self.pos += 1;
                }
                0x00..=0x1F => {
                    bail!("Invalid string literal at byte {}", start);
                }
                _ => {
                    self.pos += 1;
                }
            }
        }

        bail!("Unterminated string literal")
    }

    fn parse_number(&mut self) -> Result<&'a str> {
        let start = self.pos;

        if self.peek_byte() == Some(b'-') {
            self.pos += 1;
        }

        match self.peek_byte() {
            Some(b'0') => self.pos += 1,
            Some(b'1'..=b'9') => {
                self.pos += 1;
                while matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                    self.pos += 1;
                }
            }
            _ => bail!("Invalid number at byte {}", start),
        }

        if self.peek_byte() == Some(b'.') {
            self.pos += 1;
            if !matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                bail!("Invalid fractional part at byte {}", self.pos);
            }
            while matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }

        if matches!(self.peek_byte(), Some(b'e' | b'E')) {
            self.pos += 1;
            if matches!(self.peek_byte(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            if !matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                bail!("Invalid exponent at byte {}", self.pos);
            }
            while matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }

        Ok(&self.source[start..self.pos])
    }

    fn parse_literal(&mut self, literal: &str) -> Result<&'a str> {
        let start = self.pos;
        if self.bytes[start..].starts_with(literal.as_bytes()) {
            self.pos += literal.len();
            return Ok(&self.source[start..self.pos]);
        }

        bail!("Expected '{}' at byte {}", literal, start)
    }

    fn parse_trivia(&mut self) -> Result<&'a str> {
        let start = self.pos;

        loop {
            let checkpoint = self.pos;

            // Skip whitespace
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
                self.pos += 1;
            }

            if self.bytes[self.pos..].starts_with(b"//") {
                self.pos += 2;
                while self.pos < self.bytes.len() && self.bytes[self.pos] != b'\n' {
                    self.pos += 1;
                }
                continue;
            }

            if self.bytes[self.pos..].starts_with(b"/*") {
                self.pos += 2;
                while self.pos + 1 < self.bytes.len()
                    && !(self.bytes[self.pos] == b'*' && self.bytes[self.pos + 1] == b'/')
                {
                    self.pos += 1;
                }

                if self.pos + 1 >= self.bytes.len() {
                    bail!("Unterminated block comment")
                }

                self.pos += 2;
                continue;
            }

            if checkpoint == self.pos {
                break;
            }
        }

        Ok(&self.source[start..self.pos])
    }

    fn expect_byte(&mut self, expected: u8) -> Result<()> {
        match self.peek_byte() {
            Some(byte) if byte == expected => {
                self.pos += 1;
                Ok(())
            }
            Some(byte) => bail!(
                "Expected '{}' at byte {}, found '{}'",
                expected as char,
                self.pos,
                byte as char
            ),
            None => bail!("Expected '{}' at end of input", expected as char),
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }
}

fn sort_entries_preserving_trivia_slots(entries: &mut [ObjectEntry<'_>]) {
    // Sorting moves key/value pairs, but leading and trailing trivia belong to
    // the original object positions so formatting stays stable.
    let slots = entries
        .iter()
        .map(|entry| (entry.leading, entry.after))
        .collect::<Vec<_>>();

    entries.sort_by(|left, right| left.key.decoded.cmp(&right.key.decoded));

    for (entry, (leading, after)) in entries.iter_mut().zip(slots) {
        entry.leading = leading;
        entry.after = after;
    }
}
