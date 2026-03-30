// this is a somewhat modified/simplified
// version of terminal-spinners-rs
// minuys the symbology,
// the spinner should span across the width
// of the terminal
// alot of the code here has been reused from the
// library

use crossterm::{cursor, event, execute, queue, terminal};
use std::{
    borrow::Cow,
    fmt::Display,
    io::{Write, stdout},
    sync::mpsc::{Receiver, Sender, TryRecvError, channel},
    thread::{self, JoinHandle},
    time::Duration,
};

type Str = Cow<'static, str>;

const WAVE: &[u8] = b")(|";
const RATE: Duration = Duration::from_millis(100);

#[derive(Copy, Clone)]
enum StopType {
    Done,
    Error,
    Info,
}

impl Display for StopType {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        let symbol = match self {
            StopType::Done => "[ok]",
            StopType::Error => "[!!]",
            StopType::Info => "[i]",
        };

        write!(f, "{symbol}")
    }
}

// Commands send through the mpsc channels to notify the render thread of certain events.
enum SpinnerCommand {
    /// Changes the text of the spinner. The change is visible once the spinner gets redrawn.
    ChangeText(Cow<'static, str>),

    // Commands that stop the spinner.
    Stop(Option<StopType>),
    StopAndClear,
}

// The internal representation of a spinner.
//
// Holds all the data needed to actually render the spinner on a render thread.
struct Spinner {
    text: Str,
    prefix: Str,
    rx: Receiver<SpinnerCommand>,
}

/// A builder for creating a terminal spinner.
#[derive(Clone, Default)]
pub struct SpinnerBuilder {
    text: Option<Str>,
    prefix: Option<Str>,
}

impl<'a> SpinnerBuilder {
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

    /// The prefix to print before the actual spinning animation.
    ///
    /// # Note
    ///
    /// The prefix must not include newlines, as the library deletion does not account for those.
    pub fn prefix(
        mut self,
        prefix: impl Into<Cow<'static, str>>,
    ) -> Self {
        self.prefix = Some(prefix.into());
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
    pub fn start(self) -> SpinnerHandle {
        assert!(self.text.is_some());

        let (tx, rx) = channel();
        let spinner = Spinner {
            text: self.text.unwrap(),
            prefix: self
                .prefix
                .unwrap_or(Cow::Borrowed("")),
            rx,
        };
        spinner.start(tx)
    }
}

impl Spinner {
    fn start(
        mut self,
        tx: Sender<SpinnerCommand>,
    ) -> SpinnerHandle {
        let handle = thread::spawn(move || {
            let mut stdout = stdout();
            let mut symbol: Option<StopType> = None;

            // col idx of leading char
            let mut head: usize = 0;

            loop {
                while event::poll(RATE).unwrap_or(false) {
                    if let Ok(ev) = event::read() {
                        match ev {
                            event::Event::Resize(..) => {
                                // cheeky terminal resize handler
                                // redraw bar and reset anims
                                head = 0;
                            }
                            // LOL lidl C-c sig handler
                            event::Event::Key(key)
                                if key.code
                                    == event::KeyCode::Char('c')
                                    && key
                                        .modifiers
                                        .contains(
                                        event::KeyModifiers::CONTROL,
                                    ) =>
                            {
                                execute!(stdout, cursor::Show).ok();

                                terminal::disable_raw_mode().ok();

                                std::process::exit(1);
                            }
                            _ => {}
                        }
                    }
                }

                let mut should_clear_line = false;
                let mut should_stop_cycle_loop = false;

                match self.rx.try_recv() {
                    Ok(cmd) => match cmd {
                        SpinnerCommand::ChangeText(text) => {
                            self.text = text
                        }
                        SpinnerCommand::Stop(s) => {
                            should_stop_cycle_loop = true;
                            symbol = s;
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
                    stdout,
                    terminal::Clear(terminal::ClearType::CurrentLine)
                )
                .unwrap();

                queue!(stdout, cursor::MoveToColumn(0)).unwrap();

                let width = self.bar_width();
                let bar = ".".repeat(width as usize);

                // 2. Check if we can early-stop.
                if should_stop_cycle_loop {
                    if !should_clear_line {
                        if let Some(sym) = symbol {
                            writeln!(
                                stdout,
                                "{} {}[{}]{}",
                                self.text, self.prefix, bar, sym,
                            )
                            .unwrap();
                        } else {
                            writeln!(
                                stdout,
                                "{} {}[{}]",
                                self.text, self.prefix, bar,
                            )
                            .unwrap();
                        }
                    }

                    stdout
                        .flush()
                        .unwrap();

                    break; // Breaks out of the animation loop
                }

                // 3. Print the new line.
                let width = width as usize;
                let wave_len = WAVE.len();
                let mut cells: Vec<u8> = vec![b'.'; width];

                let pos = head % (width + wave_len);
                for i in 0..wave_len {
                    let col = pos.wrapping_sub(i);
                    if col < width {
                        cells[col] = WAVE[i];
                    }
                }

                head += 1;

                let bar = String::from_utf8(cells).unwrap();

                write!(
                    stdout,
                    "{} {}[{}][..]",
                    self.text, self.prefix, bar,
                )
                .unwrap();

                stdout
                    .flush()
                    .unwrap();
            }
        });

        SpinnerHandle { handle, tx }
    }

    /// returns the bar width as 25% of terminal columns.
    fn bar_width(&self) -> u16 {
        let cols = terminal::size()
            .map(|(w, _)| w)
            .unwrap_or(80);

        (cols / 4).max(1)
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
    pub fn error(self) {
        self.tx
            .send(SpinnerCommand::Stop(Some(StopType::Error)))
            .unwrap();

        self.handle
            .join()
            .unwrap();
    }

    /// Stops the spinner and renders an information symbol.
    pub fn info(self) {
        self.tx
            .send(SpinnerCommand::Stop(Some(StopType::Info)))
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

    /// Changes the text of the spinner.
    pub fn text(
        &self,
        text: impl Into<Cow<'static, str>>,
    ) {
        self.tx
            .send(SpinnerCommand::ChangeText(text.into()))
            .unwrap();
    }
}
