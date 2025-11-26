use std::{path::PathBuf, usize};

use crate::tabs::tabs::Tab;

pub struct TabGroup {
    subtabs: Vec<Tab>,
    index: usize,
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

    pub fn get_tabgroup_name(&self) -> String {
        match self.optname.clone() {
            Some(name) => name.clone(),
            None => match self.subtabs.first() {
                Some(initial_tab) => initial_tab.file_name.clone(),
                None => "Unnamed".to_string(),
            },
        }
    }

    pub fn create_tabgroup_with_single_tab(
        tab: Tab,
        index: usize,
        opt_name: Option<String>,
    ) -> TabGroup {
        let mut contents_vec: Vec<Tab> = Vec::new();
        contents_vec.push(tab);
        TabGroup {
            subtabs: contents_vec,
            index,
            optname: opt_name,
        }
    }

    pub fn create_tabgroup_with_multiple_tabs(
        tabs: Vec<Tab>,
        index: usize,
        opt_name: Option<String>,
    ) -> TabGroup {
        TabGroup {
            subtabs: tabs,
            index,
            optname: opt_name,
        }
    }

    pub fn get_opt_head_tab_mut(&mut self) -> Option<&mut Tab> {
        if self.subtabs.len() != 0 {
            self.subtabs.get_mut(0)
        } else {
            None
        }
    }

    pub fn get_opt_head_tab(&self) -> Option<&Tab> {
        if self.subtabs.len() != 0 {
            self.subtabs.get(0)
        } else {
            None
        }
    }
}

impl Default for TabGroup {
    fn default() -> Self {
        TabGroup {
            subtabs: Vec::new(),
            index: 0,
            optname: None,
        }
    }
}
