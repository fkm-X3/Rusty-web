// Minimal C++ launcher that calls into the Rust browser entry point.
// Build this after running `cargo build --release` so the Rust static library exists.

extern "C" int rusty_web_main();

int main() {
    return rusty_web_main();
}
