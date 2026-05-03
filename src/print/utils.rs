use std::process;

/// execute tput with the given argument and parse
/// the output as a u16.
///
/// The arg should be "cols" or "lines"
/// ripped from crossterm-rs
fn tput_value(arg: &str) -> Option<u16> {
    let output = process::Command::new("tput")
        .arg(arg)
        .output()
        .ok()?;

    let value = output
        .stdout
        .into_iter()
        .filter_map(|b| char::from(b).to_digit(10))
        .fold(0, |v, n| v * 10 + n as u16);

    if value > 0 { Some(value) } else { None }
}

/// Returns the size of the screen as determined by tput.
///
/// This alternate way of computing the size is useful
/// when in a subshell.
/// ripped from crossterm-rs
pub(super) fn tput_size() -> Option<(u16, u16)> {
    match (tput_value("cols"), tput_value("lines")) {
        (Some(w), Some(h)) => Some((w, h)),
        _ => None,
    }
}
