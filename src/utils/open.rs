// modified and simplified version of
// https://github.com/twilligon/edit/tree
// without builder for tempfile, we just create
// and delete a temp file in dir

use std::{
    env,
    ffi::OsStr,
    fs,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

static ENV_VARS: &[&str] = &["VISUAL", "EDITOR"];

// TODO: should we hardcode full paths as well in case $PATH is borked?
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
#[rustfmt::skip]
static HARDCODED_NAMES: &[&str] = &[
    // CLI editors
    "sensible-editor", "nano", "pico", "vim", "nvim", "vi", "emacs",
    // GUI editors
    "code", "atom", "subl", "gedit", "gvim",
    // Generic "file openers"
    "xdg-open", "gnome-open", "kde-open",
];

#[cfg(target_os = "macos")]
#[rustfmt::skip]
static HARDCODED_NAMES: &[&str] = &[
    // CLI editors
    "nano", "pico", "vim", "nvim", "vi", "emacs",
    // open has a special flag to open in the default text editor
    // (this really should come before the CLI editors, but in order
    // not to break compatibility, we still prefer CLI over GUI)
    "open -Wt",
    // GUI editors
    "code -w", "atom -w", "subl -w", "gvim", "mate",
    // Generic "file openers"
    "open -a TextEdit",
    "open -a TextMate",
    // TODO: "open -f" reads input from standard input and opens with
    // TextEdit. if this flag were used we could skip the tempfile
    "open",
];

#[cfg(target_os = "windows")]
#[rustfmt::skip]
static HARDCODED_NAMES: &[&str] = &[
    // GUI editors
    "code.cmd -n -w", "atom.exe -w", "subl.exe -w",
    // notepad++ does not block for input
    // Installed by default
    "notepad.exe",
    // Generic "file openers"
    "cmd.exe /C start",
];

/// Open the contents of a string or buffer in the [default editor].
///
/// This function saves its input to a temporary file and then opens the default editor to it.
/// It waits for the editor to return, re-reads the (possibly changed/edited) temporary file, and
/// then deletes it.
///
/// # Arguments
///
/// `text` is written to the temporary file before invoking the editor. (The editor opens with
/// the contents of `text` already in the file).
///
/// # Returns
///
/// If successful, returns the edited string.
/// Any errors related to spawning the editor process will also be passed through.
pub fn edit(text: &str) -> anyhow::Result<String> {
    let path = env::temp_dir().join("edit.tmp");
    fs::write(&path, text)?;

    let (editor, args) = get_editor_args()?;
    let status = Command::new(&editor)
        .args(&args)
        .arg(&path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?
        .status;

    if !status.success() {
        let _ = fs::remove_file(&path);
        anyhow::bail!(
            "editor '{}' exited with: {}",
            editor.to_string_lossy(),
            status
        );
    }

    let edited = fs::read_to_string(&path)?;
    // instead of tempdir
    fs::remove_file(&path)?;

    Ok(edited)
}

fn _get_editor() -> anyhow::Result<PathBuf> {
    get_editor_args().map(|(path, _)| path)
}

fn get_full_editor_path<T: AsRef<OsStr> + AsRef<Path>>(
    binary_name: T
) -> anyhow::Result<PathBuf> {
    if let Some(paths) = env::var_os("PATH") {
        for dir in env::split_paths(&paths) {
            if dir
                .join(&binary_name)
                .is_file()
            {
                return Ok(dir.join(&binary_name));
            }
        }
    }

    Err(Error::from(ErrorKind::NotFound).into())
}

fn string_to_cmd(s: String) -> (PathBuf, Vec<String>) {
    let mut args = s.split_ascii_whitespace();
    (
        args.next()
            .unwrap()
            .into(),
        args.map(String::from)
            .collect(),
    )
}

fn get_full_editor_cmd(
    s: String
) -> anyhow::Result<(PathBuf, Vec<String>)> {
    let (path, args) = string_to_cmd(s);
    match get_full_editor_path(&path) {
        Ok(result) => Ok((result, args)),
        Err(_) if path.exists() => Ok((path, args)),
        Err(_) => Err(Error::from(ErrorKind::NotFound).into()),
    }
}

fn get_editor_args() -> anyhow::Result<(PathBuf, Vec<String>)> {
    ENV_VARS
        .iter()
        .filter_map(env::var_os)
        .filter(|v| !v.is_empty())
        .filter_map(|v| v.into_string().ok())
        .filter_map(|s| get_full_editor_cmd(s).ok())
        .next()
        .or_else(|| {
            HARDCODED_NAMES
                .iter()
                .map(|s| s.to_string())
                .filter_map(|s| get_full_editor_cmd(s).ok())
                .next()
        })
        .ok_or_else(|| Error::from(ErrorKind::NotFound).into())
}
