use std::collections::HashMap;

pub struct GitState {
    /// We can track changes using this
    pub changes: HashMap<String, Status>,
}

pub enum Status {
    Modified,
    Untracked,
}
