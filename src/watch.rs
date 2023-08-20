use std::sync::mpsc;

use iced_winit::winit::event_loop::EventLoopProxy;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

use super::CustomEvent;

pub fn init(event_loop_proxy: EventLoopProxy<CustomEvent>) {
    let _handle = std::thread::spawn(move || {
        let (tx, rx) = mpsc::channel();

        let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();

        watcher
            .watch("./src/shader.wgsl".as_ref(), RecursiveMode::Recursive)
            .unwrap();

        for res in rx {
            match res {
                Ok(_event) => {
                    event_loop_proxy
                        .send_event(CustomEvent::ShaderFileChanged)
                        .expect("Should send custom winit event");
                }
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    });
}
