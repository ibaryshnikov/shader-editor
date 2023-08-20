# Shader editor

A minimal WebGPU application which can run your WGSL shaders.
Built on top of iced, providing the ability to add some UI on top
of the rendered image.

## Motivation

The main goal is to provide a quick way to develop shaders with
a fast iteration loop. Inspired by tools like shadertoy.

## Usage

Do `cargo run` and edit `shader.wgsl` file

## Roadmap

- [x] Watch shader file and do hot reload on changes
- [ ] Menu dialog to open shader files
- [ ] Multiline text editor for shaders

## Key dependencies

Built with iced, wgpu, winit and other crates.
The implementation is based on the following examples:
- iced [integration](https://github.com/iced-rs/iced/tree/master/examples/integration)
- wgpu [cube](https://github.com/gfx-rs/wgpu/tree/trunk/examples/cube)
  and [hello-triangle](https://github.com/gfx-rs/wgpu/tree/trunk/examples/hello-triangle).

Using [notify](https://github.com/notify-rs/notify)
for file watching.

## License

MIT or Apache-2.0
