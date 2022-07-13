use crate::buffer::{Buffer, TAB};
use std::cell::RefCell;
use std::rc::Rc;
use tui::layout::{Constraint, Direction as LayoutDirection, Layout, Rect};
use tui::widgets::Borders;

pub type Direction = LayoutDirection;
pub type Cursor = (usize, usize);
pub type Offset = (usize, usize);

pub struct Frame {
    inner_frames: Option<(Rc<RefCell<Frame>>, Rc<RefCell<Frame>>)>,
    split_direction: Direction,
    buffer: Rc<RefCell<Buffer>>,
    cursor: Cursor,
    offset: Offset,
    border_flag: Borders,
    focus: bool,
    x_mode: bool,
    show: bool,
    has_parent: bool,
}

impl Frame {
    pub fn new(buffer: Rc<RefCell<Buffer>>) -> Self {
        Frame {
            inner_frames: None,
            split_direction: Direction::Horizontal,
            buffer,
            cursor: (0, 0),
            offset: (0, 0),
            border_flag: Borders::NONE,
            focus: false,
            x_mode: false,
            show: true,
            has_parent: false,
        }
    }

    pub fn inherit(frame: &Frame, additional_border_flag: Borders) -> Self {
        Frame {
            inner_frames: None,
            split_direction: Direction::Horizontal,
            buffer: frame.buffer.clone(),
            cursor: frame.cursor,
            offset: frame.offset,
            border_flag: frame.border_flag | additional_border_flag,
            focus: false,
            x_mode: false,
            show: true,
            has_parent: true,
        }
    }

    pub fn is_main_frame(&self) -> bool {
        !self.has_parent
    }

    fn lines_len(&self) -> usize {
        self.buffer.borrow().lines_len()
    }

    fn line_len(&self) -> usize {
        self.buffer.borrow().line_len(self.cursor)
    }

    fn line_len_idx(&self, idx: usize) -> usize {
        self.buffer.borrow().line_len_idx(idx)
    }

    pub fn move_left(&mut self) {
        if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.line_len() > self.cursor.0 {
            self.cursor.0 += 1;
        } else if self.lines_len() - 1 > self.cursor.1 {
            self.cursor.0 = 0;
            self.cursor.1 += 1;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor.1 > 0 {
            self.cursor.1 -= 1;

            if self.cursor.0 > self.line_len() {
                self.cursor.0 = self.line_len();
            }
        }
    }

    pub fn move_down(&mut self) {
        if self.lines_len() - 1 > self.cursor.1 {
            self.cursor.1 += 1;

            if self.cursor.0 > self.line_len() {
                self.cursor.0 = self.line_len();
            }
        }
    }

    pub fn move_front(&mut self) {
        self.cursor.0 = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor.0 = self.line_len();
    }

    pub fn move_top(&mut self) {
        self.cursor.0 = 0;
        self.cursor.1 = 0;
    }

    pub fn move_bottom(&mut self) {
        self.cursor.0 = self.line_len_idx(self.lines_len() - 1);
        self.cursor.1 = self.lines_len() - 1;
    }

    pub fn new_char(&mut self, c: char) {
        self.buffer.borrow_mut().insert_char(self.cursor, c);
        self.cursor.0 += 1;
    }

    pub fn new_line(&mut self) {
        let right = self.buffer.borrow_mut().split_off(self.cursor);

        self.cursor.0 = 0;
        self.cursor.1 += 1;

        self.buffer
            .borrow_mut()
            .insert_line((0, self.cursor.1), &right);
    }

    pub fn tab(&mut self) {
        self.buffer.borrow_mut().insert_str((0, self.cursor.1), TAB);
        self.cursor.0 += TAB.len();
    }

    pub fn toggle_x_mode(&mut self, mode: bool) {
        self.x_mode = mode;
    }

    pub fn backspace(&mut self) {
        if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
            self.buffer.borrow_mut().remove_char(self.cursor);
        } else if self.cursor.1 > 0 {
            let deleted = self.buffer.borrow_mut().delete_line(self.cursor);

            self.cursor.1 -= 1;
            self.cursor.0 = self.line_len();
            self.buffer.borrow_mut().push_str(self.cursor, &deleted);
        }
    }

    pub fn delete(&mut self) {
        if self.cursor.0 < self.line_len() {
            self.buffer.borrow_mut().remove_char(self.cursor);
        } else if self.lines_len() - 1 > self.cursor.1 {
            let deleted = self
                .buffer
                .borrow_mut()
                .delete_line((self.cursor.0, self.cursor.1 + 1));
            self.buffer.borrow_mut().push_str(self.cursor, &deleted);
        }
    }

    pub fn kill(&mut self) -> Option<String> {
        if self.cursor.0 < self.line_len() {
            let removed = self.buffer.borrow_mut().split_off(self.cursor);
            Some(removed)
        } else if self.lines_len() - 1 > self.cursor.1 {
            let removed = self
                .buffer
                .borrow_mut()
                .delete_line((self.cursor.0, self.cursor.1 + 1));
            self.buffer.borrow_mut().push_str(self.cursor, &removed);
            Some("\n".to_owned())
        } else {
            None
        }
    }

    pub fn paste(&mut self, s: &str) {
        self.buffer.borrow_mut().insert_str(self.cursor, s);
        self.cursor.0 += s.len();
    }

    pub fn is_x_mode(&self) -> bool {
        self.x_mode
    }

    pub fn split(&mut self, direction: Direction) {
        let additional_border_flag = if direction == Direction::Horizontal {
            Borders::RIGHT
        } else {
            Borders::BOTTOM
        };

        let mut f0 = Frame::inherit(self, additional_border_flag);
        let f1 = Frame::inherit(self, Borders::NONE);

        if self.focus {
            self.focus = false;
            f0.focus = true;
        }

        self.inner_frames = Some((Rc::new(RefCell::new(f0)), Rc::new(RefCell::new(f1))));
        self.split_direction = direction;
    }

    pub fn has_focus(&self) -> bool {
        if let Some((ref f0, ref f1)) = self.inner_frames {
            f0.borrow().has_focus() || f1.borrow().has_focus()
        } else {
            self.focus
        }
    }

    pub fn has_inner_frames(&self) -> bool {
        self.inner_frames.is_some()
    }

    pub fn clone_buffer(&self) -> Rc<RefCell<Buffer>> {
        self.buffer.clone()
    }

    pub fn clone_inner_frames(&self) -> (Rc<RefCell<Frame>>, Rc<RefCell<Frame>>) {
        self.inner_frames
            .as_ref()
            .map(|(f0, f1)| (f0.clone(), f1.clone()))
            .unwrap()
    }

    pub fn set_focus(&mut self) {
        if let Some((ref mut f0, _)) = self.inner_frames {
            f0.borrow_mut().set_focus();
        } else {
            self.focus = true;

            if self.cursor.1 > self.lines_len() - 1 {
                self.cursor.1 = self.lines_len() - 1;
            }

            if self.cursor.0 > self.line_len() {
                self.cursor.0 = self.line_len();
            }
        }
    }

    pub fn move_focus(&mut self) -> bool {
        if let Some((ref mut f0, ref mut f1)) = self.inner_frames {
            if f0.borrow().has_focus() {
                let moved = f0.borrow_mut().move_focus();

                if !moved {
                    f0.borrow_mut().clear_focus();
                    f1.borrow_mut().set_focus();
                }

                true
            } else {
                f1.borrow_mut().move_focus()
            }
        } else {
            false
        }
    }

    pub fn clear_focus(&mut self) {
        self.focus = false;

        if let Some((ref mut f0, ref mut f1)) = self.inner_frames {
            f0.borrow_mut().clear_focus();
            f1.borrow_mut().clear_focus();
        }
    }

    pub fn focus_child_frame(&self) -> Rc<RefCell<Frame>> {
        if let Some((ref f0, ref f1)) = self.inner_frames {
            if f0.borrow().has_focus() {
                if f0.borrow().has_inner_frames() {
                    f0.borrow().focus_child_frame()
                } else {
                    f0.clone()
                }
            } else {
                if f1.borrow().has_inner_frames() {
                    f1.borrow().focus_child_frame()
                } else {
                    f1.clone()
                }
            }
        } else {
            unreachable!();
        }
    }

    pub fn clean_removed_frame(&mut self) {
        if !self.has_inner_frames() {
            return;
        }

        let (f0, f1) = self.clone_inner_frames();
        let f0_is_shown = f0.borrow().is_shown();
        let f1_is_shown = f1.borrow().is_shown();

        match (f0_is_shown, f1_is_shown) {
            (true, true) => {
                f0.borrow_mut().clean_removed_frame();
                f1.borrow_mut().clean_removed_frame();
                return;
            }
            (true, false) => {
                if f0.borrow().has_inner_frames() {
                    self.inner_frames = Some(f0.borrow().clone_inner_frames());
                    self.split_direction = f0.borrow().split_direction.clone();
                } else {
                    self.inner_frames = None;
                    self.buffer = f0.borrow().clone_buffer();
                }
            }
            (false, true) => {
                if f1.borrow().has_inner_frames() {
                    self.inner_frames = Some(f1.borrow().clone_inner_frames());
                    self.split_direction = f1.borrow().split_direction.clone();
                } else {
                    self.inner_frames = None;
                    self.buffer = f1.borrow().clone_buffer();
                }
            }
            _ => {
                unreachable!();
            }
        }

        self.set_focus();
    }

    fn adjust_offset(&mut self, rect: &Rect) {
        if self.offset.0 > self.cursor.0 {
            self.offset.0 = self.cursor.0;
        } else {
            if self.border_flag.intersects(Borders::RIGHT) {
                if rect.width < 2 {
                    self.offset.0 = self.cursor.0;
                } else if self.cursor.0 > self.offset.0 + rect.width as usize - 2 {
                    self.offset.0 = self.cursor.0 + 2 - rect.width as usize;
                }
            } else {
                if rect.width < 1 {
                    self.offset.0 = self.cursor.0;
                } else if self.cursor.0 > self.offset.0 + rect.width as usize - 1 {
                    self.offset.0 = self.cursor.0 + 1 - rect.width as usize;
                }
            }
        }

        if self.offset.1 > self.cursor.1 {
            self.offset.1 = self.cursor.1;
        } else {
            if self.border_flag.intersects(Borders::BOTTOM) {
                if rect.height < 2 {
                    self.offset.1 = self.cursor.1;
                } else if self.cursor.1 > self.offset.1 + rect.height as usize - 2 {
                    self.offset.1 = self.cursor.1 + 2 - rect.height as usize;
                }
            } else {
                if rect.height < 1 {
                    self.offset.1 = self.cursor.1;
                } else if self.cursor.1 > self.offset.1 + rect.height as usize - 1 {
                    self.offset.1 = self.cursor.1 + 1 - rect.height as usize;
                }
            }
        }
    }

    pub fn render(
        &mut self,
        r: Rect,
    ) -> Vec<(Rect, Rc<RefCell<Buffer>>, Borders, Offset, Option<Cursor>)> {
        self.adjust_offset(&r);

        if let Some((ref f0, ref f1)) = self.inner_frames {
            let chunks = Layout::default()
                .direction(self.split_direction.clone())
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .margin(0)
                .split(r);

            let mut r0 = f0.borrow_mut().render(chunks[0]);
            let r1 = f1.borrow_mut().render(chunks[1]);

            r0.extend(r1);
            r0
        } else {
            let cursor = if self.focus {
                Some((
                    self.cursor.0 + r.x as usize - self.offset.0,
                    self.cursor.1 + r.y as usize - self.offset.1,
                ))
            } else {
                None
            };

            vec![(
                r,
                self.buffer.clone(),
                self.border_flag,
                self.offset,
                cursor,
            )]
        }
    }

    pub fn replace_buffer(&mut self, new_buffer: Rc<RefCell<Buffer>>) -> Rc<RefCell<Buffer>> {
        let old_buffer = self.buffer.clone();
        self.buffer = new_buffer;
        self.cursor = (0, 0);
        old_buffer
    }

    pub fn release_buffer(&mut self) -> Rc<RefCell<Buffer>> {
        self.show = false;
        self.focus = false;
        self.buffer.clone()
    }

    pub fn is_shown(&self) -> bool {
        self.show
    }
}
