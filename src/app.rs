use std::collections::HashMap;

#[derive(Default)]
pub struct App {
    pub state: State,

    pub diffs: HashMap<String, String>,
}

#[derive(Default)]
pub enum State {
    /// initializing gai:
    /// checks for existing repo
    /// does a diff check
    /// and gathers the data
    /// for the user to send
    #[default]
    Warmup,

    /// state where gai is sending
    /// a request or waiting to
    /// receive the response.
    /// This is usually one continous
    /// moment.
    Pending(PendingType),

    /// state where the user can
    /// either: see what to send
    /// to the AI provider
    /// or what the AI provider has
    /// sent back
    Running,
}

pub enum PendingType {
    Sending,
    Receiving,
}

impl App {
    pub fn load_diffs(&mut self, files: HashMap<String, String>) {
        self.diffs = files.to_owned();
    }
}
