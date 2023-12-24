# Shader editor

A minimal WebGPU application which can run your WGSL shaders.
Built on top of iced, providing the ability to add some UI on top
of the rendered image.

## Motivation

The main goal is to provide a quick way to develop shaders with
a fast iteration loop. Inspired by tools like shadertoy.

## Usage

Use `cargo run` to start

Multiline editor:
- edit shader in the text area
- press `Update shader` or `Ctrl+R` to reload shader
- press `Toggle editor` to hide shader text

File watcher:
- edit `shader.wgsl` file, it will be reloaded on changes

## Preview

<img alt="preview" src="editor.png">

## Roadmap

- [x] Watch shader file and do hot reload on changes
- [ ] Menu dialog to open shader files
- [x] Multiline text editor for shaders
- [ ] Syntax highlight
- [ ] Show error position inside editor

## Key dependencies

Built with iced, wgpu, winit and other crates.
The implementation is based on the following examples:
- iced [integration](https://github.com/iced-rs/iced/tree/master/examples/integration)
- wgpu [cube](https://github.com/gfx-rs/wgpu/tree/trunk/examples/cube)
  and [hello-triangle](https://github.com/gfx-rs/wgpu/tree/trunk/examples/hello-triangle).

The text editor uses experimental iced widget.

Using [notify](https://github.com/notify-rs/notify)
for file watching.

## License

MIT or Apache-2.0
