use crate::frame::Cursor;
use tui::style::Color;
use tui::text::Spans;

pub const TAB: &'static str = "    ";

#[derive(Debug, Clone)]
pub enum FlexiblePosition {
    Idx(usize),
    Edge,
}

impl std::ops::Add for FlexiblePosition {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        if let FlexiblePosition::Idx(i0) = self {
            if let FlexiblePosition::Idx(i1) = other {
                return FlexiblePosition::Idx(i0 + i1);
            }
        }

        FlexiblePosition::Edge
    }
}

#[derive(Debug, Clone)]
pub struct StyleRange {
    pub line: usize,
    pub start: FlexiblePosition,
    pub end: FlexiblePosition,
    pub color: Color,
    pub foreground: bool,
}

// #[derive(PartialEq)]
pub struct Buffer {
    lines: Vec<String>,
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            lines: vec![String::new()],
        }
    }

    #[allow(unused)]
    pub fn text_full(&self) -> String {
        self.lines.join("\n")
    }

    pub fn text_styled(&self) -> Vec<Spans> {
        self.lines
            .iter()
            .map(move |line| Spans::from(line.clone()))
            .collect::<Vec<Spans>>()
    }

    pub fn lines_len(&self) -> usize {
        self.lines.len()
    }

    pub fn line_len(&self, cursor: Cursor) -> usize {
        self.lines[cursor.1].len()
    }

    pub fn line_len_idx(&self, idx: usize) -> usize {
        self.lines[idx].len()
    }

    pub fn insert_char(&mut self, cursor: Cursor, c: char) {
        self.lines[cursor.1].insert(cursor.0, c);
    }

    pub fn insert_str(&mut self, cursor: Cursor, s: &str) {
        self.lines[cursor.1].insert_str(cursor.0, s);
    }

    pub fn insert_line(&mut self, cursor: Cursor, s: &str) {
        self.lines.insert(cursor.1, s.to_owned());
    }

    pub fn push_str(&mut self, cursor: Cursor, s: &str) {
        self.lines[cursor.1] += s;
    }

    pub fn remove_char(&mut self, cursor: Cursor) {
        self.lines[cursor.1].remove(cursor.0);
    }

    pub fn delete_line(&mut self, cursor: Cursor) -> String {
        self.lines.remove(cursor.1)
    }

    pub fn split_off(&mut self, cursor: Cursor) -> String {
        self.lines[cursor.1].split_off(cursor.0)
    }

    #[allow(unused)]
    pub fn remove_front(&mut self, cursor: Cursor) -> String {
        self.lines[cursor.1].drain(..cursor.0).collect()
    }
}
