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

Make sure you are using the latest version of stable rust by running `rustup update`. VDS requires a proper graphics driver that provides at least one of the [supported graphics APIs](#cross-platform-support).

### Building for Native

Run it locally with `cargo run --release`.

On Linux you need to first run:

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev`

On Fedora Rawhide you need to run:

`dnf install clang clang-devel clang-tools-extra libxkbcommon-devel pkg-config openssl-devel libxcb-devel gtk3-devel atk fontconfig-devel`

### Buildin for Web

[Trunk](https://trunkrs.dev/) can be used to compile `vds` to WASM and spin up a web server with hot reloading.

1. Install Trunk with `cargo install --locked trunk`.
2. Run `trunk serve` to build and serve on `http://127.0.0.1:8080`. Trunk will rebuild automatically if you edit the project.
3. Open `http://127.0.0.1:8080/index.html#dev` in a browser. See the warning below.

> `assets/sw.js` script will try to cache our VDS app, and loads the cached version when it cannot connect to server allowing VDS to work offline (like PWA).
> appending `#dev` to `index.html` will skip this caching, allowing to load the latest builds during development.

### Web Deploy
1. Just run `trunk build --release`.
2. It will generate a `dist` directory as a "static html" website
3. Upload the `dist` directory to any of the numerous free hosting websites including [GitHub Pages](https://docs.github.com/en/free-pro-team@latest/github/working-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site).
4. This repo already provides a [workflow](.github/workflows/deploy_github_pages.yml) that auto-deploys VDS to GitHub pages.

### Building for native
Run it locally with `cargo run --release`.

### Building for web
[trunk](https://trunkrs.dev/) can be used to compile `vds` to WASM and spin up a web server with hot reloading. Run it locally with `trunk serve`.

Afterwards VDS can be viewed in a browser locally (http://127.0.0.1:8080/index.html). If you want to disable local caching, you can add '#dev' to the URL (http://127.0.0.1:8080/index.html#dev).

