use iced::{
    Alignment::Center,
    advanced::widget::Text,
    alignment::Horizontal::Left,
    highlighter,
    widget::{Container, center, mouse_area, opaque, pick_list, stack},
};

use iced_aw::{
    menu, menu_bar, menu_items,
    widget::menu::{Item, Menu, MenuBar},
};

use iced::{
    Color, Element, Font, Length, Renderer, Task, application,
    widget::{
        Row, button, column, container, horizontal_space, row, text, text_editor,
        text_editor::Content,
    },
};

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{ffi, io};

struct Tab {
    file_name: String,
    opt_path: Option<PathBuf>,
    content: text_editor::Content,
    has_been_edited: bool,
    is_new: bool,
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
}

struct Editor {
    current_tab_path: Option<PathBuf>,
    open_tabs: Vec<Tab>,
    editor_theme: iced::Theme,
    highlight_theme: highlighter::Theme,
    font: Font,
    current_error: Option<EditorError>,
    show_settings: bool,
}

impl Default for Editor {
    fn default() -> Self {
        Editor {
            current_tab_path: None,
            open_tabs: Vec::new(),
            editor_theme: iced::Theme::CatppuccinFrappe,
            highlight_theme: highlighter::Theme::SolarizedDark,
            font: Font::MONOSPACE,
            current_error: None,
            show_settings: false,
        }
    }
}

impl Editor {
    fn theme(&self) -> iced::Theme {
        self.editor_theme.clone()
    }

    fn file_already_open(&self, file_path: PathBuf) -> bool {
        for tab in self.open_tabs.iter() {
            if tab
                .opt_path
                .as_ref()
                .is_some_and(|tab_path| *tab_path == file_path)
            {
                return true;
            }
        }

        return false;
    }

    //This function attempts to update the tab as saved. Failing to find it returns false.
    fn file_saved(&mut self, file_path: PathBuf) -> bool {
        return false;
    }

    fn get_current_tab(&mut self) -> Option<&mut Tab> {
        match self.current_tab_path.clone() {
            Some(current_tab_path) => {
                for tab in self.open_tabs.iter_mut() {
                    if tab
                        .opt_path
                        .as_ref()
                        .is_some_and(|tab_path| *tab_path == current_tab_path)
                    {
                        return Some(tab);
                    }
                }
            }
            None => return None,
        }

        return None;
    }

    fn open_tab(
        &mut self,
        file_path: PathBuf,
        new_content: Arc<String>,
        new_file_name: String,
        is_new_file: bool,
    ) {
        let new_tab = Tab {
            is_new: is_new_file,
            opt_path: Some(file_path.clone()),
            content: Content::with_text(&new_content),
            file_name: new_file_name,
            has_been_edited: is_new_file,
        };
        self.open_tabs.push(new_tab);
        self.current_tab_path = Some(file_path.clone());
        self.current_error = None;
    }
}

#[derive(Debug, Clone, PartialEq)]
enum DropdownOptions {
    Open,
    Save,
    New,
}

#[derive(Debug, Clone)]
enum Messages {
    Edit(text_editor::Action),
    MenuOption(DropdownOptions),
    FileOpened(Result<(PathBuf, Arc<String>, String, bool), EditorError>),
    FileSaved(Result<PathBuf, EditorError>),
    HighlighterThemeSelected(highlighter::Theme),
    EditorThemeSelected(iced::Theme),
    ShowModal(bool),
}

#[derive(Debug, Clone)]
enum EditorError {
    DialogClosed,
    IO(io::ErrorKind),
    FailedToSave,
}

fn update(editor: &mut Editor, message: Messages) -> Task<Messages> {
    match message {
        Messages::Edit(edit_action) => {
            match editor.get_current_tab() {
                Some(tab) => {
                    tab.perform_edit(edit_action);
                    editor.current_error = None;
                }
                None => {
                    //this is fine, we're not really worried about it not having a current tab
                }
            }
            Task::none()
        }
        Messages::EditorThemeSelected(new_editor_theme) => {
            editor.editor_theme = new_editor_theme;
            Task::none()
        }
        Messages::ShowModal(should_show) => {
            editor.show_settings = should_show;
            Task::none()
        }
        Messages::MenuOption(dropdown_option) => handle_file_option(editor, dropdown_option),
        Messages::FileOpened(opening_result) => match opening_result {
            Ok((path_buf, new_content, new_file_name, is_new_file)) => Task::none(),
            Err(editor_error) => {
                editor.current_error = Some(editor_error);
                Task::none()
            }
        },
        Messages::FileSaved(result) => match result {
            Ok(file_path) => {
                match editor.file_saved(file_path) {
                    true => {
                        //fine and expected. Good even.
                    }
                    false => {
                        // TODO: this is definitely an error, and we have got to figure out how to handle it
                    }
                }
                Task::none()
            }
            Err(editor_error) => {
                editor.current_error = Some(editor_error);
                Task::none()
            }
        },
        Messages::HighlighterThemeSelected(file_theme) => {
            editor.highlight_theme = file_theme;
            Task::none()
        }
    }
}

fn handle_file_option(editor: &mut Editor, file_option: DropdownOptions) -> Task<Messages> {
    match file_option {
        DropdownOptions::Open => Task::perform(pick_file(), Messages::FileOpened),
        DropdownOptions::Save => {
            match editor.get_current_tab() {
                Some(tab) => Task::perform(
                    save_file(tab.opt_path.clone(), tab.content.text()),
                    Messages::FileSaved,
                ),
                None => {
                    //trying to save without a tab open isn't a sin
                    Task::none()
                }
            }
        }
        DropdownOptions::New => {
            editor.open_tab(file_path, new_content, new_file_name, is_new_file);
            state.content = ;
            state.current_tab_path = None;
            Task::none()
        }
    }
}

fn view(editor: &Editor) -> Element<Messages> {
    let input = text_editor(&editor.content)
        .highlight(
            editor
                .current_tab_path
                .as_deref()
                .and_then(Path::extension)
                .and_then(ffi::OsStr::to_str)
                .unwrap_or("rs"),
            editor.highlight_theme,
        )
        .on_action(Messages::Edit)
        .height(Length::Fill);

    let menu_template = |items| Menu::new(items).max_width(180.0).offset(6.0).spacing(0);

    let file_bar: MenuBar<'_, Messages, iced::Theme, iced::Renderer> = menu_bar!((
        button("File"),
        menu_template(menu_items!((button("New")
            .width(Length::Fill)
            .on_press(Messages::MenuOption(DropdownOptions::New)))(
            button("Open")
                .width(Length::Fill)
                .on_press(Messages::MenuOption(DropdownOptions::Open))
        )(
            button("Save")
                .width(Length::Fill)
                .on_press(Messages::MenuOption(DropdownOptions::Save))
        )(
            button("settings")
                .width(Length::Fill)
                .on_press(Messages::ShowModal(true))
        )))
    ));

    let mut tab_row: Row<Messages, iced::Theme, Renderer> = row![];

    for tab in editor.open_tabs {
        tab_row = tab_row.push(button(text(tab.file_name.clone())))
    }

    let controls: Row<Messages, iced::Theme, Renderer> = row![file_bar, tab_row].spacing(5);

    let status_bar = {
        let status: Text<'_, iced::Theme, Renderer> =
            if let Some(EditorError::IO(erorkind)) = editor.current_error.as_ref() {
                text(erorkind.to_string())
            } else {
                match editor.current_tab_path.as_deref().and_then(Path::to_str) {
                    Some(found_path) => text(found_path).size(14),
                    None => text("New File"),
                }
            };

        let position = {
            let (line, column) = editor.content.cursor_position();

            text(format!("{}:{}", line + 1, column + 1))
        };

        row![status, horizontal_space(), position]
    };

    let settings_container: Container<'_, Messages, iced::Theme, Renderer> = container(
        column![
            text("Settings").size(20),
            row![
                column![text("Code Highlight Theme:"), text("Editor Theme:")]
                    .width(225)
                    .spacing(10)
                    .align_x(Left),
                column![
                    pick_list(
                        highlighter::Theme::ALL,
                        Some(editor.highlight_theme),
                        Messages::HighlighterThemeSelected
                    ),
                    pick_list(
                        iced::Theme::ALL,
                        Some(editor.editor_theme.clone()),
                        Messages::EditorThemeSelected
                    )
                ]
                .width(150)
                .spacing(10)
                .align_x(Left)
            ],
        ]
        .align_x(Center)
        .spacing(10),
    );

    let contents: Container<'_, Messages, iced::Theme, Renderer> =
        container(column![controls, input, status_bar].spacing(5))
            .height(Length::Fill)
            .width(Length::Fill)
            .padding(10);

    if editor.show_settings {
        modal(contents, settings_container, Messages::ShowModal(false))
    } else {
        contents.into()
    }
}

async fn load_file(path: PathBuf) -> Result<(PathBuf, Arc<String>, String, bool), EditorError> {
    let contents = tokio::fs::read_to_string(&path)
        .await
        .map(Arc::new)
        .map_err(|error| error.kind())
        .map_err(EditorError::IO)?;

    let file_name: String = match path.file_name() {
        Some(x) => match x.to_str() {
            Some(z) => z.to_string(),
            None => "Unnamed".to_string(),
        },
        None => "Unnamed".to_string(),
    };

    Ok((path, contents, file_name, false))
}

async fn pick_file() -> Result<(PathBuf, Arc<String>, String, bool), EditorError> {
    let file_path = rfd::AsyncFileDialog::new()
        .set_title("Choose a text file...")
        .pick_file()
        .await
        .ok_or(EditorError::DialogClosed)?;

    load_file(file_path.path().to_owned()).await
}

async fn save_file(opt_path: Option<PathBuf>, text: String) -> Result<PathBuf, EditorError> {
    let path = if let Some(current_path) = opt_path {
        current_path
    } else {
        rfd::AsyncFileDialog::new()
            .set_title("Save File...")
            .save_file()
            .await
            .ok_or(EditorError::DialogClosed)
            .map(|handle| handle.path().to_owned())?
    };

    save_file_work(path, text).await
}

async fn save_file_work(path: PathBuf, text: String) -> Result<PathBuf, EditorError> {
    tokio::fs::write(&path, text)
        .await
        .map_err(|err| EditorError::IO(err.kind()))?;

    Ok(path)
}

fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    on_blur: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    stack![
        base.into(),
        opaque(
            mouse_area(center(opaque(content)).style(|_theme| {
                container::Style {
                    background: Some(
                        Color {
                            a: 0.8,
                            ..Color::BLACK
                        }
                        .into(),
                    ),
                    ..container::Style::default()
                }
            }))
            .on_press(on_blur)
        )
    ]
    .into()
}

fn main() {
    match application("Rust Editor", update, view)
        .theme(|_s| _s.theme())
        .run()
    {
        Err(ex) => {}
        _ => {}
    }
}
