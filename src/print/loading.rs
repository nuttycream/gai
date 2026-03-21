use std::time::Duration;

/// util loading bar
pub struct Loading {
    interval: Duration,
}

impl Loading {
    /// create loading widget
    /// compact is a single line
    /// otherwise, a multiline loading spinner
    pub fn new(
        text: &str,
        compact: bool,
    ) -> anyhow::Result<Self> {
        let interval = Duration::default();
        Ok(Self { interval })
    }

    pub fn interval(
        mut self,
        interval: Duration,
    ) -> Self {
        self.interval = interval;
        self
    }

    pub fn set_text(
        &mut self,
        text: &str,
    ) {
    }

    pub fn start(&self) {}

    pub fn stop_clear(&self) {}

    pub fn stop_with_message(
        &self,
        text: &str,
    ) {
    }

    pub fn stop(&self) {}
}

pub const TICK_COMPACT: &[&str; 9] =
    &["⣼", "⣹", "⢻", "⠿", "⡟", "⣏", "⣧", "⣶", "⣿"];

/// made with ascii-motion.app
/// cat from https://www.messletters.com/en/text-art/
const TICK_LONG: &[&str] = &[
    concat!(
        "            .           \n",
        "                   .    \n",
        " ∧,,∧                   \n",
        "( ̳•·•)                  \n",
        "/   づ      .        .  \n",
        "─────────               \n",
    ),
    concat!(
        "            *           \n",
        "                   *    \n",
        " ∧,,∧      .            \n",
        "( ̳•·•)づ          .     \n",
        "/   づ      *        *  \n",
        "─────────               \n",
    ),
    concat!(
        "            ⟡           \n",
        "                   ⟡    \n",
        " ∧,,∧      *            \n",
        "( ̳•·•)            *     \n",
        "/   づ      ⟡        ⟡  \n",
        "─────────               \n",
    ),
    concat!(
        "            *           \n",
        "                   *    \n",
        " ∧,,∧      ⟡            \n",
        "( ̳•·•)づ          ⟡     \n",
        "/   づ      *        *  \n",
        "─────────               \n",
    ),
    concat!(
        "            .           \n",
        "                   .    \n",
        " ∧,,∧      *            \n",
        "( ̳•·•)            *     \n",
        "/   づ      .        .  \n",
        "─────────               \n",
    ),
    concat!(
        "            *           \n",
        "                   *    \n",
        " ∧,,∧  <AllDone!>       \n",
        "( ̳^·^)            *     \n",
        "/ u u       *        *  \n",
        "─────────               \n",
    ),
];
