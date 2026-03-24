use wgpu::{Backends, DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits};

fn main() {
    env_logger::init();
    pollster::block_on(run());
}

async fn run() {
    let instance = Instance::new(InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            backend_options: wgpu::BackendOptions::default(),
            flags: Default::default(),
            display: Default::default(),
            memory_budget_thresholds: Default::default(),
        });

    let adapter = instance
        .enumerate_adapters(Backends::VULKAN)
        .await.into_iter()
        .find(|a| a.get_info().device == 49374)
        .unwrap_or_else(|| {
            panic!("Could not find the SwiftShader adapter! Make sure SwiftShader is in your Vulkan ICD path.");
        });

    println!("Found Adapter: {:?}", adapter.get_info());

    let features = adapter.features();
    if !features.contains(Features::SUBGROUP) {
        println!("Adapter does not report SUBGROUP support. Cannot trigger the bug.");
        return;
    }

    let (device, _queue) = adapter
        .request_device(
            &DeviceDescriptor {
                required_limits: Limits {
                    // This is needed to support swiftshader
                    max_storage_textures_per_shader_stage: 4,
                    ..Default::default()
                },
                required_features: Features::SUBGROUP,
                ..Default::default()
            }
        )
        .await
        .expect("Failed to create device");

    println!("Device created successfully with SUBGROUP feature.");

    let shader_source = r#"
        enable subgroups;

        @group(0) @binding(0) var<storage, read_write> data: array<u32>;

        @compute @workgroup_size(64)
        fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
            let sum = subgroupAdd(data[global_id.x]);
            data[global_id.x] = sum;
        }
    "#;

    println!("Compiling shader module...");
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Repro Shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    println!("Creating compute pipeline (this should panic instead of returning an error)...");
    
    let _pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Repro Pipeline"),
        layout: None,
        module: &module,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });

    println!("Bug not reproduced");
}