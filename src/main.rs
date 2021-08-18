
async fn run() {
    let adapter = wgpu::Instance::new(wgpu::BackendBit::PRIMARY)
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .unwrap();

    println!("{:?}", adapter.get_info())
}

fn main() {
    env_logger::init();
    pollster::block_on(run());
}
