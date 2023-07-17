# Volume Data Suite
Volume Data Suite (VDS) is a free and open source app that can be used to visualize and interact with volumetric image data.

View the web demo app online at <https://online.volumedatasuite.com> or download the [latest native version](https://github.com/Volume-Data-Suite/vds/releases) for your plattform.

## Status
VDS is currently nothing more than a proof of concept and in a very early pre-alpha state. There are many features I want to add, and the API and User Interface is still evolving. _Expect breaking changes!_

## Supported Features
### Cross-Platform Support
Currently the following graphics APIs are supported:
- Windows (Vulkan and DirectX)
- macOS (Metal)
- Linux (Vulkan)
- Web (WebGPU)

_WegGL_ and _OpenGL_ are currrently not supported due to significant performance reductions. Make sure that you use the latest version of your web browser and check its WebGPU support, if you want to use a the web based version of VDS. The latest versions of Chromium based browsers like Google Chrome, Microsoft Edge and Chromium itself should work.

### File Formats
Currenltly the following file formats can be imported:
- RAW 3D

### Visulatization
Currently the volume data can be visualized as:
- 2D Slices (X-Axis only)

## Development
VDS is written in [Rust](https://www.rust-lang.org/) and shaders are written [WGSL](https://www.w3.org/TR/WGSL/).

### Building for native
Run it locally with `cargo run --release`.

### Building for web
[trunk](https://trunkrs.dev/) can be used to compile `vds` to WASM and spin up a web server with hot reloading. Run it locally with `trunk serve`.

Afterwards VDS can be viewed in a browser locally (http://127.0.0.1:8080/index.html). If you want to disable local caching, you can add '#dev' to the URL (http://127.0.0.1:8080/index.html#dev).

