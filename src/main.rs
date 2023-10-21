mod window;
mod surface;
mod vertex;

fn main() {
    println!("Hello, world!");
    // window::run();
    pollster::block_on(window::run());
}
