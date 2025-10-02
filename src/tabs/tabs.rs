use std::path::PathBuf;

use iced::widget::text_editor::{self, Content};

pub struct Tab {
    pub file_name: String,
    pub opt_path: Option<PathBuf>,
    pub content: text_editor::Content,
    pub has_been_edited: bool,
    pub is_new: bool,
}

impl Tab {
    fn file_saved(&mut self) {
        self.has_been_edited = false;
        self.is_new = false;
    }

    fn perform_edit(&mut self, action: text_editor::Action) {
        self.content.perform(action);
        self.has_been_edited = true;
    }

    pub fn is_path(&self, path_in_question: PathBuf) -> bool {
        match self.opt_path.clone() {
            Some(self_path) => self_path == path_in_question,
            None => false,
        }
    }
}

impl Default for Tab {
    fn default() -> Self {
        Tab {
            file_name: "Untitlted".to_string(),
            opt_path: None,
            content: Content::with_text(""),
            has_been_edited: true,
            is_new: true,
        }
    }
}
