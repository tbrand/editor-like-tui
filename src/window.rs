use crate::buffer::Buffer;
use crate::frame::Frame;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Window {
    main_frame: Rc<RefCell<Frame>>,
    detached_buffer: Vec<Rc<RefCell<Buffer>>>,
    yank: Option<String>,
}

impl Window {
    pub fn new() -> Self {
        let buffer = Rc::new(RefCell::new(Buffer::new()));
        let main_frame = Rc::new(RefCell::new(Frame::new(buffer.clone())));

        main_frame.borrow_mut().set_focus();

        Window {
            main_frame,
            detached_buffer: Vec::new(),
            yank: None,
        }
    }

    pub fn move_focus(&self) {
        if !self.main_frame.borrow_mut().move_focus() {
            self.main_frame.borrow_mut().clear_focus();
            self.main_frame.borrow_mut().set_focus();
        }
    }

    pub fn kill(&mut self) {
        let frame = self.focus_frame();
        self.yank = frame.borrow_mut().kill();
    }

    pub fn paste(&self) {
        let frame = self.focus_frame();
        if let Some(s) = &self.yank {
            frame.borrow_mut().paste(s);
        }
    }

    pub fn remove_focus_frame(&mut self) {
        let frame = self.focus_frame();
        if !frame.borrow().is_main_frame() {
            let buffer = frame.borrow_mut().release_buffer();
            self.main_frame().borrow_mut().clean_removed_frame();
            self.detached_buffer.push(buffer);
        }
    }

    pub fn main_frame(&self) -> Rc<RefCell<Frame>> {
        self.main_frame.clone()
    }

    pub fn focus_frame(&self) -> Rc<RefCell<Frame>> {
        if self.main_frame.borrow().has_inner_frames() {
            self.main_frame.borrow().focus_child_frame()
        } else {
            self.main_frame.clone()
        }
    }

    #[allow(unused)]
    pub fn detached_buffer(&self) -> Vec<Rc<RefCell<Buffer>>> {
        self.detached_buffer.clone()
    }
}
