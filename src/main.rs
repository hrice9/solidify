mod window;
mod surface;
mod vertex;
mod camera;
mod model;
mod texture;

fn main() {
    println!("Hello, world!");
    // window::run();
    pollster::block_on(window::run());
}
