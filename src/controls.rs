use iced_wgpu::Renderer;
use iced_widget::{button, container, horizontal_space, text, text_editor, Column, Row};
use iced_winit::core::{Element, Length, Theme};
use iced_winit::runtime::{Program, Task};
use iced_winit::winit;
use winit::event_loop::EventLoopProxy;

use crate::highlighter;
use crate::CustomEvent;

pub struct Controls {
    #[allow(unused)]
    event_loop_proxy: EventLoopProxy<CustomEvent>,
    content: text_editor::Content<<Controls as Program>::Renderer>,
    editor_visible: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Edit(text_editor::Action),
    UpdateShader,
    ToggleEditor,
}

impl Controls {
    pub fn new(event_loop_proxy: EventLoopProxy<CustomEvent>) -> Controls {
        let content = text_editor::Content::with_text(include_str!("../shader.wgsl"));
        Controls {
            event_loop_proxy,
            content,
            editor_visible: true,
        }
    }
}

impl Program for Controls {
    type Renderer = Renderer;
    type Theme = Theme;
    type Message = Message;

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Edit(action) => {
                self.content.perform(action);
            }
            Message::UpdateShader => {
                let shader_text = self.content.text();
                let event = CustomEvent::UpdateShader(shader_text);
                if let Err(e) = self.event_loop_proxy.send_event(event) {
                    println!("Error sending UpdateShader event: {}", e);
                }
            }
            Message::ToggleEditor => {
                self.editor_visible = !self.editor_visible;
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Message, Theme, Renderer> {
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

        let status_bar = Row::new().push(horizontal_space()).push(position);

        let control_buttons = Row::new()
            .push(
                button("Toggle editor")
                    .on_press(Message::ToggleEditor)
                    .width(Length::Fill)
                    .style(button::secondary),
            )
            .push(
                button("Update shader")
                    .on_press(Message::UpdateShader)
                    .width(Length::Fill)
                    .style(button::secondary),
            )
            .spacing(1)
            .padding(1);

        let mut column = Column::new().push(control_buttons);

        if self.editor_visible {
            column = column.push(editor).push(status_bar);
        }

        container(column).width(400).style(add_background).into()
    }
}

fn add_background(theme: &Theme) -> container::Style {
    theme.palette().background.into()
}
