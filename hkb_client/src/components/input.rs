use crossterm::event::{Event, KeyCode};
use hkb_core::logger::debug;
use ratatui::{
    prelude::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{app_state, events, focus::Focusable};

use super::StatefulComponent;

pub struct InputState {
    pub buffer: String,
    focused: bool,
    cursor_offset: u16,
    visible_buffer_offset: usize,
    last_render_width: u16,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            focused: false,
            last_render_width: 0,
            cursor_offset: 0,
            visible_buffer_offset: 0,
            //buffer: String::with_capacity(512),
            buffer: String::from(
                "Testing this if this is a good idea lorem ipsum bopsum best test best mest sest Testing this if this is a good idea lorem ipsum bopsum best test best mest sest",
            ),
        }
    }
}

impl Focusable for InputState {
    fn focus(&mut self) {
        self.focused = true;
    }

    fn blur(&mut self) {
        self.focused = false;
    }
}

pub struct Input<'a> {
    title: &'a str,
}

impl<'a> Input<'a> {
    pub fn new(title: &'a str) -> Self {
        Self { title }
    }
}

impl<'a> Input<'a> {
    fn trimmed_buffer(&self, state: &'a InputState, area: &Rect) -> &str {
        let mut output = &state.buffer[..];
        if state.visible_buffer_offset >= (area.width as usize) {
            output = &state.buffer[state.visible_buffer_offset - (area.width as usize)..];
        }

        output
    }

    fn get_max_right_cursor_pos(&self, state: &InputState) -> u16 {
        std::cmp::min(state.buffer.len(), (state.last_render_width - 1) as usize) as u16
    }

    fn go_left(&self, state: &mut InputState) {
        if state.cursor_offset != 0 {
            state.cursor_offset -= 1;
        } else if state.visible_buffer_offset != 0 {
            state.visible_buffer_offset -= 1;
        }
    }

    fn go_right(&self, state: &mut InputState) {
        if state.cursor_offset < self.get_max_right_cursor_pos(state) {
            state.cursor_offset += 1;
        } else if state.visible_buffer_offset < state.buffer.len() {
            state.visible_buffer_offset += 1;
        }
    }

    fn go_far_left(&self, state: &mut InputState) {
        state.visible_buffer_offset = 0;
        state.cursor_offset = 0;
    }

    fn go_far_right(&self, state: &mut InputState) {
        state.visible_buffer_offset = state.buffer.len();
        state.cursor_offset = self.get_max_right_cursor_pos(state);
    }

    fn update_on_not_editing(&self, state: &mut InputState) {
        events::consume_key_event!(
            KeyCode::Char(c) if c == 'h' => {
                self.go_left(state);
            }

            KeyCode::Char(c) if c == 'l' => {
                self.go_right(state);
            }

            KeyCode::Char(c) if c == '$' => {
                self.go_far_right(state);
            }

            KeyCode::Char(c) if c == '^' => {
                self.go_far_left(state);
            }
        );
    }

    fn update(&self, state: &mut InputState) {
        if !app_state::is_editing() {
            self.update_on_not_editing(state);

            return;
        }

        events::consume_key_event!(
            KeyCode::Char(c) => {
                state.buffer.push(c);
                state.visible_buffer_offset = state.buffer.len();

                self.go_right(state);
            }
            KeyCode::Left => {
                self.go_left(state);
            }
            KeyCode::Right => {
                self.go_right(state);
            }
            KeyCode::Backspace => {
                if state.buffer.len() > 0 {
                    state.buffer = (&state.buffer[0..state.buffer.len() - 1]).to_owned();
                    state.visible_buffer_offset = state.buffer.len();

                    if state.cursor_offset != 0 {
                        state.cursor_offset -= 1;
                    }
                }
            }
        );
    }
}

impl<'a> StatefulComponent for Input<'a> {
    type State = InputState;

    fn render(&mut self, frame: &mut Frame, state: &mut InputState, area: Rect) {
        let block = Block::default().borders(Borders::ALL);
        let block_area = block.inner(area);

        if state.focused {
            self.update(state);

            frame.set_cursor(state.cursor_offset + block_area.x, block_area.y);
        }

        state.last_render_width = block_area.width;
        frame.render_widget(
            Paragraph::new(self.trimmed_buffer(state, &area))
                .block(block.title(self.title.as_ref())),
            area,
        );
    }
}
