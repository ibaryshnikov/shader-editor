use std::sync::Arc;
use std::time::Instant;

use iced_wgpu::graphics::{Shell, Viewport};
use iced_wgpu::{Engine, Renderer, wgpu};
use iced_winit::core::window;
use iced_winit::core::{Event, Font, Pixels, Size, Theme, mouse, renderer};
use iced_winit::runtime::user_interface::{self, UserInterface};
use iced_winit::{Clipboard, conversion, winit};
use wgpu_types::TextureFormat;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
use winit::window::Window;

mod controls;
mod editor;
mod highlighter;
mod scene;
mod validator;
mod watch;

use controls::{Controls, Message};
use editor::Editor;

const SHADER_FILE: &str = "./shaders/pattern_2.wgsl";
const SHADER_SOURCE: &str = include_str!("../shaders/pattern_2.wgsl");

#[derive(Debug)]
pub enum CustomEvent {
    ShaderFileChanged,
    UpdateShader(String),
}

struct App {
    app_data: Option<AppData>,
    resized: bool,
    cursor: mouse::Cursor,
    modifiers: ModifiersState,
    events: Vec<Event>,
    cache: user_interface::Cache,
    controls: Controls,
}

impl App {
    fn new(controls: Controls) -> App {
        let modifiers = ModifiersState::default();
        App {
            app_data: None,
            resized: false,
            cursor: mouse::Cursor::Unavailable,
            modifiers,
            events: Vec::new(),
            cache: user_interface::Cache::new(),
            controls,
        }
    }
}

struct AppData {
    window: Arc<Window>,
    viewport: Viewport,
    clipboard: Clipboard,
    surface: wgpu::Surface<'static>,
    #[allow(unused)]
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    format: TextureFormat,
    config: wgpu::SurfaceConfiguration,
    editor: Editor,
    renderer: Renderer,
}

impl ApplicationHandler<CustomEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.app_data.is_some() {
            println!("Already initialized, skipping");
            return;
        }
        let app_data = init_app(event_loop);
        self.app_data = Some(app_data);
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: CustomEvent) {
        let Some(app_data) = &mut self.app_data else {
            return;
        };
        match event {
            CustomEvent::ShaderFileChanged => {
                app_data
                    .editor
                    .update_rectangle_shader(&app_data.device, &app_data.config);
                app_data.window.request_redraw();
            }
            CustomEvent::UpdateShader(text) => {
                if let Err(e) = validator::validate(&text) {
                    self.controls.update(Message::ShaderError(e));
                    app_data.window.request_redraw();
                    return;
                } else {
                    self.controls.update(Message::ShaderValid);
                }
                app_data.editor.update_rectangle_shader_with_text(
                    &app_data.device,
                    &app_data.config,
                    &text,
                );
                app_data.window.request_redraw();
            }
        }
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(app_data) = self.app_data.as_mut() else {
            return;
        };
        let AppData {
            window,
            clipboard,
            surface,
            device,
            queue,
            format,
            editor,
            renderer,
            ..
        } = app_data;

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = mouse::Cursor::Available(conversion::cursor_position(
                    position,
                    app_data.viewport.scale_factor(),
                ));
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key,
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match physical_key {
                PhysicalKey::Code(KeyCode::Escape) => {
                    event_loop.exit();
                }
                PhysicalKey::Code(KeyCode::KeyR) => {
                    if self.modifiers.control_key() {
                        self.controls.update(controls::Message::UpdateShader);
                        return;
                    }
                }
                _ => (),
            },
            WindowEvent::Resized(_) => {
                self.resized = true;
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if self.resized {
                    let size = window.inner_size();

                    app_data.viewport = Viewport::with_physical_size(
                        Size::new(size.width, size.height),
                        window.scale_factor() as f32,
                    );

                    surface.configure(
                        device,
                        &wgpu::SurfaceConfiguration {
                            format: *format,
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                            width: size.width,
                            height: size.height,
                            present_mode: wgpu::PresentMode::AutoVsync,
                            alpha_mode: wgpu::CompositeAlphaMode::Auto,
                            view_formats: vec![],
                            desired_maximum_frame_latency: 2,
                        },
                    );

                    self.resized = false;
                }

                match surface.get_current_texture() {
                    Ok(frame) => {
                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: None,
                            });

                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        editor.render(&view, &mut encoder);

                        queue.submit([encoder.finish()]);

                        let mut interface = UserInterface::build(
                            self.controls.view(),
                            app_data.viewport.logical_size(),
                            std::mem::take(&mut self.cache),
                            renderer,
                        );

                        let (state, _) = interface.update(
                            &[Event::Window(
                                window::Event::RedrawRequested(Instant::now()),
                            )],
                            self.cursor,
                            renderer,
                            clipboard,
                            &mut Vec::new(),
                        );

                        if let user_interface::State::Updated {
                            mouse_interaction, ..
                        } = state
                        {
                            if let Some(icon) = conversion::mouse_interaction(mouse_interaction) {
                                window.set_cursor(icon);
                                window.set_cursor_visible(true);
                            } else {
                                window.set_cursor_visible(false);
                            }
                        }

                        let theme = Theme::SolarizedDark;
                        interface.draw(
                            renderer,
                            &theme,
                            &renderer::Style {
                                text_color: theme.palette().text,
                            },
                            self.cursor,
                        );
                        self.cache = interface.into_cache();

                        renderer.present(None, frame.texture.format(), &view, &app_data.viewport);

                        frame.present();
                    }
                    Err(error) => match error {
                        wgpu::SurfaceError::OutOfMemory => {
                            panic!("Swapchain error: {error}. Rendering cannot continue.");
                        }
                        _ => {
                            window.request_redraw();
                        }
                    },
                }
            }
            _ => (),
        }

        if let Some(event) =
            conversion::window_event(event, window.scale_factor() as f32, self.modifiers)
        {
            self.events.push(event);
        }

        if self.events.is_empty() {
            return;
        }
        let mut interface = UserInterface::build(
            self.controls.view(),
            app_data.viewport.logical_size(),
            std::mem::take(&mut self.cache),
            renderer,
        );

        let mut messages = Vec::new();

        let _ = interface.update(
            &self.events,
            self.cursor,
            renderer,
            clipboard,
            &mut messages,
        );

        self.events.clear();
        self.cache = interface.into_cache();

        for message in messages {
            self.controls.update(message);
        }

        window.request_redraw();
    }
}

fn init_app(event_loop: &ActiveEventLoop) -> AppData {
    let window = event_loop
        .create_window(winit::window::WindowAttributes::default())
        .expect("Should create window");

    let window = Arc::new(window);

    window.set_title("Shader editor");

    println!("Initializing the surface...");

    let physical_size = window.inner_size();

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::from_env_or_default());

    let surface = instance
        .create_surface(window.clone())
        .expect("Should create surface");

    let (format, adapter, device, queue) = futures::executor::block_on(async {
        let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, Some(&surface))
            .await
            .expect("Adapter not found");

        let adapter_info = adapter.get_info();
        println!("Using {} ({:?})", adapter_info.name, adapter_info.backend);

        let adapter_features = adapter.features();
        let needed_limits = wgpu::Limits::default();
        let capabilities = surface.get_capabilities(&adapter);

        let format = capabilities
            .formats
            .iter()
            .filter(|format| format.is_srgb())
            .copied()
            .next()
            .or_else(|| capabilities.formats.first().copied())
            .expect("Format not found");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: adapter_features & wgpu::Features::default(),
                required_limits: needed_limits,
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
            })
            .await
            .expect("Device not found");

        (format, adapter, device, queue)
    });

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: physical_size.width,
        height: physical_size.height,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    let surface_view_format = config.format.add_srgb_suffix();
    config.view_formats.push(surface_view_format);
    surface.configure(&device, &config);

    println!("width {}", physical_size.width);
    println!("height {}", physical_size.height);
    let viewport = Viewport::with_physical_size(
        Size::new(physical_size.width, physical_size.height),
        window.scale_factor() as f32,
    );
    let clipboard = Clipboard::connect(window.clone());

    let editor = Editor::init(&config, &device);

    let engine = Engine::new(
        &adapter,
        device.clone(),
        queue.clone(),
        format,
        None,
        Shell::headless(),
    );
    let renderer = Renderer::new(engine, Font::default(), Pixels(16.0));

    AppData {
        window,
        viewport,
        clipboard,
        surface,
        adapter,
        device,
        queue,
        format,
        config,
        editor,
        renderer,
    }
}

fn main() {
    let event_loop = EventLoop::with_user_event()
        .build()
        .expect("Should build event loop");

    let event_loop_proxy = event_loop.create_proxy();
    let controls = Controls::new(event_loop_proxy.clone());

    let mut app = App::new(controls);

    // watch for shader changes
    watch::init(event_loop_proxy);

    println!("Entering render loop...");
    event_loop.run_app(&mut app).expect("Should run event loop");
}
