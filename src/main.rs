use iced::{
    Application, Theme,
    advanced::{graphics::text::cosmic_text::Edit, widget::Text},
    highlighter, settings,
    widget::{Container, center, mouse_area, opaque, pick_list, stack},
};

use iced_aw::{
    menu, menu_bar, menu_items,
    widget::menu::{Item, Menu, MenuBar},
};

use iced::{
    Color, Element, Font, Length, Renderer, Task, application,
    highlighter::Highlight,
    widget::{
        Row, button, column, container, horizontal_space, row, text, text_editor,
        text_editor::Content,
    },
};

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{ffi, io};

struct Editor {
    path: Option<PathBuf>,
    content: text_editor::Content,
    editor_theme: iced::Theme,
    highlight_theme: highlighter::Theme,
    font: Font,
    current_error: Option<EditorError>,
    has_been_edited: bool,
    show_settings: bool,
}

impl Default for Editor {
    fn default() -> Self {
        Editor {
            path: None,
            content: Content::with_text(""),
            editor_theme: iced::Theme::CatppuccinFrappe,
            highlight_theme: highlighter::Theme::SolarizedDark,
            font: Font::MONOSPACE,
            current_error: None,
            has_been_edited: false,
            show_settings: false,
        }
    }
}

impl Editor {
    fn Theme(&self) -> iced::Theme {
        self.editor_theme.clone()
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
    FileOpened(Result<(PathBuf, Arc<String>), EditorError>),
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

fn update(state: &mut Editor, message: Messages) -> Task<Messages> {
    match message {
        Messages::Edit(edit_action) => {
            state.content.perform(edit_action);
            state.current_error = None;
            state.has_been_edited = true;
            Task::none()
        }
        Messages::EditorThemeSelected(new_editor_theme) => {
            state.editor_theme = new_editor_theme;
            Task::none()
        }
        Messages::ShowModal(should_show) => {
            state.show_settings = should_show;
            Task::none()
        }
        Messages::MenuOption(dropdown_option) => handle_file_option(state, dropdown_option),
        Messages::FileOpened(opening_result) => match opening_result {
            Ok((path_buf, new_content)) => {
                state.content = Content::with_text(&new_content);
                state.path = Some(path_buf);
                state.current_error = None;
                state.has_been_edited = false;
                Task::none()
            }
            Err(editor_error) => {
                state.current_error = Some(editor_error);
                Task::none()
            }
        },
        Messages::FileSaved(result) => match result {
            Ok(_) => {
                state.has_been_edited = false;
                Task::none()
            }
            Err(editor_error) => {
                state.current_error = Some(editor_error);
                Task::none()
            }
        },
        Messages::HighlighterThemeSelected(file_theme) => {
            state.highlight_theme = file_theme;
            Task::none()
        }
    }
}

fn handle_file_option(state: &mut Editor, file_option: DropdownOptions) -> Task<Messages> {
    match file_option {
        DropdownOptions::Open => Task::perform(pick_file(), Messages::FileOpened),
        DropdownOptions::Save => Task::perform(
            save_file(state.path.clone(), state.content.text()),
            Messages::FileSaved,
        ),
        DropdownOptions::New => {
            state.content = Content::with_text("");
            state.path = None;
            Task::none()
        }
    }
}

fn view(editor: &Editor) -> Element<Messages> {
    let input = text_editor(&editor.content)
        .highlight(
            editor
                .path
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

    let controls: Row<Messages, iced::Theme, Renderer> = row![file_bar];

    let status_bar = {
        let status: Text<'_, iced::Theme, Renderer> =
            if let Some(EditorError::IO(erorkind)) = editor.current_error.as_ref() {
                text(erorkind.to_string())
            } else {
                match editor.path.as_deref().and_then(Path::to_str) {
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

    let settings_container: Container<'_, Messages, iced::Theme, Renderer> = container(column![
        text("Settings").size(20),
        row![
            text("Code Highlight Theme:"),
            pick_list(
                highlighter::Theme::ALL,
                Some(editor.highlight_theme),
                Messages::HighlighterThemeSelected
            )
        ]
        .spacing(5),
        row![
            text("Editor Theme:"),
            pick_list(
                iced::Theme::ALL,
                Some(editor.editor_theme.clone()),
                Messages::EditorThemeSelected
            )
        ]
        .spacing(5),
    ]);

    let contents: Container<'_, Messages, iced::Theme, Renderer> =
        container(column![controls, input, status_bar].spacing(5))
            .height(Length::Fill)
            .width(Length::Fill)
            .padding(10);

    if (editor.show_settings) {
        modal(contents, settings_container, Messages::ShowModal(false))
    } else {
        contents.into()
    }
}

async fn load_file(path: PathBuf) -> Result<(PathBuf, Arc<String>), EditorError> {
    let contents = tokio::fs::read_to_string(&path)
        .await
        .map(Arc::new)
        .map_err(|error| error.kind())
        .map_err(EditorError::IO)?;

    Ok((path, contents))
}

async fn pick_file() -> Result<(PathBuf, Arc<String>), EditorError> {
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
        .theme(|_s| _s.Theme())
        .run()
    {
        Err(ex) => {}
        _ => {}
    }
}
