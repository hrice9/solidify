mod window;
mod surface;

fn main() {
    println!("Hello, world!");
    // window::run();
    pollster::block_on(window::run());
}
