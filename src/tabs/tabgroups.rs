use std::path::PathBuf;

use crate::tabs::tabs::Tab;

pub struct TabGroup {
    subtabs: Vec<Tab>,
    opt_tab_index: Option<usize>,
    optname: Option<String>,
}

impl TabGroup {
    pub fn containing_file_index(&self, reference_path: PathBuf) -> Option<usize> {
        for (index, tab) in self.subtabs.iter().enumerate() {
            if tab.is_path(reference_path.clone()) {
                return Some(index);
            }
        }
        None
    }
}
