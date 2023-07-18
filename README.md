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

#### Web Plattform restrictions:
**_WebGL_** is currrently not supported due to significant performance reductions. Make sure that you use the latest version of your web browser and check its **_WebGPU_** support, if you want to use the web based version of VDS. The latest versions of Chromium based browsers like Google Chrome, Microsoft Edge and Chromium itself should work.

**_Direct file system access_** is currently [not possible](https://stackoverflow.com/questions/71017592/can-i-read-files-from-the-disk-by-using-webassembly-re-evaluated) with WebAssembly (WASM). Therefore files can only be opened via drag and drop or by downloading them from a URL and not all file types are supported. Files can only be exported as a download.

**_Gerneral Perfomance_** is better on native versions of VDS that run directly on the operating system and not inside a web browser. Main reason are missing [SIMD](https://en.wikipedia.org/wiki/Single_instruction,_multiple_data) like _f16c_ on x86 or _fp16_ on arm64 which are not available in WASM.

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

