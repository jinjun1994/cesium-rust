//! 3D Tiles Styling Language expression parser and evaluator matching CesiumJS `Scene/Expression.js`.
use std::collections::HashMap;

use cesium_math::{Color, DeveloperError};
use serde_json::Value;

use crate::style_expression::{StyleValue, TileFeature};

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Number(f64),
    String(String, bool), // (raw_content, has_interpolation)
    Bool(bool),
    Null,
    Ident(String),
    Var(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    EqEq,
    EqEqEq,
    NotEq,
    NotEqEq,
    Lt,
    Le,
    Gt,
    Ge,
    AndAnd,
    OrOr,
    Question,
    Colon,
    Comma,
    Dot,
    LParen,
    RParen,
    LBracket,
    RBracket,
}

fn tokenize(input: &str) -> Result<Vec<Token>, DeveloperError> {
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    let mut tokens = Vec::new();

    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // Variable ${...}
        if c == '$' && i + 1 < chars.len() && chars[i + 1] == '{' {
            i += 2;
            let start = i;
            let mut depth = 1;
            while i < chars.len() && depth > 0 {
                if chars[i] == '{' {
                    depth += 1;
                } else if chars[i] == '}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                i += 1;
            }
            if depth != 0 {
                return Err(DeveloperError::new("Unterminated variable reference."));
            }
            let name: String = chars[start..i].iter().collect();
            i += 1; // skip '}'
            tokens.push(Token::Var(name));
            continue;
        }

        // String literal '...' or "..."
        if c == '\'' || c == '"' {
            let quote = c;
            i += 1;
            let mut s = String::new();
            let mut closed = false;
            while i < chars.len() {
                let ch = chars[i];
                if ch == '\\' && i + 1 < chars.len() {
                    let next = chars[i + 1];
                    match next {
                        'n' => s.push('\n'),
                        'r' => s.push('\r'),
                        't' => s.push('\t'),
                        '\\' => s.push('\\'),
                        '\'' => s.push('\''),
                        '"' => s.push('"'),
                        other => s.push(other),
                    }
                    i += 2;
                    continue;
                }
                if ch == quote {
                    i += 1;
                    closed = true;
                    break;
                }
                s.push(ch);
                i += 1;
            }
            if !closed {
                return Err(DeveloperError::new("Unterminated string literal."));
            }
            let has_interp = s.contains("${");
            tokens.push(Token::String(s, has_interp));
            continue;
        }

        // Number literal
        if c.is_ascii_digit() || (c == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit())
        {
            let start = i;
            if c == '.' {
                i += 1;
            }
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            if i < chars.len() && chars[i] == '.' {
                i += 1;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
            }
            if i < chars.len() && (chars[i] == 'e' || chars[i] == 'E') {
                i += 1;
                if i < chars.len() && (chars[i] == '+' || chars[i] == '-') {
                    i += 1;
                }
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
            }
            let num_str: String = chars[start..i].iter().collect();
            let n = num_str
                .parse::<f64>()
                .map_err(|_| DeveloperError::new(format!("Invalid number literal: {num_str}")))?;
            tokens.push(Token::Number(n));
            continue;
        }

        // Identifier / keyword
        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let ident: String = chars[start..i].iter().collect();
            match ident.as_str() {
                "true" => tokens.push(Token::Bool(true)),
                "false" => tokens.push(Token::Bool(false)),
                "null" | "undefined" => tokens.push(Token::Null),
                _ => tokens.push(Token::Ident(ident)),
            }
            continue;
        }

        // Multi-char operators
        let two: String = chars[i..(i + 2).min(chars.len())].iter().collect();
        let three: String = chars[i..(i + 3).min(chars.len())].iter().collect();
        if three == "===" {
            tokens.push(Token::EqEqEq);
            i += 3;
            continue;
        }
        if three == "!==" {
            tokens.push(Token::NotEqEq);
            i += 3;
            continue;
        }
        match two.as_str() {
            "==" => {
                tokens.push(Token::EqEq);
                i += 2;
                continue;
            }
            "!=" => {
                tokens.push(Token::NotEq);
                i += 2;
                continue;
            }
            "<=" => {
                tokens.push(Token::Le);
                i += 2;
                continue;
            }
            ">=" => {
                tokens.push(Token::Ge);
                i += 2;
                continue;
            }
            "&&" => {
                tokens.push(Token::AndAnd);
                i += 2;
                continue;
            }
            "||" => {
                tokens.push(Token::OrOr);
                i += 2;
                continue;
            }
            _ => {}
        }

        // Single-char operators & punctuation
        let tok = match c {
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,
            '!' => Token::Bang,
            '<' => Token::Lt,
            '>' => Token::Gt,
            '?' => Token::Question,
            ':' => Token::Colon,
            ',' => Token::Comma,
            '.' => Token::Dot,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            other => {
                return Err(DeveloperError::new(format!(
                    "Unexpected character '{other}' in expression."
                )));
            }
        };
        tokens.push(tok);
        i += 1;
    }

    Ok(tokens)
}

#[derive(Clone, Debug)]
enum Ast {
    Literal(StyleValue),
    InterpolatedString(String),
    Var(String),
    Array(Vec<Ast>),
    Unary(UnaryOp, Box<Ast>),
    Binary(BinaryOp, Box<Ast>, Box<Ast>),
    Ternary(Box<Ast>, Box<Ast>, Box<Ast>),
    Call(String, Vec<Ast>),
    Member(Box<Ast>, String),
    Index(Box<Ast>, Box<Ast>),
}

#[derive(Clone, Copy, Debug)]
enum UnaryOp {
    Plus,
    Minus,
    Not,
}

#[derive(Clone, Copy, Debug)]
enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    StrictEq,
    NotEq,
    StrictNotEq,
    And,
    Or,
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<(), DeveloperError> {
        match self.next() {
            Some(tok) if tok == expected => Ok(()),
            other => Err(DeveloperError::new(format!(
                "Expected {expected:?}, found {other:?}"
            ))),
        }
    }

    fn parse(&mut self) -> Result<Ast, DeveloperError> {
        let expr = self.parse_ternary()?;
        if let Some(extra) = self.peek() {
            return Err(DeveloperError::new(format!(
                "Unexpected token {extra:?} in expression."
            )));
        }
        Ok(expr)
    }

    fn parse_ternary(&mut self) -> Result<Ast, DeveloperError> {
        let cond = self.parse_or()?;
        if matches!(self.peek(), Some(Token::Question)) {
            self.next();
            let then_branch = self.parse_ternary()?;
            self.expect(&Token::Colon)?;
            let else_branch = self.parse_ternary()?;
            return Ok(Ast::Ternary(
                Box::new(cond),
                Box::new(then_branch),
                Box::new(else_branch),
            ));
        }
        Ok(cond)
    }

    fn parse_or(&mut self) -> Result<Ast, DeveloperError> {
        let mut lhs = self.parse_and()?;
        while matches!(self.peek(), Some(Token::OrOr)) {
            self.next();
            let rhs = self.parse_and()?;
            lhs = Ast::Binary(BinaryOp::Or, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_and(&mut self) -> Result<Ast, DeveloperError> {
        let mut lhs = self.parse_equality()?;
        while matches!(self.peek(), Some(Token::AndAnd)) {
            self.next();
            let rhs = self.parse_equality()?;
            lhs = Ast::Binary(BinaryOp::And, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_equality(&mut self) -> Result<Ast, DeveloperError> {
        let mut lhs = self.parse_comparison()?;
        loop {
            let op = match self.peek() {
                Some(Token::EqEq) => BinaryOp::Eq,
                Some(Token::EqEqEq) => BinaryOp::StrictEq,
                Some(Token::NotEq) => BinaryOp::NotEq,
                Some(Token::NotEqEq) => BinaryOp::StrictNotEq,
                _ => break,
            };
            self.next();
            let rhs = self.parse_comparison()?;
            lhs = Ast::Binary(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_comparison(&mut self) -> Result<Ast, DeveloperError> {
        let mut lhs = self.parse_additive()?;
        loop {
            let op = match self.peek() {
                Some(Token::Lt) => BinaryOp::Lt,
                Some(Token::Le) => BinaryOp::Le,
                Some(Token::Gt) => BinaryOp::Gt,
                Some(Token::Ge) => BinaryOp::Ge,
                _ => break,
            };
            self.next();
            let rhs = self.parse_additive()?;
            lhs = Ast::Binary(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_additive(&mut self) -> Result<Ast, DeveloperError> {
        let mut lhs = self.parse_multiplicative()?;
        loop {
            let op = match self.peek() {
                Some(Token::Plus) => BinaryOp::Add,
                Some(Token::Minus) => BinaryOp::Sub,
                _ => break,
            };
            self.next();
            let rhs = self.parse_multiplicative()?;
            lhs = Ast::Binary(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_multiplicative(&mut self) -> Result<Ast, DeveloperError> {
        let mut lhs = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Some(Token::Star) => BinaryOp::Mul,
                Some(Token::Slash) => BinaryOp::Div,
                Some(Token::Percent) => BinaryOp::Rem,
                _ => break,
            };
            self.next();
            let rhs = self.parse_unary()?;
            lhs = Ast::Binary(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Ast, DeveloperError> {
        match self.peek() {
            Some(Token::Plus) => {
                self.next();
                Ok(Ast::Unary(UnaryOp::Plus, Box::new(self.parse_unary()?)))
            }
            Some(Token::Minus) => {
                self.next();
                Ok(Ast::Unary(UnaryOp::Minus, Box::new(self.parse_unary()?)))
            }
            Some(Token::Bang) => {
                self.next();
                Ok(Ast::Unary(UnaryOp::Not, Box::new(self.parse_unary()?)))
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Ast, DeveloperError> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek() {
                Some(Token::Dot) => {
                    self.next();
                    match self.next() {
                        Some(Token::Ident(name)) => {
                            expr = Ast::Member(Box::new(expr), name.clone());
                        }
                        other => {
                            return Err(DeveloperError::new(format!(
                                "Expected identifier after '.', found {other:?}"
                            )));
                        }
                    }
                }
                Some(Token::LBracket) => {
                    self.next();
                    let idx = self.parse_ternary()?;
                    self.expect(&Token::RBracket)?;
                    expr = Ast::Index(Box::new(expr), Box::new(idx));
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Ast, DeveloperError> {
        match self.next().cloned() {
            Some(Token::Number(n)) => Ok(Ast::Literal(StyleValue::Number(n))),
            Some(Token::Bool(b)) => Ok(Ast::Literal(StyleValue::Boolean(b))),
            Some(Token::Null) => Ok(Ast::Literal(StyleValue::Null)),
            Some(Token::String(s, false)) => Ok(Ast::Literal(StyleValue::String(s))),
            Some(Token::String(s, true)) => Ok(Ast::InterpolatedString(s)),
            Some(Token::Var(name)) => Ok(Ast::Var(name)),
            Some(Token::LParen) => {
                let inner = self.parse_ternary()?;
                self.expect(&Token::RParen)?;
                Ok(inner)
            }
            Some(Token::LBracket) => {
                let mut items = Vec::new();
                if !matches!(self.peek(), Some(Token::RBracket)) {
                    loop {
                        items.push(self.parse_ternary()?);
                        if matches!(self.peek(), Some(Token::Comma)) {
                            self.next();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(&Token::RBracket)?;
                Ok(Ast::Array(items))
            }
            Some(Token::Ident(name)) => {
                if matches!(self.peek(), Some(Token::LParen)) {
                    self.next();
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Some(Token::RParen)) {
                        loop {
                            args.push(self.parse_ternary()?);
                            if matches!(self.peek(), Some(Token::Comma)) {
                                self.next();
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect(&Token::RParen)?;
                    Ok(Ast::Call(name, args))
                } else {
                    match name.as_str() {
                        "Math" => Ok(Ast::IdentPlaceholder),
                        "NaN" => Ok(Ast::Literal(StyleValue::Number(f64::NAN))),
                        "Infinity" => Ok(Ast::Literal(StyleValue::Number(f64::INFINITY))),
                        _ => Err(DeveloperError::new(format!(
                            "Unknown identifier '{name}' in expression."
                        ))),
                    }
                }
            }
            other => Err(DeveloperError::new(format!(
                "Unexpected token {other:?} in expression."
            ))),
        }
    }
}

// Placeholder variant helper on Ast
trait AstExt {
    #[allow(non_upper_case_globals)]
    const IdentPlaceholder: Ast = Ast::Literal(StyleValue::Null);
}
impl AstExt for Ast {}

/// 3D Tiles Styling expression matching CesiumJS `Scene/Expression.js`.
#[derive(Clone, Debug, PartialEq)]
pub struct Expression {
    pub expression: String,
    pub defines: HashMap<String, String>,
    ast: Ast,
}

impl Expression {
    /// Creates and parses an [`Expression`] without defines.
    pub fn new(expression: impl Into<String>) -> Result<Self, DeveloperError> {
        Self::with_defines(expression, HashMap::new())
    }

    /// Creates and parses an [`Expression`] with style `defines`.
    pub fn with_defines(
        expression: impl Into<String>,
        defines: HashMap<String, String>,
    ) -> Result<Self, DeveloperError> {
        let expression = expression.into();
        let expanded = expand_defines(&expression, &defines, 0)?;
        let tokens = tokenize(&expanded)?;
        let mut parser = Parser {
            tokens: &tokens,
            pos: 0,
        };
        let ast = parser.parse()?;
        Ok(Self {
            expression,
            defines,
            ast,
        })
    }

    /// Evaluates the expression against `feature`.
    pub fn evaluate<F: TileFeature>(&self, feature: &F) -> Result<StyleValue, DeveloperError> {
        eval_ast(&self.ast, feature, &self.defines)
    }

    /// Evaluates the expression as a [`Color`].
    pub fn evaluate_color<F: TileFeature>(&self, feature: &F) -> Result<Color, DeveloperError> {
        let val = self.evaluate(feature)?;
        style_value_to_color(&val)
    }
}

impl PartialEq for Ast {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

fn expand_defines(
    expr: &str,
    defines: &HashMap<String, String>,
    depth: usize,
) -> Result<String, DeveloperError> {
    if defines.is_empty() || depth > 16 {
        return Ok(expr.to_string());
    }
    let mut out = expr.to_string();
    let mut changed = true;
    let mut iter = 0;
    while changed && iter < 16 {
        changed = false;
        iter += 1;
        for (k, v) in defines {
            let needle = format!("${{{k}}}");
            if out.contains(&needle) {
                out = out.replace(&needle, &format!("({v})"));
                changed = true;
            }
        }
    }
    Ok(out)
}

fn json_to_style_value(val: &Value) -> StyleValue {
    match val {
        Value::Null => StyleValue::Null,
        Value::Bool(b) => StyleValue::Boolean(*b),
        Value::Number(n) => StyleValue::Number(n.as_f64().unwrap_or(0.0)),
        Value::String(s) => StyleValue::String(s.clone()),
        Value::Array(arr) => StyleValue::Array(arr.iter().map(json_to_style_value).collect()),
        Value::Object(obj) => {
            // Support serialized Cartesian2/3/4 or Color
            if let (Some(x), Some(y)) = (
                obj.get("x").and_then(Value::as_f64),
                obj.get("y").and_then(Value::as_f64),
            ) {
                if let Some(z) = obj.get("z").and_then(Value::as_f64) {
                    if let Some(w) = obj.get("w").and_then(Value::as_f64) {
                        return StyleValue::Array(vec![
                            StyleValue::Number(x),
                            StyleValue::Number(y),
                            StyleValue::Number(z),
                            StyleValue::Number(w),
                        ]);
                    }
                    return StyleValue::Array(vec![
                        StyleValue::Number(x),
                        StyleValue::Number(y),
                        StyleValue::Number(z),
                    ]);
                }
                return StyleValue::Array(vec![StyleValue::Number(x), StyleValue::Number(y)]);
            }
            if let (Some(r), Some(g), Some(b)) = (
                obj.get("red").and_then(Value::as_f64),
                obj.get("green").and_then(Value::as_f64),
                obj.get("blue").and_then(Value::as_f64),
            ) {
                let a = obj.get("alpha").and_then(Value::as_f64).unwrap_or(1.0);
                return StyleValue::Color(Color::new(r as f32, g as f32, b as f32, a as f32));
            }
            StyleValue::Null
        }
    }
}

fn style_value_to_string(val: &StyleValue) -> String {
    match val {
        StyleValue::Boolean(b) => b.to_string(),
        StyleValue::Number(n) => {
            if n.fract() == 0.0 && n.abs() < 1e15 {
                format!("{}", *n as i64)
            } else {
                n.to_string()
            }
        }
        StyleValue::String(s) => s.clone(),
        StyleValue::Color(c) => format!("rgba({},{},{},{})", c.red, c.green, c.blue, c.alpha),
        StyleValue::Array(items) => {
            let parts: Vec<String> = items.iter().map(style_value_to_string).collect();
            format!("[{}]", parts.join(","))
        }
        StyleValue::Null => "null".to_string(),
    }
}

fn style_value_to_bool(val: &StyleValue) -> bool {
    match val {
        StyleValue::Boolean(b) => *b,
        StyleValue::Number(n) => *n != 0.0 && !n.is_nan(),
        StyleValue::String(s) => !s.is_empty(),
        StyleValue::Color(_) | StyleValue::Array(_) => true,
        StyleValue::Null => false,
    }
}

fn style_value_to_number(val: &StyleValue) -> Result<f64, DeveloperError> {
    match val {
        StyleValue::Number(n) => Ok(*n),
        StyleValue::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),
        StyleValue::String(s) => s
            .trim()
            .parse::<f64>()
            .map_err(|_| DeveloperError::new(format!("Cannot convert '{s}' to number."))),
        StyleValue::Null => Ok(0.0),
        _ => Err(DeveloperError::new("Value is not a number.")),
    }
}

fn style_value_to_vec(val: &StyleValue) -> Option<Vec<f64>> {
    match val {
        StyleValue::Color(c) => Some(vec![
            c.red as f64,
            c.green as f64,
            c.blue as f64,
            c.alpha as f64,
        ]),
        StyleValue::Array(arr) => arr
            .iter()
            .map(|v| match v {
                StyleValue::Number(n) => Some(*n),
                _ => None,
            })
            .collect(),
        _ => None,
    }
}

fn style_value_to_color(val: &StyleValue) -> Result<Color, DeveloperError> {
    match val {
        StyleValue::Color(c) => Ok(*c),
        StyleValue::Array(arr) if arr.len() == 4 => {
            let r = style_value_to_number(&arr[0])? as f32;
            let g = style_value_to_number(&arr[1])? as f32;
            let b = style_value_to_number(&arr[2])? as f32;
            let a = style_value_to_number(&arr[3])? as f32;
            Ok(Color::new(r, g, b, a))
        }
        StyleValue::String(s) => parse_css_color(s),
        _ => Err(DeveloperError::new("Expression did not evaluate to a Color.")),
    }
}

fn parse_hex_byte(s: &str) -> Result<u8, DeveloperError> {
    u8::from_str_radix(s, 16).map_err(|_| DeveloperError::new("Invalid hex color component."))
}

fn parse_css_color(spec: &str) -> Result<Color, DeveloperError> {
    let s = spec.trim();
    if let Some(hex) = s.strip_prefix('#') {
        return match hex.len() {
            3 => {
                let r = parse_hex_byte(&hex[0..1])? * 17;
                let g = parse_hex_byte(&hex[1..2])? * 17;
                let b = parse_hex_byte(&hex[2..3])? * 17;
                Ok(Color::from_bytes(r, g, b, 255))
            }
            6 => {
                let r = parse_hex_byte(&hex[0..2])?;
                let g = parse_hex_byte(&hex[2..4])?;
                let b = parse_hex_byte(&hex[4..6])?;
                Ok(Color::from_bytes(r, g, b, 255))
            }
            8 => {
                let r = parse_hex_byte(&hex[0..2])?;
                let g = parse_hex_byte(&hex[2..4])?;
                let b = parse_hex_byte(&hex[4..6])?;
                let a = parse_hex_byte(&hex[6..8])?;
                Ok(Color::from_bytes(r, g, b, a))
            }
            _ => Err(DeveloperError::new(format!("Invalid hex color: {spec}"))),
        };
    }
    match s.to_ascii_lowercase().as_str() {
        "white" => Ok(Color::WHITE),
        "black" => Ok(Color::BLACK),
        "red" => Ok(Color::RED),
        "green" => Ok(Color::GREEN),
        "blue" => Ok(Color::BLUE),
        "yellow" => Ok(Color::YELLOW),
        "cyan" => Ok(Color::CYAN),
        "magenta" | "fuchsia" => Ok(Color::MAGENTA),
        "lime" => Ok(Color::LIME),
        "transparent" => Ok(Color::TRANSPARENT),
        "orange" => Ok(Color::from_bytes(255, 165, 0, 255)),
        "coral" => Ok(Color::from_bytes(255, 127, 80, 255)),
        "gray" | "grey" => Ok(Color::from_bytes(128, 128, 128, 255)),
        other => Err(DeveloperError::new(format!("Unknown CSS color: {other}"))),
    }
}

fn eval_ast<F: TileFeature>(
    ast: &Ast,
    feature: &F,
    defines: &HashMap<String, String>,
) -> Result<StyleValue, DeveloperError> {
    match ast {
        Ast::Literal(v) => Ok(v.clone()),
        Ast::InterpolatedString(template) => {
            let mut result = String::new();
            let chars: Vec<char> = template.chars().collect();
            let mut i = 0;
            while i < chars.len() {
                if chars[i] == '$' && i + 1 < chars.len() && chars[i + 1] == '{' {
                    if let Some(close_rel) = chars[i + 2..].iter().position(|&c| c == '}') {
                        let var_name: String = chars[i + 2..i + 2 + close_rel].iter().collect();
                        let val = feature
                            .get_property_inherited(&var_name)
                            .map(|v| json_to_style_value(&v))
                            .unwrap_or(StyleValue::Null);
                        result.push_str(&style_value_to_string(&val));
                        i = i + 2 + close_rel + 1;
                        continue;
                    }
                }
                result.push(chars[i]);
                i += 1;
            }
            Ok(StyleValue::String(result))
        }
        Ast::Var(name) => {
            if let Some(def_expr) = defines.get(name) {
                let sub = Expression::with_defines(def_expr.clone(), defines.clone())?;
                return sub.evaluate(feature);
            }
            Ok(feature
                .get_property_inherited(name)
                .map(|v| json_to_style_value(&v))
                .unwrap_or(StyleValue::Null))
        }
        Ast::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(eval_ast(item, feature, defines)?);
            }
            Ok(StyleValue::Array(out))
        }
        Ast::Unary(op, inner) => {
            let v = eval_ast(inner, feature, defines)?;
            match op {
                UnaryOp::Plus => Ok(StyleValue::Number(style_value_to_number(&v)?)),
                UnaryOp::Minus => {
                    if let Some(vec) = style_value_to_vec(&v) {
                        Ok(StyleValue::Array(
                            vec.into_iter().map(|x| StyleValue::Number(-x)).collect(),
                        ))
                    } else {
                        Ok(StyleValue::Number(-style_value_to_number(&v)?))
                    }
                }
                UnaryOp::Not => Ok(StyleValue::Boolean(!style_value_to_bool(&v))),
            }
        }
        Ast::Binary(op, lhs_ast, rhs_ast) => {
            if matches!(op, BinaryOp::And) {
                let l = eval_ast(lhs_ast, feature, defines)?;
                if !style_value_to_bool(&l) {
                    return Ok(StyleValue::Boolean(false));
                }
                let r = eval_ast(rhs_ast, feature, defines)?;
                return Ok(StyleValue::Boolean(style_value_to_bool(&r)));
            }
            if matches!(op, BinaryOp::Or) {
                let l = eval_ast(lhs_ast, feature, defines)?;
                if style_value_to_bool(&l) {
                    return Ok(StyleValue::Boolean(true));
                }
                let r = eval_ast(rhs_ast, feature, defines)?;
                return Ok(StyleValue::Boolean(style_value_to_bool(&r)));
            }

            let l = eval_ast(lhs_ast, feature, defines)?;
            let r = eval_ast(rhs_ast, feature, defines)?;
            eval_binary(*op, l, r)
        }
        Ast::Ternary(cond, t, e) => {
            let c = eval_ast(cond, feature, defines)?;
            if style_value_to_bool(&c) {
                eval_ast(t, feature, defines)
            } else {
                eval_ast(e, feature, defines)
            }
        }
        Ast::Member(target, field) => {
            let v = eval_ast(target, feature, defines)?;
            match (&v, field.as_str()) {
                (StyleValue::Color(c), "r" | "red") => Ok(StyleValue::Number(c.red as f64)),
                (StyleValue::Color(c), "g" | "green") => Ok(StyleValue::Number(c.green as f64)),
                (StyleValue::Color(c), "b" | "blue") => Ok(StyleValue::Number(c.blue as f64)),
                (StyleValue::Color(c), "a" | "alpha") => Ok(StyleValue::Number(c.alpha as f64)),
                (StyleValue::Array(arr), "x" | "r") => {
                    Ok(arr.first().cloned().unwrap_or(StyleValue::Null))
                }
                (StyleValue::Array(arr), "y" | "g") => {
                    Ok(arr.get(1).cloned().unwrap_or(StyleValue::Null))
                }
                (StyleValue::Array(arr), "z" | "b") => {
                    Ok(arr.get(2).cloned().unwrap_or(StyleValue::Null))
                }
                (StyleValue::Array(arr), "w" | "a") => {
                    Ok(arr.get(3).cloned().unwrap_or(StyleValue::Null))
                }
                (StyleValue::Array(arr), "length") => Ok(StyleValue::Number(arr.len() as f64)),
                _ => Err(DeveloperError::new(format!("Unknown member '{field}'"))),
            }
        }
        Ast::Index(target, idx_ast) => {
            let v = eval_ast(target, feature, defines)?;
            let idx = style_value_to_number(&eval_ast(idx_ast, feature, defines)?)? as usize;
            match v {
                StyleValue::Array(arr) => Ok(arr.get(idx).cloned().unwrap_or(StyleValue::Null)),
                _ => Err(DeveloperError::new("Indexing requires an array.")),
            }
        }
        Ast::Call(name, args) => {
            let mut vals = Vec::with_capacity(args.len());
            for a in args {
                vals.push(eval_ast(a, feature, defines)?);
            }
            eval_call(name, &vals)
        }
    }
}

fn eval_binary(op: BinaryOp, l: StyleValue, r: StyleValue) -> Result<StyleValue, DeveloperError> {
    // Component-wise vector/color arithmetic
    if let (Some(vl), Some(vr)) = (style_value_to_vec(&l), style_value_to_vec(&r)) {
        if vl.len() == vr.len() {
            let res: Option<Vec<f64>> = match op {
                BinaryOp::Add => Some(vl.iter().zip(&vr).map(|(a, b)| a + b).collect()),
                BinaryOp::Sub => Some(vl.iter().zip(&vr).map(|(a, b)| a - b).collect()),
                BinaryOp::Mul => Some(vl.iter().zip(&vr).map(|(a, b)| a * b).collect()),
                BinaryOp::Div => Some(vl.iter().zip(&vr).map(|(a, b)| a / b).collect()),
                _ => None,
            };
            if let Some(vec) = res {
                if matches!(l, StyleValue::Color(_)) && vec.len() == 4 {
                    return Ok(StyleValue::Color(Color::new(
                        vec[0] as f32,
                        vec[1] as f32,
                        vec[2] as f32,
                        vec[3] as f32,
                    )));
                }
                return Ok(StyleValue::Array(
                    vec.into_iter().map(StyleValue::Number).collect(),
                ));
            }
        }
    }
    if let (Some(vl), StyleValue::Number(s)) = (style_value_to_vec(&l), &r) {
        let res: Option<Vec<f64>> = match op {
            BinaryOp::Mul => Some(vl.iter().map(|a| a * s).collect()),
            BinaryOp::Div => Some(vl.iter().map(|a| a / s).collect()),
            BinaryOp::Add => Some(vl.iter().map(|a| a + s).collect()),
            BinaryOp::Sub => Some(vl.iter().map(|a| a - s).collect()),
            _ => None,
        };
        if let Some(vec) = res {
            if matches!(l, StyleValue::Color(_)) && vec.len() == 4 {
                return Ok(StyleValue::Color(Color::new(
                    vec[0] as f32,
                    vec[1] as f32,
                    vec[2] as f32,
                    vec[3] as f32,
                )));
            }
            return Ok(StyleValue::Array(
                vec.into_iter().map(StyleValue::Number).collect(),
            ));
        }
    }

    match op {
        BinaryOp::Add => match (l, r) {
            (StyleValue::String(a), b) => {
                Ok(StyleValue::String(format!("{a}{}", style_value_to_string(&b))))
            }
            (a, StyleValue::String(b)) => {
                Ok(StyleValue::String(format!("{}{b}", style_value_to_string(&a))))
            }
            (a, b) => Ok(StyleValue::Number(
                style_value_to_number(&a)? + style_value_to_number(&b)?,
            )),
        },
        BinaryOp::Sub => Ok(StyleValue::Number(
            style_value_to_number(&l)? - style_value_to_number(&r)?,
        )),
        BinaryOp::Mul => Ok(StyleValue::Number(
            style_value_to_number(&l)? * style_value_to_number(&r)?,
        )),
        BinaryOp::Div => Ok(StyleValue::Number(
            style_value_to_number(&l)? / style_value_to_number(&r)?,
        )),
        BinaryOp::Rem => Ok(StyleValue::Number(
            style_value_to_number(&l)? % style_value_to_number(&r)?,
        )),
        BinaryOp::Lt => Ok(StyleValue::Boolean(
            style_value_to_number(&l)? < style_value_to_number(&r)?,
        )),
        BinaryOp::Le => Ok(StyleValue::Boolean(
            style_value_to_number(&l)? <= style_value_to_number(&r)?,
        )),
        BinaryOp::Gt => Ok(StyleValue::Boolean(
            style_value_to_number(&l)? > style_value_to_number(&r)?,
        )),
        BinaryOp::Ge => Ok(StyleValue::Boolean(
            style_value_to_number(&l)? >= style_value_to_number(&r)?,
        )),
        BinaryOp::Eq | BinaryOp::StrictEq => Ok(StyleValue::Boolean(l == r)),
        BinaryOp::NotEq | BinaryOp::StrictNotEq => Ok(StyleValue::Boolean(l != r)),
        BinaryOp::And | BinaryOp::Or => unreachable!(),
    }
}

fn eval_call(name: &str, args: &[StyleValue]) -> Result<StyleValue, DeveloperError> {
    match name {
        "color" => {
            let spec = match args.first() {
                Some(StyleValue::String(s)) => s.as_str(),
                _ => return Err(DeveloperError::new("color() requires a string argument.")),
            };
            let mut c = parse_css_color(spec)?;
            if let Some(alpha) = args.get(1) {
                c.alpha = style_value_to_number(alpha)? as f32;
            }
            Ok(StyleValue::Color(c))
        }
        "rgb" if args.len() == 3 => {
            let r = (style_value_to_number(&args[0])? / 255.0) as f32;
            let g = (style_value_to_number(&args[1])? / 255.0) as f32;
            let b = (style_value_to_number(&args[2])? / 255.0) as f32;
            Ok(StyleValue::Color(Color::new(r, g, b, 1.0)))
        }
        "rgba" if args.len() == 4 => {
            let r = (style_value_to_number(&args[0])? / 255.0) as f32;
            let g = (style_value_to_number(&args[1])? / 255.0) as f32;
            let b = (style_value_to_number(&args[2])? / 255.0) as f32;
            let a = style_value_to_number(&args[3])? as f32;
            Ok(StyleValue::Color(Color::new(r, g, b, a)))
        }
        "vec2" => {
            let x = style_value_to_number(args.first().unwrap_or(&StyleValue::Number(0.0)))?;
            let y = style_value_to_number(args.get(1).unwrap_or(&StyleValue::Number(x)))?;
            Ok(StyleValue::Array(vec![
                StyleValue::Number(x),
                StyleValue::Number(y),
            ]))
        }
        "vec3" => {
            let x = style_value_to_number(args.first().unwrap_or(&StyleValue::Number(0.0)))?;
            let y = style_value_to_number(args.get(1).unwrap_or(&StyleValue::Number(x)))?;
            let z = style_value_to_number(args.get(2).unwrap_or(&StyleValue::Number(x)))?;
            Ok(StyleValue::Array(vec![
                StyleValue::Number(x),
                StyleValue::Number(y),
                StyleValue::Number(z),
            ]))
        }
        "vec4" => {
            let x = style_value_to_number(args.first().unwrap_or(&StyleValue::Number(0.0)))?;
            let y = style_value_to_number(args.get(1).unwrap_or(&StyleValue::Number(x)))?;
            let z = style_value_to_number(args.get(2).unwrap_or(&StyleValue::Number(x)))?;
            let w = style_value_to_number(args.get(3).unwrap_or(&StyleValue::Number(x)))?;
            Ok(StyleValue::Array(vec![
                StyleValue::Number(x),
                StyleValue::Number(y),
                StyleValue::Number(z),
                StyleValue::Number(w),
            ]))
        }
        "Boolean" => Ok(StyleValue::Boolean(
            args.first().is_some_and(style_value_to_bool),
        )),
        "Number" => Ok(StyleValue::Number(
            style_value_to_number(args.first().unwrap_or(&StyleValue::Number(0.0)))?,
        )),
        "String" => Ok(StyleValue::String(
            args.first().map(style_value_to_string).unwrap_or_default(),
        )),
        "isNaN" => Ok(StyleValue::Boolean(
            args.first()
                .and_then(|v| style_value_to_number(v).ok())
                .is_none_or(f64::is_nan),
        )),
        "isFinite" => Ok(StyleValue::Boolean(
            args.first()
                .and_then(|v| style_value_to_number(v).ok())
                .is_some_and(f64::is_finite),
        )),
        "abs" if args.len() == 1 => Ok(StyleValue::Number(style_value_to_number(&args[0])?.abs())),
        "sqrt" if args.len() == 1 => {
            Ok(StyleValue::Number(style_value_to_number(&args[0])?.sqrt()))
        }
        "floor" if args.len() == 1 => {
            Ok(StyleValue::Number(style_value_to_number(&args[0])?.floor()))
        }
        "ceil" if args.len() == 1 => {
            Ok(StyleValue::Number(style_value_to_number(&args[0])?.ceil()))
        }
        "round" if args.len() == 1 => {
            Ok(StyleValue::Number(style_value_to_number(&args[0])?.round()))
        }
        "fract" if args.len() == 1 => {
            Ok(StyleValue::Number(style_value_to_number(&args[0])?.fract()))
        }
        "sin" if args.len() == 1 => Ok(StyleValue::Number(style_value_to_number(&args[0])?.sin())),
        "cos" if args.len() == 1 => Ok(StyleValue::Number(style_value_to_number(&args[0])?.cos())),
        "tan" if args.len() == 1 => Ok(StyleValue::Number(style_value_to_number(&args[0])?.tan())),
        "radians" if args.len() == 1 => Ok(StyleValue::Number(
            style_value_to_number(&args[0])?.to_radians(),
        )),
        "degrees" if args.len() == 1 => Ok(StyleValue::Number(
            style_value_to_number(&args[0])?.to_degrees(),
        )),
        "pow" if args.len() == 2 => Ok(StyleValue::Number(
            style_value_to_number(&args[0])?.powf(style_value_to_number(&args[1])?),
        )),
        "min" if args.len() == 2 => Ok(StyleValue::Number(
            style_value_to_number(&args[0])?.min(style_value_to_number(&args[1])?),
        )),
        "max" if args.len() == 2 => Ok(StyleValue::Number(
            style_value_to_number(&args[0])?.max(style_value_to_number(&args[1])?),
        )),
        "clamp" if args.len() == 3 => {
            let v = style_value_to_number(&args[0])?;
            let lo = style_value_to_number(&args[1])?;
            let hi = style_value_to_number(&args[2])?;
            Ok(StyleValue::Number(v.clamp(lo, hi)))
        }
        "mix" if args.len() == 3 => {
            let a = style_value_to_number(&args[0])?;
            let b = style_value_to_number(&args[1])?;
            let t = style_value_to_number(&args[2])?;
            Ok(StyleValue::Number(a + (b - a) * t))
        }
        "length" if args.len() == 1 => {
            let vec = style_value_to_vec(&args[0])
                .ok_or_else(|| DeveloperError::new("length() requires a vector."))?;
            let sum_sq: f64 = vec.iter().map(|x| x * x).sum();
            Ok(StyleValue::Number(sum_sq.sqrt()))
        }
        "dot" if args.len() == 2 => {
            let a = style_value_to_vec(&args[0])
                .ok_or_else(|| DeveloperError::new("dot() requires vectors."))?;
            let b = style_value_to_vec(&args[1])
                .ok_or_else(|| DeveloperError::new("dot() requires vectors."))?;
            let d: f64 = a.iter().zip(&b).map(|(x, y)| x * y).sum();
            Ok(StyleValue::Number(d))
        }
        other => Err(DeveloperError::new(format!(
            "Unsupported function '{other}' in expression."
        ))),
    }
}
