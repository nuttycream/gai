use console::Style;
use std::collections::HashSet;

// this is a tree printing util that helps
// pretty printing trees
// it's a modified version of the original
// arena graph that used crossterm
// this time however, it also takes heavy
// inspiration from the `tui-rs-tree-widget`
// crate that implements ratatui's widget trait
//
// the main difference, being this doesn't
// have state  since i dont plan on having
// interactivity for this.
//
// styling is done by console-rs

/// tui-rs-tree-widget esque TreeItem
/// while identifier is not really being used
/// since we dont have selected state or tracking
/// open/closed state, ill still keep it, mainly for when
/// we implement any type of fuzzy searching
#[derive(Debug, Clone)]
pub struct TreeItem<Identifier> {
    identifier: Identifier,
    children: Vec<Self>,

    text: String,
    style: Style,
}

/// A Tree which can be rendered
#[derive(Debug, Clone)]
pub struct Tree<'a, Identifier> {
    items: &'a [TreeItem<Identifier>],

    /// left padding is applied
    /// during prefix building
    padding_left: usize,
    padding_top: usize,
    padding_bottom: usize,

    style: Style,

    collapsed: bool,

    /// pre - pipe "│"
    other_child: &'a str,

    /// connector - tee "├──"
    other_entry: &'a str,

    /// pre - no more siblings " "
    final_child: &'a str,

    /// connector - elbow "└── "
    final_entry: &'a str,
}

impl<Identifier> TreeItem<Identifier>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    /// create a new treeitem but
    /// without children
    pub fn new_leaf<T>(
        identifier: Identifier,
        text: T,
    ) -> Self
    where
        T: Into<String>,
    {
        let text = text.into();

        Self {
            identifier,
            text,
            children: Vec::new(),
            style: Style::default(),
        }
    }

    /// create a new treeitem
    /// with children
    /// fails if the identifiers in the children are not
    /// unique
    pub fn new<T>(
        identifier: Identifier,
        text: T,
        children: Vec<Self>,
    ) -> std::io::Result<Self>
    where
        T: Into<String>,
    {
        let identifiers: HashSet<_> = children
            .iter()
            .map(|item| &item.identifier)
            .collect();

        if identifiers.len() != children.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "The children contain duplicate identifiers",
            ));
        }

        let text = text.into();

        Ok(Self {
            identifier,
            text,
            children,
            style: Style::default(),
        })
    }

    /// text content styling
    pub fn style(
        mut self,
        style: Style,
    ) -> Self {
        self.style = style;
        self
    }

    pub fn text(
        mut self,
        text: String,
    ) -> Self {
        self.text = text;
        self
    }

    pub const fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    pub fn children(&self) -> &[Self] {
        &self.children
    }

    pub fn child(
        &self,
        index: usize,
    ) -> Option<&Self> {
        self.children
            .get(index)
    }

    pub fn child_mut(
        &mut self,
        index: usize,
    ) -> Option<&mut Self> {
        self.children
            .get_mut(index)
    }
}

impl<'a, Identifier> Tree<'a, Identifier>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    pub fn new(
        items: &'a [TreeItem<Identifier>]
    ) -> std::io::Result<Self> {
        let identifiers = items
            .iter()
            .map(|item| &item.identifier)
            .collect::<HashSet<_>>();

        if identifiers.len() != items.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "The items contain duplicate identifiers",
            ));
        }

        let other_child = "│   ";
        let other_entry = "├── ";
        let final_child = "    ";
        let final_entry = "└── ";

        Ok(Self {
            items,
            padding_left: 0,
            padding_top: 0,
            padding_bottom: 0,
            style: Style::default(),
            collapsed: false,
            other_child,
            other_entry,
            final_child,
            final_entry,
        })
    }

    /// render tree
    /// using console-rs styling
    pub fn render(self) {
        if self
            .items
            .is_empty()
        {
            return;
        }

        // top
        for _ in 0..self.padding_top {
            println!();
        }

        let flattened = flatten(self.items, &[], self.collapsed, 0);

        for flat in flattened.iter() {
            let prefix = self.prefix(&flat.is_last_at_depth);
            let prefix = self
                .style
                .apply_to(&prefix);
            let text = flat
                .item
                .style
                .apply_to(&flat.item.text);

            println!("{prefix}{text}");
        }

        // bottom
        for _ in 0..self.padding_bottom {
            println!();
        }
    }

    /// helper util func to return as string
    /// instead of printing to terminal
    /// should still apply Stylings
    /// using this instead a From<> impl
    pub fn as_string(self) -> String {
        if self
            .items
            .is_empty()
        {
            return String::new();
        }

        let mut s = String::new();
        let flattened = flatten(self.items, &[], self.collapsed, 0);

        for flat in flattened.iter() {
            let prefix = self.prefix(&flat.is_last_at_depth);
            let prefix = self
                .style
                .apply_to(&prefix);
            let text = flat
                .item
                .style
                .apply_to(&flat.item.text);

            s.push_str(&format!("{prefix}{text}\n"));
        }

        s
    }

    /// prefix styling
    pub fn style(
        mut self,
        style: Style,
    ) -> Self {
        self.style = style;
        self
    }

    /// show the tree as collapsed
    /// which only displays the roots
    pub fn collapsed(
        mut self,
        collapsed: bool,
    ) -> Self {
        self.collapsed = collapsed;
        self
    }

    /// set left space padding
    pub fn padding_left(
        mut self,
        padding: usize,
    ) -> Self {
        self.padding_left = padding;
        self
    }

    /// set bottom space padding
    pub fn padding_bottom(
        mut self,
        padding: usize,
    ) -> Self {
        self.padding_bottom = padding;
        self
    }

    /// set top space padding
    pub fn padding_top(
        mut self,
        padding: usize,
    ) -> Self {
        self.padding_top = padding;
        self
    }

    pub fn other_child(
        mut self,
        other_child: &'a str,
    ) -> Self {
        self.other_child = other_child;
        self
    }

    pub fn other_entry(
        mut self,
        other_entry: &'a str,
    ) -> Self {
        self.other_entry = other_entry;
        self
    }

    pub fn final_child(
        mut self,
        final_child: &'a str,
    ) -> Self {
        self.final_child = final_child;
        self
    }

    pub fn final_entry(
        mut self,
        final_entry: &'a str,
    ) -> Self {
        self.final_entry = final_entry;
        self
    }

    // util create the prefix character
    // based on a flattened item
    fn prefix(
        &self,
        is_last_at_depth: &[bool],
    ) -> String {
        let depth = is_last_at_depth.len();

        let mut prefix = " ".repeat(self.padding_left);

        if depth == 0 {
            return prefix;
        }

        // add continuation characters
        for &is_last in &is_last_at_depth[..depth - 1] {
            if is_last {
                prefix.push_str(self.final_child);
            } else {
                prefix.push_str(self.other_child);
            }
        }

        // add connector for curr_level
        if is_last_at_depth[depth - 1] {
            prefix.push_str(self.final_entry);
        } else {
            prefix.push_str(self.other_entry);
        }

        prefix
    }
}

// util flatten function, compared to tui-rs-tree-widget
// we don't collapse per tree item, the entire tree is collapsed
// but i think keeping the identifier is fine here, in case
// i do want to track which tree item corresponds to whatever
// likely wont use it though
struct Flattened<'text, Identifier> {
    item: &'text TreeItem<Identifier>,
    /// assign the last item for the each depth
    is_last_at_depth: Vec<bool>,
}

fn flatten<'text, Identifier>(
    items: &'text [TreeItem<Identifier>],
    parent_is_last_chain: &[bool],
    collapsed: bool,
    depth: usize,
) -> Vec<Flattened<'text, Identifier>>
where
    Identifier: Clone + PartialEq + Eq + core::hash::Hash,
{
    let mut flattened = Vec::new();
    let len = items.len();

    for (i, item) in items
        .iter()
        .enumerate()
    {
        let is_last = i == len - 1;

        // Roots (depth 0) have empty is_last_at_depth (no prefix)
        // Children extend parent's chain with their own is_last status
        let is_last_at_depth = if depth == 0 {
            Vec::new()
        } else {
            let mut chain = parent_is_last_chain.to_vec();
            chain.push(is_last);
            chain
        };

        flattened.push(Flattened {
            item,
            is_last_at_depth: is_last_at_depth.clone(),
        });

        // handle children with recursion
        if !collapsed {
            let children = flatten(
                &item.children,
                &is_last_at_depth,
                collapsed,
                depth + 1,
            );

            flattened.extend(children);
        }
    }

    flattened
}
