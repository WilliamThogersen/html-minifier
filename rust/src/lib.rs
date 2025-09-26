use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// =============================================================================
// HTML Element and Attribute Constants
// =============================================================================

const SINGLETON_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

const CLOSE_OPTIONAL_ELEMENTS: &[&str] = &[
    "p", "dt", "dd", "li", "option", "thead", "th", "tbody", "tr", "td", "tfoot", "colgroup",
];

const BOOLEAN_ATTRIBUTES: &[&str] = &[
    "allowfullscreen",
    "async",
    "autofocus",
    "autoplay",
    "checked",
    "controls",
    "default",
    "defer",
    "disabled",
    "formnovalidate",
    "hidden",
    "inert",
    "ismap",
    "itemscope",
    "loop",
    "multiple",
    "muted",
    "nomodule",
    "novalidate",
    "open",
    "playsinline",
    "readonly",
    "required",
    "reversed",
    "selected",
    "typemustmatch",
];

const EMPTY_REMOVABLE_ATTRIBUTES: &[&str] = &[
    "id",
    "class",
    "style",
    "title",
    "action",
    "lang",
    "dir",
    "onfocus",
    "onblur",
    "onchange",
    "onclick",
    "ondblclick",
    "onmousedown",
    "onmouseup",
    "onmouseover",
    "onmousemove",
    "onmouseout",
    "onkeypress",
    "onkeydown",
    "onkeyup",
    "target",
];

// =============================================================================
// Token Types
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
enum Token<'a> {
    TextNode(&'a str),
    TagOpenStart(&'a str),
    Attribute(&'a str),
    TagOpenEnd,
    TagSelfClose,
    TagClose(&'a str),
    Comment(&'a str),
    Doctype(&'a str),
    Cdata(&'a str),
}

// =============================================================================
// HTML Element Utilities
// =============================================================================

fn is_singleton_element(tag: &str) -> bool {
    SINGLETON_ELEMENTS.contains(&tag)
}

fn is_close_optional(tag: &str) -> bool {
    CLOSE_OPTIONAL_ELEMENTS.contains(&tag)
}

fn is_boolean_attribute(attr: &str) -> bool {
    BOOLEAN_ATTRIBUTES.contains(&attr)
}

fn is_empty_removable(attr: &str) -> bool {
    EMPTY_REMOVABLE_ATTRIBUTES.contains(&attr)
}

fn has_default_value(tag: &str, attr: &str, value: &str) -> bool {
    match (tag, attr, value) {
        ("script", "type", "text/javascript") => true,
        ("style", "type", "text/css") => true,
        ("style", "media", "all") => true,
        ("form", "method", "get") => true,
        ("form", "autocomplete", "on") => true,
        ("form", "enctype", "application/x-www-form-urlencoded") => true,
        ("input", "type", "text") => true,
        ("button", "type", "submit") => true,
        _ => false,
    }
}

fn should_remove_quotes(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }

    !value.chars().any(|c| {
        c.is_whitespace()
            || matches!(
                c,
                '"' | '\''
                    | '`'
                    | '='
                    | '<'
                    | '>'
                    | '&'
                    | '?'
                    | '{'
                    | '}'
                    | '['
                    | ']'
                    | '('
                    | ')'
                    | ';'
                    | ','
                    | '+'
            )
    }) && value.chars().all(|c| {
        c.is_alphanumeric()
            || matches!(
                c,
                '-' | '_' | '.' | ':' | '/' | '#' | '@' | '%' | '!' | '*' | '~'
            )
    })
}

// =============================================================================
// Tokenizer
// =============================================================================

struct Tokenizer<'a> {
    input: &'a str,
    position: usize,
    end: usize,
    bytes: &'a [u8],
    in_tag: bool,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            position: 0,
            end: input.len(),
            bytes: input.as_bytes(),
            in_tag: false,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.end {
            match self.bytes[self.position] {
                b' ' | b'\t' | b'\n' | b'\r' => self.position += 1,
                _ => break,
            }
        }
    }

    fn consume_until_bytes(&mut self, delimiter: &[u8]) -> &'a str {
        let start = self.position;

        while self.position < self.end {
            if self.position + delimiter.len() <= self.end
                && &self.bytes[self.position..self.position + delimiter.len()] == delimiter
            {
                let result = &self.input[start..self.position];
                self.position += delimiter.len();
                return result;
            }
            self.position += 1;
        }

        &self.input[start..self.end]
    }

    fn consume_tag_name(&mut self) -> &'a str {
        let start = self.position;

        while self.position < self.end {
            match self.bytes[self.position] {
                b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r' => break,
                _ => self.position += 1,
            }
        }

        &self.input[start..self.position]
    }

    fn consume_until_byte(&mut self, byte: u8) -> &'a str {
        let start = self.position;

        while self.position < self.end && self.bytes[self.position] != byte {
            self.position += 1;
        }

        &self.input[start..self.position]
    }

    fn consume_attribute(&mut self) -> Option<&'a str> {
        self.skip_whitespace();
        if self.position >= self.end || self.bytes[self.position] == b'>' {
            return None;
        }

        let start = self.position;
        let mut has_equals = false;

        // Find the attribute name
        while self.position < self.end {
            match self.bytes[self.position] {
                b'=' => {
                    has_equals = true;
                    self.position += 1;
                    break;
                }
                b' ' | b'\t' | b'\n' | b'\r' | b'>' => break,
                _ => self.position += 1,
            }
        }

        // If we found '=', consume the value
        if has_equals {
            self.skip_whitespace();

            // Check if value is quoted
            if self.position < self.end && matches!(self.bytes[self.position], b'"' | b'\'') {
                let quote_char = self.bytes[self.position];
                self.position += 1;

                // Consume until closing quote
                while self.position < self.end {
                    if self.bytes[self.position] == quote_char {
                        self.position += 1;
                        break;
                    }
                    self.position += 1;
                }
            } else {
                // Unquoted value - consume until whitespace or >
                while self.position < self.end {
                    match self.bytes[self.position] {
                        b' ' | b'\t' | b'\n' | b'\r' | b'>' => break,
                        _ => self.position += 1,
                    }
                }
            }
        }

        if self.position > start {
            Some(&self.input[start..self.position])
        } else {
            None
        }
    }

    fn next_token(&mut self) -> Option<Token<'a>> {
        self.skip_whitespace();

        if self.position >= self.end {
            return None;
        }

        // Handle attributes if we're inside a tag
        if self.in_tag {
            if self.position < self.end && self.bytes[self.position] == b'>' {
                self.position += 1;
                self.in_tag = false;
                return Some(Token::TagOpenEnd);
            }

            if self.position + 1 < self.end
                && self.bytes[self.position] == b'/'
                && self.bytes[self.position + 1] == b'>'
            {
                self.position += 2;
                self.in_tag = false;
                return Some(Token::TagSelfClose);
            }

            if let Some(attr) = self.consume_attribute() {
                return Some(Token::Attribute(attr));
            }

            // If we can't parse an attribute, exit tag mode
            self.in_tag = false;
        }

        match self.bytes[self.position] {
            b'<' => self.parse_tag(),
            _ => self.parse_text_node(),
        }
    }

    fn parse_tag(&mut self) -> Option<Token<'a>> {
        self.position += 1;
        if self.position >= self.end {
            return None;
        }

        match self.bytes[self.position] {
            b'!' => self.parse_special_tag(),
            b'/' => self.parse_close_tag(),
            _ => self.parse_open_tag(),
        }
    }

    fn parse_special_tag(&mut self) -> Option<Token<'a>> {
        self.position += 1;

        if self.position + 2 < self.end && &self.bytes[self.position..self.position + 2] == b"--" {
            // Comment
            self.position += 2;
            let content = self.consume_until_bytes(b"-->");
            Some(Token::Comment(content))
        } else if self.position + 7 < self.end
            && &self.bytes[self.position..self.position + 7] == b"DOCTYPE"
        {
            // Doctype
            let start = self.position - 2;
            let _content = self.consume_until_byte(b'>');
            if self.position < self.end && self.bytes[self.position] == b'>' {
                self.position += 1;
            }
            Some(Token::Doctype(&self.input[start..self.position]))
        } else if self.position + 7 < self.end
            && &self.bytes[self.position..self.position + 7] == b"[CDATA["
        {
            // CDATA
            self.position += 7;
            let content = self.consume_until_bytes(b"]]>");
            Some(Token::Cdata(content))
        } else {
            // Other special content
            let start = self.position - 2;
            let _content = self.consume_until_byte(b'>');
            if self.position < self.end && self.bytes[self.position] == b'>' {
                self.position += 1;
            }
            Some(Token::Comment(&self.input[start..self.position]))
        }
    }

    fn parse_close_tag(&mut self) -> Option<Token<'a>> {
        self.position += 1;
        let tag_name = self.consume_until_byte(b'>');
        if self.position < self.end && self.bytes[self.position] == b'>' {
            self.position += 1;
        }
        Some(Token::TagClose(tag_name))
    }

    fn parse_open_tag(&mut self) -> Option<Token<'a>> {
        let tag_name = self.consume_tag_name();
        self.in_tag = true;
        Some(Token::TagOpenStart(tag_name))
    }

    fn parse_text_node(&mut self) -> Option<Token<'a>> {
        let start = self.position;
        while self.position < self.end && self.bytes[self.position] != b'<' {
            self.position += 1;
        }

        if self.position > start {
            Some(Token::TextNode(&self.input[start..self.position]))
        } else {
            None
        }
    }
}

// =============================================================================
// CSS and JavaScript Minifiers
// =============================================================================

pub fn minify_javascript(js: &str) -> String {
    let mut result = String::with_capacity(js.len());
    let mut chars = js.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            // Handle string literals - preserve them completely
            '"' | '\'' | '`' => {
                result.push(ch);
                let quote = ch;

                while let Some(inner_ch) = chars.next() {
                    result.push(inner_ch);
                    if inner_ch == quote {
                        break;
                    }
                    // Handle escaped characters
                    if inner_ch == '\\' {
                        if let Some(escaped) = chars.next() {
                            result.push(escaped);
                        }
                    }
                }
            }
            // Handle comments
            '/' => {
                if let Some(&'/') = chars.peek() {
                    // Line comment - skip to end of line
                    chars.next(); // consume second '/'
                    while let Some(c) = chars.next() {
                        if c == '\n' {
                            break;
                        }
                    }
                } else if let Some(&'*') = chars.peek() {
                    // Block comment - skip to */
                    chars.next(); // consume '*'
                    let mut prev = ' ';
                    while let Some(c) = chars.next() {
                        if prev == '*' && c == '/' {
                            break;
                        }
                        prev = c;
                    }
                } else {
                    result.push(ch);
                }
            }
            // Handle whitespace conservatively
            c if c.is_whitespace() => {
                if !result.is_empty() && !result.ends_with(' ') {
                    // Check if we need a space for separation
                    if let Some(&next_ch) = chars.peek() {
                        let last_ch = result.chars().last().unwrap_or(' ');
                        if (last_ch.is_alphanumeric() || last_ch == '_')
                            && (next_ch.is_alphanumeric() || next_ch == '_')
                        {
                            result.push(' ');
                        }
                    }
                }
                // Skip consecutive whitespace
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_whitespace() {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
            _ => result.push(ch),
        }
    }

    result.trim().to_string()
}

pub fn minify_css(css: &str) -> String {
    let mut result = String::with_capacity(css.len());
    let mut chars = css.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '/' if chars.peek() == Some(&'*') => {
                // Skip CSS comments
                chars.next(); // consume '*'
                let mut prev = ' ';
                while let Some(c) = chars.next() {
                    if prev == '*' && c == '/' {
                        break;
                    }
                    prev = c;
                }
            }
            c if c.is_whitespace() => {
                // Skip unnecessary whitespace
                if !result.is_empty() {
                    let last_ch = result.chars().last().unwrap_or(' ');
                    if !matches!(last_ch, '{' | '}' | ':' | ';' | ',' | '>' | '+' | '~') {
                        if !result.ends_with(' ') {
                            result.push(' ');
                        }
                    }
                }
                // Skip consecutive whitespace
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_whitespace() {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
            _ => result.push(ch),
        }
    }

    result.trim().to_string()
}

// =============================================================================
// HTML Processing Utilities
// =============================================================================

fn append_collapsed_whitespace(result: &mut String, content: &str) {
    let mut prev_was_space = false;
    for ch in content.chars() {
        if ch.is_whitespace() {
            if !prev_was_space {
                result.push(' ');
                prev_was_space = true;
            }
        } else {
            result.push(ch);
            prev_was_space = false;
        }
    }
}

fn process_style_attribute(value: &str) -> String {
    let minified_style = value
        .replace('\t', " ")
        .replace('\r', "")
        .replace('\n', " ");

    let mut style_result = String::with_capacity(minified_style.len());
    let mut prev_space = false;

    for ch in minified_style.chars() {
        match ch {
            ' ' => {
                if !prev_space {
                    style_result.push(' ');
                    prev_space = true;
                }
            }
            ':' | ';' => {
                while style_result.ends_with(' ') {
                    style_result.pop();
                }
                style_result.push(ch);
                prev_space = false;
            }
            _ => {
                style_result.push(ch);
                prev_space = false;
            }
        }
    }

    style_result.trim_end_matches(';').trim().to_string()
}

fn process_class_attribute(value: &str) -> String {
    let mut class_result = String::with_capacity(value.len());
    let mut prev_space = false;

    for ch in value.chars() {
        if ch.is_whitespace() {
            if !prev_space && !class_result.is_empty() {
                class_result.push(' ');
                prev_space = true;
            }
        } else {
            class_result.push(ch);
            prev_space = false;
        }
    }

    class_result
}

// =============================================================================
// Main HTML Minifier
// =============================================================================

pub fn minify_html_tokens(html: &str) -> String {
    let mut result = String::with_capacity(html.len() * 3 / 4);
    let mut tokenizer = Tokenizer::new(html);
    let mut in_pre_tag = false;
    let mut in_script_tag = false;
    let mut in_style_tag = false;
    let mut current_tag = String::new();

    while let Some(token) = tokenizer.next_token() {
        match token {
            Token::Doctype(content) => {
                result.push_str(&content.to_lowercase());
            }
            Token::Comment(_) => {
                // Skip comments for minification
            }
            Token::Cdata(content) => {
                result.push_str("<![CDATA[");
                result.push_str(content);
                result.push_str("]]>");
            }
            Token::TagOpenStart(tag_name) => {
                current_tag = tag_name.to_lowercase();
                in_pre_tag = matches!(current_tag.as_str(), "pre" | "code" | "textarea");
                in_script_tag = current_tag == "script";
                in_style_tag = current_tag == "style";

                result.push('<');
                result.push_str(&current_tag);
            }
            Token::Attribute(attr) => {
                process_attribute(&mut result, attr, &current_tag);
            }
            Token::TagOpenEnd => {
                result.push('>');
            }
            Token::TagSelfClose => {
                if is_singleton_element(&current_tag) {
                    result.push('>');
                } else {
                    result.push_str("/>");
                }
            }
            Token::TagClose(tag_name) => {
                let tag_lower = tag_name.to_lowercase();

                if !is_close_optional(&tag_lower) {
                    result.push_str("</");
                    result.push_str(&tag_lower);
                    result.push('>');
                }

                // Update context flags
                if matches!(tag_lower.as_str(), "pre" | "code" | "textarea") {
                    in_pre_tag = false;
                }
                if tag_lower == "script" {
                    in_script_tag = false;
                }
                if tag_lower == "style" {
                    in_style_tag = false;
                }
            }
            Token::TextNode(content) => {
                if in_style_tag {
                    let minified_css = minify_css(content);
                    result.push_str(&minified_css);
                } else if in_script_tag {
                    let minified_js = minify_javascript(content);
                    result.push_str(&minified_js);
                } else if in_pre_tag {
                    result.push_str(content);
                } else {
                    append_collapsed_whitespace(&mut result, content);
                }
            }
        }
    }

    // Final cleanup with UTF-8 awareness
    cleanup_html_spacing(&result)
}

fn process_attribute(result: &mut String, attr: &str, current_tag: &str) {
    let clean_attr = attr.trim();
    if clean_attr.is_empty() {
        return;
    }

    if let Some(eq_pos) = clean_attr.find('=') {
        let key = clean_attr[..eq_pos].trim().to_lowercase();
        let raw_value = clean_attr[eq_pos + 1..].trim();

        let value = if raw_value.len() >= 2
            && ((raw_value.starts_with('"') && raw_value.ends_with('"'))
                || (raw_value.starts_with('\'') && raw_value.ends_with('\'')))
        {
            &raw_value[1..raw_value.len() - 1]
        } else {
            raw_value
        };

        // Handle boolean attributes
        if is_boolean_attribute(&key) {
            result.push(' ');
            result.push_str(&key);
            return;
        }

        // Skip empty removable attributes
        if value.is_empty() {
            if is_empty_removable(&key)
                || matches!(key.as_str(), "type" | "value" | "alt" | "title")
            {
                return;
            }
        }

        // Skip attributes with default values
        if has_default_value(current_tag, &key, value) {
            return;
        }

        result.push(' ');
        result.push_str(&key);
        result.push('=');

        // Process specific attribute types
        let processed_value = match key.as_str() {
            "style" => process_style_attribute(value),
            "class" if value.contains(' ') => process_class_attribute(value),
            _ => value.to_string(),
        };

        // Add quotes if necessary
        if should_remove_quotes(&processed_value) {
            result.push_str(&processed_value);
        } else {
            result.push('"');
            result.push_str(&processed_value);
            result.push('"');
        }
    } else {
        // Attribute without value
        let key = clean_attr.to_lowercase();
        if !is_empty_removable(&key) {
            result.push(' ');
            result.push_str(&key);
        }
    }
}

fn cleanup_html_spacing(html: &str) -> String {
    let mut cleaned = String::with_capacity(html.len());
    let mut chars = html.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '>' => {
                cleaned.push('>');
                // Skip whitespace after >
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_whitespace() {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
            ch if ch.is_whitespace() => {
                // Check if next character is '<'
                if let Some(&'<') = chars.peek() {
                    continue; // Skip whitespace before <
                }

                // Collapse whitespace
                if !cleaned.ends_with(' ') {
                    cleaned.push(' ');
                }

                // Skip additional whitespace
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_whitespace() {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
            '=' => {
                // Remove spaces around =
                while cleaned.ends_with(' ') {
                    cleaned.pop();
                }
                cleaned.push('=');

                // Skip whitespace after =
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == ' ' {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
            _ => cleaned.push(ch),
        }
    }

    cleaned.trim().to_string()
}

// =============================================================================
// FFI Interface
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn minify_html_string(html_ptr: *const c_char) -> *mut c_char {
    if html_ptr.is_null() {
        return std::ptr::null_mut();
    }

    let c_str = match CStr::from_ptr(html_ptr).to_str() {
        Ok(s) => s,
        Err(_) => {
            // Handle invalid UTF-8 gracefully
            let c_str_bytes = CStr::from_ptr(html_ptr);
            return match CString::new(c_str_bytes.to_string_lossy().as_ref()) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => std::ptr::null_mut(),
            };
        }
    };

    let minified = minify_html_tokens(c_str);

    match CString::new(minified) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn minify_javascript_string(js_ptr: *const c_char) -> *mut c_char {
    if js_ptr.is_null() {
        return std::ptr::null_mut();
    }

    let c_str = match CStr::from_ptr(js_ptr).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let minified = minify_javascript(c_str);

    match CString::new(minified) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);
    }
}
