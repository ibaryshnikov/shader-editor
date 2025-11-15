use iced_wgpu::Renderer;
use iced_widget::{button, column, container, row, space, text, text_editor};
use iced_winit::core::{Element, Length, Theme};
use iced_winit::winit;
use winit::event_loop::EventLoopProxy;

use crate::{CustomEvent, SHADER_SOURCE, highlighter};

pub struct Controls {
    event_loop_proxy: EventLoopProxy<CustomEvent>,
    content: text_editor::Content<Renderer>,
    editor_visible: bool,
    shader_error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Edit(text_editor::Action),
    UpdateShader,
    ToggleEditor,
    ShaderError(String),
    ShaderValid,
}

impl Controls {
    pub fn new(event_loop_proxy: EventLoopProxy<CustomEvent>) -> Controls {
        let content = text_editor::Content::with_text(SHADER_SOURCE);
        Controls {
            event_loop_proxy,
            content,
            editor_visible: true,
            shader_error: None,
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Edit(action) => {
                self.content.perform(action);
            }
            Message::UpdateShader => {
                let shader_text = self.content.text();
                let event = CustomEvent::UpdateShader(shader_text);
                if let Err(e) = self.event_loop_proxy.send_event(event) {
                    println!("Error sending UpdateShader event: {e}");
                }
            }
            Message::ToggleEditor => {
                self.editor_visible = !self.editor_visible;
            }
            Message::ShaderError(e) => {
                self.shader_error = Some(e);
            }
            Message::ShaderValid => {
                self.shader_error = None;
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message, Theme, Renderer> {
        let position = {
            let (line, column) = self.content.cursor_position();
            text(format!("{}:{}", line + 1, column + 1))
        };

        let editor = text_editor(&self.content)
            .on_action(Message::Edit)
            .highlight_with::<highlighter::Highlighter>(
                highlighter::Settings {
                    theme: highlighter::Theme::SolarizedDark,
                    token: "wgsl".to_owned(),
                },
                |highlight, _theme| highlight.to_format(),
            );

        let status_bar = row![space::horizontal(), position];

        let control_buttons = row![
            button("Toggle editor")
                .on_press(Message::ToggleEditor)
                .width(Length::Fill)
                .style(button::secondary),
            button("Update shader")
                .on_press(Message::UpdateShader)
                .width(Length::Fill)
                .style(button::secondary),
        ]
        .spacing(1)
        .padding(1);

        let mut column = column![control_buttons];

        if self.editor_visible {
            column = column.push(editor).push(status_bar);
        }
        if let Some(error) = &self.shader_error {
            column = column.push(text(error));
        }

        container(column).width(500).style(add_background).into()
    }
}

fn add_background(theme: &Theme) -> container::Style {
    theme.palette().background.into()
}
