use cuda_builder::CudaBuilder;

fn main() {
    CudaBuilder::new("./gpu")
        .copy_to("target/gpu.ptx")
        .build()
        .unwrap();
}