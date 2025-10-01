use ratatui::Frame;

use crate::response::Response;

pub fn draw_splash(frame: &mut Frame) {}

pub fn draw_pending(frame: &mut Frame) {}

pub fn draw_diffview(frame: &mut Frame) {}

pub fn draw_opsview(frame: &mut Frame, ops: Option<&[Response]>) {}
