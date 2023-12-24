use anyhow::{anyhow, Result};
use iced_wgpu::graphics::Viewport;
use iced_wgpu::{wgpu, Backend, Renderer, Settings};
use iced_winit::core::{mouse, renderer, Color, Font, Pixels, Size};
use iced_winit::runtime::{program, Debug};
use iced_winit::{conversion, winit, Clipboard};
use wgpu_types::TextureFormat;
use winit::event::{Event, ModifiersState, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};

mod controls;
mod editor;
mod scene;
mod watch;

use controls::Controls;
use editor::Editor;

#[derive(Debug)]
pub enum CustomEvent {
    ShaderFileChanged,
    UpdateShader(String),
}

struct RenderDetails {
    window: winit::window::Window,
    viewport: Viewport,
    clipboard: Clipboard,
    surface: wgpu::Surface,
    #[allow(unused)]
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    format: TextureFormat,
    config: wgpu::SurfaceConfiguration,
}

async fn init_wgpu(event_loop: &EventLoop<CustomEvent>) -> Result<RenderDetails> {
    let window = winit::window::Window::new(event_loop)?;
    window.set_title("Shader editor");

    log::info!("Initializing the surface...");

    let backends = wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all);
    let dx12_shader_compiler = wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default();

    let physical_size = window.inner_size();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends,
        dx12_shader_compiler,
    });

    let surface = unsafe { instance.create_surface(&window)? };

    let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, Some(&surface))
        .await
        .ok_or_else(|| anyhow!("Adapter not found"))?;

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
        .ok_or_else(|| anyhow!("Format not found"))?;

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: adapter_features & wgpu::Features::default(),
                limits: needed_limits,
            },
            None,
        )
        .await?;

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: physical_size.width,
        height: physical_size.height,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
    };
    let surface_view_format = config.format.add_srgb_suffix();
    config.view_formats.push(surface_view_format);
    surface.configure(&device, &config);

    println!("width {}", physical_size.width);
    println!("height {}", physical_size.height);
    let viewport = Viewport::with_physical_size(
        Size::new(physical_size.width, physical_size.height),
        window.scale_factor(),
    );
    let clipboard = Clipboard::connect(&window);

    let details = RenderDetails {
        window,
        viewport,
        clipboard,
        surface,
        adapter,
        device,
        queue,
        format,
        config,
    };
    Ok(details)
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let event_loop = EventLoopBuilder::<CustomEvent>::with_user_event().build();

    let RenderDetails {
        window,
        mut viewport,
        mut clipboard,
        surface,
        device,
        queue,
        format,
        mut config,
        ..
    } = init_wgpu(&event_loop)
        .await
        .expect("Should initialize wgpu details");

    log::info!("Initializing...");
    let mut editor = Editor::init(&config, &device);

    let mut resized = false;

    let mut staging_belt = wgpu::util::StagingBelt::new(5 * 1024);

    let event_loop_proxy = event_loop.create_proxy();
    let controls = Controls::new(event_loop_proxy.clone());

    // watch for shader changes
    watch::init(event_loop_proxy);

    let mut debug = Debug::new();
    let mut renderer = Renderer::new(
        Backend::new(&device, &queue, Settings::default(), format),
        Font::default(),
        Pixels(16.0),
    );

    let mut state =
        program::State::new(controls, viewport.logical_size(), &mut renderer, &mut debug);

    let mut cursor_position = None;
    let mut modifiers = ModifiersState::default();

    log::info!("Entering render loop...");
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        cursor_position = Some(position);
                    }
                    WindowEvent::ModifiersChanged(new_modifiers) => {
                        modifiers = new_modifiers;
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                                state: winit::event::ElementState::Pressed,
                                ..
                            },
                        ..
                    } => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode: Some(winit::event::VirtualKeyCode::F12),
                                state: winit::event::ElementState::Pressed,
                                ..
                            },
                        ..
                    } => {
                        debug.toggle();
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode: Some(winit::event::VirtualKeyCode::R),
                                state: winit::event::ElementState::Pressed,
                                ..
                            },
                        ..
                    } => {
                        if modifiers.ctrl() {
                            state.queue_message(controls::Message::UpdateShader);
                        }
                    }
                    WindowEvent::Resized(size) => {
                        resized = true;
                        log::info!("Resizing to {:?}", size);
                        config.width = size.width.max(1);
                        config.height = size.height.max(1);
                        editor.resize(&config, &queue);
                        surface.configure(&device, &config);
                    }
                    WindowEvent::ScaleFactorChanged {
                        new_inner_size: &mut size,
                        ..
                    } => {
                        log::info!("Resizing to {:?}", size);
                        config.width = size.width.max(1);
                        config.height = size.height.max(1);
                        editor.resize(&config, &queue);
                        surface.configure(&device, &config);
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => (),
                }

                if let Some(event) =
                    conversion::window_event(&event, window.scale_factor(), modifiers)
                {
                    state.queue_event(event);
                }
            }
            Event::UserEvent(custom_event) => match custom_event {
                CustomEvent::ShaderFileChanged => {
                    editor.update_rectangle_shader(&device, &config);
                    window.request_redraw();
                }
                CustomEvent::UpdateShader(text) => {
                    editor.update_rectangle_shader_with_text(&device, &config, &text);
                }
            },
            Event::MainEventsCleared => {
                if state.is_queue_empty() {
                    return;
                }
                let _ = state.update(
                    viewport.logical_size(),
                    cursor_position
                        .map(|position| {
                            conversion::cursor_position(position, viewport.scale_factor())
                        })
                        .map(mouse::Cursor::Available)
                        .unwrap_or(mouse::Cursor::Unavailable),
                    &mut renderer,
                    &iced_winit::style::Theme::Dark,
                    &renderer::Style {
                        text_color: Color::new(0.0, 0.6, 0.6, 1.0),
                    },
                    &mut clipboard,
                    &mut debug,
                );
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                if resized {
                    let size = window.inner_size();

                    viewport = Viewport::with_physical_size(
                        Size::new(size.width, size.height),
                        window.scale_factor(),
                    );

                    surface.configure(
                        &device,
                        &wgpu::SurfaceConfiguration {
                            format,
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                            width: size.width,
                            height: size.height,
                            present_mode: wgpu::PresentMode::AutoVsync,
                            alpha_mode: wgpu::CompositeAlphaMode::Auto,
                            view_formats: vec![],
                        },
                    );

                    resized = false;
                }

                match surface.get_current_texture() {
                    Ok(frame) => {
                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: None,
                            });

                        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
                            format: Some(format),
                            ..wgpu::TextureViewDescriptor::default()
                        });

                        editor.render(&view, &device, &queue);

                        renderer.with_primitives(|backend, primitive| {
                            backend.present(
                                &device,
                                &queue,
                                &mut encoder,
                                None,
                                &view,
                                primitive,
                                &viewport,
                                &debug.overlay(),
                            );
                        });

                        staging_belt.finish();
                        queue.submit(Some(encoder.finish()));
                        frame.present();

                        window.set_cursor_icon(conversion::mouse_interaction(
                            state.mouse_interaction(),
                        ));

                        staging_belt.recall();
                    }
                    Err(error) => match error {
                        wgpu::SurfaceError::OutOfMemory => {
                            panic!("Swapchain error: {error}. Rendering cannot continue.");
                        }
                        _ => {
                            // Try rendering again next frame.
                            window.request_redraw();
                        }
                    },
                }
            }
            _ => (),
        }
    })
}
