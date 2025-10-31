use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};

fn main() {
    // 1. Create Vulkan instance
    let instance = Instance::new(InstanceCreateInfo::default())
        .expect("failed to create instance");

    // 2. Pick a physical device
    let physical = PhysicalDevice::enumerate(&instance)
        .filter(|p| p.properties().device_type == PhysicalDeviceType::DiscreteGpu)
        .next()
        .unwrap_or_else(|| PhysicalDevice::enumerate(&instance).next().unwrap());

    println!("Using device: {} (type: {:?})", 
        physical.properties().device_name, 
        physical.properties().device_type
    );

    // 3. Create a logical device and a queue
    let queue_family = physical.queue_families()
        .find(|&q| q.supports_graphics())
        .expect("couldn't find a graphics queue family");

    let (device, mut queues) = Device::new(
        physical,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
            ..Default::default()
        }
    ).expect("failed to create device");

    let _queue = queues.next().unwrap();

    println!("Minimal Vulkan setup done!");
}
