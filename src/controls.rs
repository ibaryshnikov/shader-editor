use iced_wgpu::Renderer;
use iced_widget::{Row, Text};
use iced_winit::core::{Alignment, Element, Length};
use iced_winit::runtime::{Command, Program};
use iced_winit::style::Theme;
use iced_winit::winit;
use winit::event_loop::EventLoopProxy;

use crate::CustomEvent;

pub struct Controls {
    #[allow(unused)]
    event_loop_proxy: EventLoopProxy<CustomEvent>,
}

#[derive(Debug, Clone)]
pub enum Message {}

impl Controls {
    pub fn new(event_loop_proxy: EventLoopProxy<CustomEvent>) -> Controls {
        Controls { event_loop_proxy }
    }
}

impl Program for Controls {
    type Renderer = Renderer<Theme>;
    type Message = Message;

    fn update(&mut self, _message: Message) -> Command<Message> {
        // match message {
        //
        // }
        Command::none()
    }

    fn view(&self) -> Element<Message, Renderer<Theme>> {
        Row::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Alignment::End)
            .push(Text::new("Built with iced"))
            .into()
    }
}
