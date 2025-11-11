// yes
//
// https://rust-leipzig.github.io/architecture/2016/12/20/idiomatic-trees-in-rust/

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use std::io::Write;

pub struct Arena {
    nodes: Vec<Node>,
}

pub struct Node {
    parent: Option<usize>,
    children: Vec<usize>,
    prefix: Option<String>,
    count: Option<usize>,
    color: Color,
    text: String,
}

impl Arena {
    pub fn new() -> Self {
        Arena { nodes: Vec::new() }
    }

    pub fn new_node(
        &mut self,
        text: impl Into<String>,
        color: Color,
    ) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(Node {
            parent: None,
            children: Vec::new(),
            prefix: None,
            count: None,
            color,
            text: text.into(),
        });
        idx
    }

    // prefix for a node M A D for staging etc.
    pub fn set_prefix(
        &mut self,
        node_id: usize,
        prefix: impl Into<String>,
    ) {
        self.nodes[node_id].prefix = Some(prefix.into());
    }

    // sets optional count
    pub fn set_count(&mut self, node_id: usize, count: usize) {
        self.nodes[node_id].count = Some(count);
    }

    pub fn add_child(&mut self, parent_id: usize, child_id: usize) {
        self.nodes[parent_id].children.push(child_id);
        self.nodes[child_id].parent = Some(parent_id);
    }

    // helper for truncating, honestly specific to commit msg bodiesy
    // dont want to deal with multiline wrapping
    pub fn truncate(&self, text: &str, max_len: usize) -> String {
        if text.len() > max_len {
            format!("{}...", &text[..max_len])
        } else {
            text.to_string()
        }
    }

    // some bits of print_tree and print_node were LLM generated
    // surface level, they seem fine
    // ideally didn't want to introduce another crate such as
    // ptree
    // since they don't use crossterm
    // this is fine for now

    pub fn print_tree<W: Write>(
        &self,
        writer: &mut W,
    ) -> std::io::Result<()> {
        let roots: Vec<usize> = self
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node.parent.is_none())
            .map(|(idx, _)| idx)
            .collect();

        for (i, &root_id) in roots.iter().enumerate() {
            let is_last = i == roots.len() - 1;
            self.print_node(writer, root_id, "", is_last, 0)?;
        }

        Ok(())
    }

    fn print_node<W: Write>(
        &self,
        writer: &mut W,
        node_id: usize,
        indent: &str,
        is_last: bool,
        level: usize,
    ) -> std::io::Result<()> {
        let node = &self.nodes[node_id];

        let (branch, continuation) = match level {
            0 => {
                let branch =
                    if is_last { "└─" } else { "├─" };
                let continuation =
                    if is_last { "  " } else { "│ " };
                (branch, continuation)
            }
            1 => {
                let branch =
                    if is_last { "└──" } else { "├──" };
                let continuation =
                    if is_last { "   " } else { "│  " };
                (branch, continuation)
            }
            _ => {
                let branch =
                    if is_last { "└─" } else { "├─" };
                let continuation =
                    if is_last { "  " } else { "│ " };
                (branch, continuation)
            }
        };

        execute!(
            writer,
            SetForegroundColor(Color::DarkGrey),
            Print(format!("{}{} ", indent, branch)),
        )?;

        if let Some(prefix) = &node.prefix {
            execute!(
                writer,
                SetForegroundColor(node.color),
                Print(format!("{} ", prefix)),
            )?;
        }

        execute!(
            writer,
            SetForegroundColor(node.color),
            Print(&node.text),
        )?;

        if let Some(count) = node.count {
            execute!(
                writer,
                SetForegroundColor(Color::DarkGrey),
                Print(format!(" [{}]", count)),
            )?;
        }

        execute!(writer, ResetColor, Print("\n"))?;

        let new_indent = format!("{}{}", indent, continuation);
        for (i, &c_id) in node.children.iter().enumerate() {
            let last_child = i == node.children.len() - 1;

            // recurse here, its ok
            self.print_node(
                writer,
                c_id,
                &new_indent,
                last_child,
                level + 1,
            )?;
        }

        Ok(())
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}
