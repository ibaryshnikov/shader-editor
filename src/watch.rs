use std::sync::mpsc;

use iced_winit::winit::event_loop::EventLoopProxy;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

use super::{CustomEvent, SHADER_FILE};

pub fn init(event_loop_proxy: EventLoopProxy<CustomEvent>) {
    let _handle = std::thread::spawn(move || {
        let (tx, rx) = mpsc::channel();

        let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();

        watcher
            .watch(SHADER_FILE.as_ref(), RecursiveMode::Recursive)
            .unwrap();

        for res in rx {
            match res {
                Ok(_event) => {
                    let shader_text =
                        std::fs::read_to_string(SHADER_FILE).expect("Should read the shader");
                    event_loop_proxy
                        .send_event(CustomEvent::UpdateShader(shader_text))
                        .expect("Should send custom winit event");
                }
                Err(e) => println!("watch error: {e:?}"),
            }
        }
    });
}
