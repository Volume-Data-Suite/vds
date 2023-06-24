use vds_rust::run;

fn main() {
    // might cause issues with wasm
    // https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/#state-new
    pollster::block_on(run());
}

