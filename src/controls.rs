use iced_wgpu::Renderer;
use iced_widget::{
    button, container, horizontal_space, text, text_editor, Column, Row, TextEditor,
};
use iced_winit::core::text::highlighter;
use iced_winit::core::{Element, Length};
use iced_winit::runtime::{Command, Program};
use iced_winit::style::Theme;
use iced_winit::winit;
use winit::event_loop::EventLoopProxy;

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
        let content = text_editor::Content::with(include_str!("../shader.wgsl"));
        Controls {
            event_loop_proxy,
            content,
            editor_visible: true,
        }
    }
}

impl Program for Controls {
    type Renderer = Renderer<Theme>;
    type Message = Message;

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Edit(action) => {
                self.content.edit(action);
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
        Command::none()
    }

    fn view(&self) -> Element<'_, Message, Renderer<Theme>> {
        let position = {
            let (line, column) = self.content.cursor_position();
            text(format!("{}:{}", line + 1, column + 1))
        };

        let editor: TextEditor<'_, highlighter::PlainText, Message, Self::Renderer> =
            TextEditor::new(&self.content).on_edit(Message::Edit);

        let status_bar = Row::new()
            .push(horizontal_space(Length::Fill))
            .push(position);

        let control_buttons = Row::new()
            .push(
                button("Toggle editor")
                    .on_press(Message::ToggleEditor)
                    .width(Length::Fill),
            )
            .push(
                button("Update shader")
                    .on_press(Message::UpdateShader)
                    .width(Length::Fill),
            )
            .spacing(1)
            .padding(1);

        let mut column = Column::new().push(control_buttons);

        if self.editor_visible {
            column = column.push(editor).push(status_bar);
        }

        container(column).width(400).into()
    }
}
