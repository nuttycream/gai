/// a util for displaying an columned menu
/// originally a style for dialoguer-rs
///
/// admittedly kinda jank. and some hacky ways
/// were implemented for clearing lines
/// we need to handle wrapping lines + resized
/// terminals
use std::io::{Write, stdin, stdout};

use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};

use super::renderer::Renderer;

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
    pub fn render(
        self,
        renderer: &Renderer,
    ) -> anyhow::Result<T> {
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

        if renderer
            .style
            .allow_colors
        {}

        write!(out, "{} {}", &self.prompt, &form,)?;

        out.flush()?;

        let mut input = String::new();

        stdin().read_line(&mut input)?;

        let Some(ch) = input
            .trim()
            .chars()
            .next()
        else {
            return self.render(renderer);
        };

        let mut stdout = StandardStream::stdout(ColorChoice::Auto);

        if let Some(item) = self
            .items
            .iter()
            .find(|i| i.keybind == ch)
        {
            match &item.val {
                Some(v) => return Ok(v.to_owned()),
                None if item.keybind == '?' => {
                    self.help(renderer, &mut stdout)?;
                    return self.render(renderer);
                }
                None => {
                    anyhow::bail!(
                        "no matching val? {}",
                        item.keybind
                    );
                }
            }
        } else {
            if renderer
                .style
                .allow_colors
            {
                stdout.set_color(
                    ColorSpec::new()
                        .set_fg(Some(renderer.style.error))
                        .set_bold(true),
                )?;
            }

            writeln!(stdout, "{ch} is not a valid option, see ?")?;

            stdout.reset()?;

            return self.render(renderer);
        }
    }

    fn help(
        &self,
        renderer: &Renderer,
        out: &mut StandardStream,
    ) -> anyhow::Result<()> {
        if renderer
            .style
            .allow_colors
        {
            out.set_color(
                ColorSpec::new()
                    .set_fg(Some(
                        renderer
                            .style
                            .tertiary,
                    ))
                    .set_bold(true),
            )?;
        }

        for item in self.items.iter() {
            writeln!(out, "{} - {}", item.keybind, item.description)?;
        }

        out.reset()?;

        Ok(())
    }
}
