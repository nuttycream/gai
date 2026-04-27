/// a util for displaying an columned menu
/// originally a style for dialoguer-rs
///
/// admittedly kinda jank. and some hacky ways
/// were implemented for clearing lines
/// we need to handle wrapping lines + resized
/// terminals
use std::io::{Write, stdin};

use anstream::stdout;
use owo_colors::OwoColorize;

#[derive(Debug)]
pub(crate) struct Menu<T> {
    prompt: String,
    items: Vec<MenuItem<T>>,
}

#[derive(Debug, Clone)]
struct MenuItem<T> {
    description: String,
    keybind: char,
    val: Option<T>,
}

impl<T: Clone> Menu<T> {
    /// create a new menu, takes in a slice tuple
    /// where the first is generic enum type
    /// the char or bind is the
    /// second and the description is the third
    /// note, ? are reserved for help
    pub fn new(
        prompt: &str,
        opts: &[(T, char, &str)],
    ) -> Self {
        let mut items = Vec::new();
        for (val, bind, desc) in opts {
            if *bind != '?' {
                items.push(MenuItem {
                    val: Some(val.to_owned()),
                    description: desc.to_string(),
                    keybind: *bind,
                });
            }
        }

        items.push(MenuItem {
            description: "print this help".to_owned(),
            keybind: '?',
            val: None,
        });

        Self {
            items,
            prompt: prompt.to_owned(),
        }
    }

    /// event handling, this is a generic
    /// function that should and would be handled
    /// by higher level functions
    /// can take in a max of 9 options
    /// prompt/label is rendered columned if compact
    pub fn render(self) -> anyhow::Result<T> {
        let mut out = stdout();

        let opts = self
            .items
            .iter()
            .map(|i| {
                i.keybind
                    .to_string()
            })
            .collect::<Vec<String>>();

        let form = format!("[{}]: ", opts.join(","));

        write!(
            out,
            "{} {}",
            &self
                .prompt
                .blue()
                .bold(),
            &form.blue(),
        )?;

        out.flush()?;

        let mut input = String::new();

        stdin().read_line(&mut input)?;

        let Some(ch) = input
            .trim()
            .chars()
            .next()
        else {
            return self.render();
        };

        let mut stdout = stdout();

        if let Some(item) = self
            .items
            .iter()
            .find(|i| i.keybind == ch)
        {
            match &item.val {
                Some(v) => return Ok(v.to_owned()),
                None if item.keybind == '?' => {
                    self.help(&mut stdout)?;
                    return self.render();
                }
                None => {
                    anyhow::bail!(
                        "no matching val? {}",
                        item.keybind
                    );
                }
            }
        } else {
            writeln!(stdout, "{ch} is not a valid option, see ?")?;

            stdout.flush()?;

            return self.render();
        }
    }

    fn help(
        &self,
        out: &mut impl Write,
    ) -> anyhow::Result<()> {
        for item in self.items.iter() {
            writeln!(
                out,
                "{} - {}",
                item.keybind
                    .yellow()
                    .bold(),
                item.description
                    .yellow()
            )?;
        }

        Ok(())
    }
}
