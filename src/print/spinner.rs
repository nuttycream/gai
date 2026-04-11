// this is a somewhat modified/simplified
// version of terminal-spinners-rs
// minuys the symbology,
// the spinner should span across the width
// of the terminal
// alot of the code here has been reused from the
// library

use crossterm::{
    cursor, execute, queue,
    style::{Print, ResetColor, SetForegroundColor},
    terminal,
};
use std::{
    borrow::Cow,
    io::{Write, stdout},
    sync::mpsc::{Receiver, Sender, TryRecvError, channel},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use super::renderer::Renderer;

type Str = Cow<'static, str>;

const COL: usize = 30;
const RATE: Duration = Duration::from_millis(300);
const DOTS: [&str; 4] = ["", ".", "..", "..."];

#[derive(Copy, Clone)]
enum StopType {
    Done,
    Error,
}

// Commands send through the mpsc channels to notify the render thread of certain events.
enum SpinnerCommand {
    // Commands that stop the spinner.
    Stop(Option<StopType>),
    StopAndClear,
}

// The internal representation of a spinner.
//
// Holds all the data needed to actually render the spinner on a render thread.
struct Spinner {
    text: Str,
    rx: Receiver<SpinnerCommand>,
}

/// A builder for creating a terminal spinner.
#[derive(Clone, Default)]
pub struct SpinnerBuilder {
    text: Option<Str>,
}

impl SpinnerBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// The text to show after the spinner animation.
    pub fn text(
        mut self,
        text: impl Into<Cow<'static, str>>,
    ) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Starts the spinner and renders it on a separate thread.
    ///
    /// # Returns
    ///
    /// A `SpinnerHandle`, allowing for further control of the spinner after it gets rendered.
    ///
    /// # Panics
    ///
    /// If no text and spinner have been set.
    pub fn start(
        self,
        renderer: &Renderer,
    ) -> SpinnerHandle {
        assert!(self.text.is_some());

        let (tx, rx) = channel();
        let spinner = Spinner {
            text: self.text.unwrap(),
            rx,
        };

        spinner.start(tx, renderer)
    }
}

impl Spinner {
    fn start(
        self,
        tx: Sender<SpinnerCommand>,
        renderer: &Renderer,
    ) -> SpinnerHandle {
        let colors = renderer
            .style
            .allow_colors;

        let primary = renderer
            .style
            .primary;

        let highlight = renderer
            .style
            .highlight;

        let handle = thread::spawn(move || {
            let mut out = stdout();

            let start = Instant::now();
            let mut tick: usize = 0;

            execute!(out, cursor::Hide).ok();

            let mut stop_msg = String::new();
            loop {
                let mut should_clear_line = false;
                let mut should_stop_cycle_loop = false;

                match self.rx.try_recv() {
                    Ok(cmd) => match cmd {
                        SpinnerCommand::Stop(s) => {
                            should_stop_cycle_loop = true;
                            if let Some(s) = s {
                                match s {
                                    StopType::Done => {
                                        stop_msg = "done".to_string()
                                    }
                                    StopType::Error => {
                                        stop_msg = "error".to_string()
                                    }
                                }
                            } else {
                                stop_msg = "stopping".to_string()
                            }
                        }
                        SpinnerCommand::StopAndClear => {
                            should_clear_line = true;
                            should_stop_cycle_loop = true;
                        }
                    },
                    Err(TryRecvError::Disconnected) => {
                        should_stop_cycle_loop = true
                    }
                    _ => {} // We do not care about other types of errors.
                }

                // Continue with the animation.
                // 1. Delete current line.
                queue!(
                    out,
                    terminal::Clear(terminal::ClearType::CurrentLine)
                )
                .unwrap();

                queue!(out, cursor::MoveToColumn(0)).unwrap();

                let dots = DOTS[tick % DOTS.len()];
                let elapsed = start
                    .elapsed()
                    .as_secs_f64();

                let mut left = format!("{}{}", self.text, dots);
                let mut right = format!("({elapsed:.0}s)");
                let pad = COL.saturating_sub(left.len());

                // 2. Check if we can early-stop.
                if should_stop_cycle_loop {
                    if !should_clear_line {
                        left = format!("{}...", self.text);
                        right =
                            format!("{} ({elapsed:.1}s)", stop_msg);

                        if colors {
                            queue!(
                                out,
                                SetForegroundColor(primary),
                                Print(&left),
                                Print(" ".repeat(pad)),
                                SetForegroundColor(highlight),
                                Print(&right),
                                Print("\r\n"),
                                ResetColor,
                            )
                            .ok();
                        } else {
                            queue!(
                                out,
                                Print(&left),
                                Print(" ".repeat(pad)),
                                Print(&right),
                                Print("\r\n"),
                            )
                            .ok();
                        }
                    }

                    out.flush().unwrap();

                    execute!(out, cursor::Show).ok();

                    break; // Breaks out of the animation loop
                }

                if colors {
                    queue!(
                        out,
                        SetForegroundColor(primary),
                        Print(&left),
                        Print(" ".repeat(pad)),
                        SetForegroundColor(highlight),
                        Print(&right),
                        ResetColor,
                    )
                    .ok();
                } else {
                    queue!(
                        out,
                        Print(&left),
                        Print(" ".repeat(pad)),
                        Print(&right),
                    )
                    .ok();
                }

                out.flush().ok();

                tick += 1;
                thread::sleep(RATE);
            }
        });

        SpinnerHandle { handle, tx }
    }
}

/// A handle to a running spinner.
///
/// Can be used to send commands to the render thread.
pub struct SpinnerHandle {
    handle: JoinHandle<()>,
    tx: Sender<SpinnerCommand>,
}

impl SpinnerHandle {
    /// Stops the spinner and renders a success symbol.
    pub fn done(self) {
        self.tx
            .send(SpinnerCommand::Stop(Some(StopType::Done)))
            .unwrap();

        self.handle
            .join()
            .unwrap();
    }

    /// Stops the spinner and renders an error symbol.
    /// FIXME: this is broken, need to handle this
    /// properly when we implement better error handling
    pub fn error(self) {
        self.tx
            .send(SpinnerCommand::Stop(Some(StopType::Error)))
            .unwrap();

        self.handle
            .join()
            .unwrap();
    }

    /// Stops the spinner.
    pub fn stop(self) {
        self.tx
            .send(SpinnerCommand::Stop(None))
            .unwrap();

        self.handle
            .join()
            .unwrap();
    }

    /// Stops the spinner and clears the line it was printed on.
    pub fn stop_and_clear(self) {
        self.tx
            .send(SpinnerCommand::StopAndClear)
            .unwrap();

        self.handle
            .join()
            .unwrap();
    }
}
