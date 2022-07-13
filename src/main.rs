#[macro_use]
extern crate log;

mod buffer;
mod frame;
mod window;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use frame::Direction;
use log::LevelFilter;
use simplelog::{Config as LogConfig, WriteLogger};
use std::fs::File;
use std::{error::Error, io};
use tui::backend::{Backend, CrosstermBackend};
use tui::widgets::{Block, BorderType, Paragraph};
use tui::{Frame as TerminalFrame, Terminal};
use window::Window;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    hook_panic();

    WriteLogger::init(
        LevelFilter::Debug,
        LogConfig::default(),
        File::create("editor.log").unwrap(),
    )?;

    debug!("hello");

    let mut terminal = init_terminal()?;
    let window = Window::new();

    if let Err(e) = run_app(&mut terminal, window).await {
        eprintln!("{:?}", e);
    }

    reset_terminal()?;

    Ok(())
}

fn hook_panic() {
    let hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic| {
        reset_terminal().unwrap();
        hook(panic);
    }));
}

fn init_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, Box<dyn Error>> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();

    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}

fn reset_terminal() -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;

    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut window: Window) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &window))?;

        let frame = window.focus_frame();

        if let Event::Key(key) = event::read()? {
            if frame.borrow().is_x_mode() {
                match key.modifiers {
                    KeyModifiers::CONTROL => match key.code {
                        KeyCode::Char('c') => return Ok(()),
                        KeyCode::Char('g') => {}
                        _ => {}
                    },
                    KeyModifiers::SHIFT => match key.code {
                        _ => {}
                    },
                    KeyModifiers::ALT => match key.code {
                        _ => {}
                    },
                    KeyModifiers::NONE => match key.code {
                        KeyCode::Char('[') => frame.borrow_mut().move_top(),
                        KeyCode::Char(']') => frame.borrow_mut().move_bottom(),
                        KeyCode::Char('2') => frame.borrow_mut().split(Direction::Vertical),
                        KeyCode::Char('3') => frame.borrow_mut().split(Direction::Horizontal),
                        KeyCode::Char('o') => window.move_focus(),
                        KeyCode::Char('b') => {
                            // TODO
                            use buffer::Buffer;
                            use std::cell::RefCell;
                            use std::rc::Rc;
                            frame
                                .borrow_mut()
                                .replace_buffer(Rc::new(RefCell::new(Buffer::new())));
                        }
                        KeyCode::Char('0') => window.remove_focus_frame(),
                        _ => {}
                    },
                    _ => {}
                }

                frame.borrow_mut().toggle_x_mode(false);
            } else {
                match key.modifiers {
                    KeyModifiers::CONTROL => match key.code {
                        KeyCode::Char('b') => frame.borrow_mut().move_left(),
                        KeyCode::Char('f') => frame.borrow_mut().move_right(),
                        KeyCode::Char('p') => frame.borrow_mut().move_up(),
                        KeyCode::Char('n') => frame.borrow_mut().move_down(),
                        KeyCode::Char('a') => frame.borrow_mut().move_front(),
                        KeyCode::Char('e') => frame.borrow_mut().move_end(),
                        KeyCode::Char('j') => frame.borrow_mut().new_line(),
                        KeyCode::Char('x') => frame.borrow_mut().toggle_x_mode(true),
                        KeyCode::Char('h') => frame.borrow_mut().backspace(),
                        KeyCode::Char('d') => frame.borrow_mut().delete(),
                        KeyCode::Char('k') => window.kill(),
                        KeyCode::Char('y') => window.paste(),
                        _ => {}
                    },
                    KeyModifiers::ALT => match key.code {
                        _ => {}
                    },
                    KeyModifiers::SHIFT => match key.code {
                        KeyCode::Char(c) => frame.borrow_mut().new_char(c),
                        _ => {}
                    },
                    KeyModifiers::NONE => match key.code {
                        KeyCode::Char(c) => frame.borrow_mut().new_char(c),
                        KeyCode::Left => frame.borrow_mut().move_left(),
                        KeyCode::Right => frame.borrow_mut().move_right(),
                        KeyCode::Up => frame.borrow_mut().move_up(),
                        KeyCode::Down => frame.borrow_mut().move_down(),
                        KeyCode::Home => frame.borrow_mut().move_front(),
                        KeyCode::End => frame.borrow_mut().move_end(),
                        KeyCode::Enter => frame.borrow_mut().new_line(),
                        KeyCode::Tab => frame.borrow_mut().tab(),
                        KeyCode::Backspace => frame.borrow_mut().backspace(),
                        KeyCode::Delete => frame.borrow_mut().delete(),
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }
}

fn ui<B: Backend>(f: &mut TerminalFrame<B>, window: &Window) {
    let main_frame = window.main_frame();
    let rendered = main_frame.borrow_mut().render(f.size());

    for (rect, buffer, borders, (offset_x, offset_y), cursor) in rendered {
        f.render_widget(
            Paragraph::new(buffer.borrow().text_styled())
                .block(
                    Block::default()
                        .borders(borders)
                        .border_type(BorderType::Thick),
                )
                .scroll((offset_y as u16, offset_x as u16)),
            rect,
        );

        if let Some(cursor) = cursor {
            f.set_cursor(cursor.0 as u16, cursor.1 as u16);
        }
    }
}

#[cfg(test)]
mod monkey_test {
    use super::*;
    use rand::distributions::WeightedIndex;
    use rand::prelude::*;

    const MONKEY_COUNTER_LIMIT: u32 = 5000;

    #[derive(Clone, Copy, Debug)]
    enum Event {
        NewChar,
        NewLine,
        MoveLeft,
        MoveRight,
        MoveUp,
        MoveDown,
        MoveFront,
        MoveEnd,
        MoveTop,
        MoveBottom,
        Backspace,
        Delete,
        Kill,
        Paste,
        SplitHorizontal,
        SplitVertical,
        MoveFocus,
        NewBuffer,
        RemoveFrame,
    }

    fn create_events() -> (Vec<(Event, u32, u32)>, WeightedIndex<u32>) {
        let events = vec![
            (Event::NewChar, 80, 0),
            (Event::NewLine, 80, 0),
            (Event::MoveLeft, 20, 0),
            (Event::MoveRight, 20, 0),
            (Event::MoveUp, 20, 0),
            (Event::MoveDown, 20, 0),
            (Event::MoveFront, 10, 0),
            (Event::MoveEnd, 10, 0),
            (Event::MoveTop, 10, 0),
            (Event::MoveBottom, 10, 0),
            (Event::Backspace, 8, 0),
            (Event::Delete, 8, 0),
            (Event::Kill, 5, 0),
            (Event::Paste, 5, 0),
            (Event::SplitHorizontal, 5, 0),
            (Event::SplitVertical, 5, 0),
            (Event::MoveFocus, 8, 0),
            (Event::NewBuffer, 2, 0),
            (Event::RemoveFrame, 15, 0),
        ];

        (
            events.clone(),
            WeightedIndex::new(events.iter().map(|item| item.1)).unwrap(),
        )
    }

    #[tokio::test]
    async fn run() {
        WriteLogger::init(
            LevelFilter::Debug,
            LogConfig::default(),
            File::create("editor-monkey-test.log").unwrap(),
        )
        .unwrap();

        let res = tokio::spawn(async move {
            let mut terminal = init_terminal().unwrap();
            let mut window = Window::new();
            let mut counter = 0;
            let (mut events, weighted_index) = create_events();
            let mut rng = thread_rng();

            loop {
                terminal.draw(|f| ui(f, &window)).unwrap();

                let frame = window.focus_frame();
                let mut event = &mut events[weighted_index.sample(&mut rng)];

                match event.0 {
                    Event::NewChar => {
                        let c = rng.sample(rand::distributions::Alphanumeric) as char;
                        frame.borrow_mut().new_char(c);
                    }
                    Event::NewLine => frame.borrow_mut().new_line(),
                    Event::MoveLeft => frame.borrow_mut().move_left(),
                    Event::MoveRight => frame.borrow_mut().move_right(),
                    Event::MoveUp => frame.borrow_mut().move_up(),
                    Event::MoveDown => frame.borrow_mut().move_down(),
                    Event::MoveFront => frame.borrow_mut().move_front(),
                    Event::MoveEnd => frame.borrow_mut().move_end(),
                    Event::MoveTop => frame.borrow_mut().move_top(),
                    Event::MoveBottom => frame.borrow_mut().move_bottom(),
                    Event::Backspace => frame.borrow_mut().backspace(),
                    Event::Delete => frame.borrow_mut().delete(),
                    Event::Kill => window.kill(),
                    Event::Paste => window.paste(),
                    Event::SplitHorizontal => frame.borrow_mut().split(Direction::Horizontal),
                    Event::SplitVertical => frame.borrow_mut().split(Direction::Vertical),
                    Event::MoveFocus => window.move_focus(),
                    Event::NewBuffer => {
                        // TODO
                        use buffer::Buffer;
                        use std::cell::RefCell;
                        use std::rc::Rc;
                        frame
                            .borrow_mut()
                            .replace_buffer(Rc::new(RefCell::new(Buffer::new())));
                    }
                    Event::RemoveFrame => window.remove_focus_frame(),
                }

                event.2 += 1;
                counter += 1;

                if counter == MONKEY_COUNTER_LIMIT {
                    break;
                }
            }

            info!("Event Counts");
            for e in &events {
                info!("- {:?} : {}", e.0, e.2);
            }
        })
        .await;

        reset_terminal().unwrap();

        if let Err(_) = res {
            panic!();
        }
    }
}
