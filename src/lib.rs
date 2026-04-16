use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::fs;
use std::ops::Range;
use std::path::Path;

pub fn sort_json_string(original: &str) -> Result<String> {
    validate_json(original)?;

    let mut parser = Parser::new(original);
    let document = parser.parse_document()?;
    document.render()
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

fn validate_json(original: &str) -> Result<()> {
    let stripped = json_comments::StripComments::new(original.as_bytes());
    let _: Value = serde_json::from_reader(stripped).context("Failed to parse JSON")?;
    Ok(())
}

struct Document<'a> {
    source: &'a str,
    leading: Range<usize>,
    root: JsonNode,
    trailing: Range<usize>,
}

impl<'a> Document<'a> {
    fn render(&self) -> Result<String> {
        let mut rendered = String::with_capacity(self.source.len());
        rendered.push_str(self.slice(&self.leading));
        self.root.render(self.source, &mut rendered);
        rendered.push_str(self.slice(&self.trailing));
        Ok(rendered)
    }

    fn slice(&self, range: &Range<usize>) -> &str {
        &self.source[range.clone()]
    }
}

enum JsonNode {
    Object(ObjectNode),
    Array(ArrayNode),
    Primitive(Range<usize>),
}

impl JsonNode {
    fn render(&self, source: &str, out: &mut String) {
        match self {
            JsonNode::Object(object) => object.render(source, out),
            JsonNode::Array(array) => array.render(source, out),
            JsonNode::Primitive(range) => out.push_str(&source[range.clone()]),
        }
    }
}

struct ObjectNode {
    leading_slots: Vec<Range<usize>>,
    entries: Vec<ObjectEntry>,
    trailing_slots: Vec<Range<usize>>,
    empty_trivia: Option<Range<usize>>,
}

impl ObjectNode {
    fn render(&self, source: &str, out: &mut String) {
        out.push('{');

        if let Some(trivia) = &self.empty_trivia {
            out.push_str(&source[trivia.clone()]);
            out.push('}');
            return;
        }

        let mut entries: Vec<&ObjectEntry> = self.entries.iter().collect();
        entries.sort_by(|left, right| left.key.cmp(&right.key));

        for (index, entry) in entries.iter().enumerate() {
            out.push_str(&source[self.leading_slots[index].clone()]);
            out.push_str(&source[entry.key_span.clone()]);
            out.push_str(&source[entry.between.clone()]);
            entry.value.render(source, out);
            out.push_str(&source[self.trailing_slots[index].clone()]);

            if index + 1 < entries.len() {
                out.push(',');
            }
        }

        out.push('}');
    }
}

struct ObjectEntry {
    key_span: Range<usize>,
    key: String,
    between: Range<usize>,
    value: JsonNode,
}

struct ArrayNode {
    items: Vec<ArrayItem>,
    empty_trivia: Option<Range<usize>>,
}

impl ArrayNode {
    fn render(&self, source: &str, out: &mut String) {
        out.push('[');

        if let Some(trivia) = &self.empty_trivia {
            out.push_str(&source[trivia.clone()]);
            out.push(']');
            return;
        }

        for (index, item) in self.items.iter().enumerate() {
            out.push_str(&source[item.leading.clone()]);
            item.value.render(source, out);
            out.push_str(&source[item.after.clone()]);

            if index + 1 < self.items.len() {
                out.push(',');
            }
        }

        out.push(']');
    }
}

struct ArrayItem {
    leading: Range<usize>,
    value: JsonNode,
    after: Range<usize>,
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
            source: self.source,
            leading,
            root,
            trailing,
        })
    }

    fn parse_value(&mut self) -> Result<JsonNode> {
        match self.peek_byte() {
            Some(b'{') => self.parse_object().map(JsonNode::Object),
            Some(b'[') => self.parse_array().map(JsonNode::Array),
            Some(b'"') => self.parse_string().map(JsonNode::Primitive),
            Some(b'-' | b'0'..=b'9') => self.parse_number().map(JsonNode::Primitive),
            Some(b't') => self.parse_literal("true").map(JsonNode::Primitive),
            Some(b'f') => self.parse_literal("false").map(JsonNode::Primitive),
            Some(b'n') => self.parse_literal("null").map(JsonNode::Primitive),
            Some(byte) => bail!("Unexpected byte '{}' at byte {}", byte as char, self.pos),
            None => bail!("Unexpected end of input"),
        }
    }

    fn parse_object(&mut self) -> Result<ObjectNode> {
        self.expect_byte(b'{')?;

        let mut next_leading = self.parse_trivia()?;
        if self.peek_byte() == Some(b'}') {
            self.pos += 1;
            return Ok(ObjectNode {
                leading_slots: Vec::new(),
                entries: Vec::new(),
                trailing_slots: Vec::new(),
                empty_trivia: Some(next_leading),
            });
        }

        let mut leading_slots = Vec::new();
        let mut entries = Vec::new();
        let mut trailing_slots = Vec::new();
        loop {
            leading_slots.push(next_leading);
            let key_span = self.parse_string()?;
            let key = serde_json::from_str::<String>(&self.source[key_span.clone()])
                .context("Failed to parse object key")?;

            let between_start = self.pos;
            self.parse_trivia()?;
            self.expect_byte(b':')?;
            self.parse_trivia()?;
            let between = between_start..self.pos;

            let value = self.parse_value()?;
            let after = self.parse_trivia()?;

            entries.push(ObjectEntry {
                key_span,
                key,
                between,
                value,
            });
            trailing_slots.push(after);

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

        Ok(ObjectNode {
            leading_slots,
            entries,
            trailing_slots,
            empty_trivia: None,
        })
    }

    fn parse_array(&mut self) -> Result<ArrayNode> {
        self.expect_byte(b'[')?;

        let mut next_leading = self.parse_trivia()?;
        if self.peek_byte() == Some(b']') {
            self.pos += 1;
            return Ok(ArrayNode {
                items: Vec::new(),
                empty_trivia: Some(next_leading),
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
            items,
            empty_trivia: None,
        })
    }

    fn parse_string(&mut self) -> Result<Range<usize>> {
        self.expect_byte(b'"')?;
        let start = self.pos - 1;

        while let Some(byte) = self.peek_byte() {
            match byte {
                b'"' => {
                    self.pos += 1;
                    return Ok(start..self.pos);
                }
                b'\\' => {
                    self.pos += 1;
                    if self.peek_byte().is_none() {
                        bail!("Unterminated escape sequence at byte {}", self.pos);
                    }
                    self.pos += 1;
                }
                _ => {
                    self.pos += 1;
                }
            }
        }

        bail!("Unterminated string literal")
    }

    fn parse_number(&mut self) -> Result<Range<usize>> {
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

        Ok(start..self.pos)
    }

    fn parse_literal(&mut self, literal: &str) -> Result<Range<usize>> {
        let start = self.pos;
        if self.bytes[start..].starts_with(literal.as_bytes()) {
            self.pos += literal.len();
            return Ok(start..self.pos);
        }

        bail!("Expected '{}' at byte {}", literal, start)
    }

    fn parse_trivia(&mut self) -> Result<Range<usize>> {
        let start = self.pos;

        loop {
            let checkpoint = self.pos;

            while matches!(self.peek_byte(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
                self.pos += 1;
            }

            if self.bytes[self.pos..].starts_with(b"//") {
                self.pos += 2;
                while let Some(byte) = self.peek_byte() {
                    if byte == b'\n' {
                        break;
                    }
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

        Ok(start..self.pos)
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
