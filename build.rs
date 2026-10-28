fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS");
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV");

    if target_os.as_deref() == Ok("windows") {
        // A level change loads the destination while the viewer and outgoing
        // scene are still live. Windows' 1 MiB default exhausts the main thread
        // stack in that path; reserve 8 MiB, with pages committed on demand.
        // RUST_MIN_STACK only affects spawned threads, not the window thread.
        let arg = if target_env.as_deref() == Ok("msvc") {
            "/STACK:8388608"
        } else {
            "-Wl,--stack,8388608"
        };
        println!("cargo:rustc-link-arg={arg}");
    } else if target_os.as_deref() == Ok("linux") {
        // Linux default stack is 8 MiB but can be too small for deep recursion
        // in scene graph operations. Explicitly set 8 MiB for both GNU and musl.
        if target_env.as_deref() == Ok("gnu") || target_env.as_deref() == Ok("musl") {
            println!("cargo:rustc-link-arg=-Wl,-z,stack-size=8388608");
        }
    }
}
