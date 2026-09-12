fn main() {
    // No modes, file/network/credential access or identity mutations.
    // Fixed stdout marker is discarded by the all-null adapter in stage one.
    println!("PF27_SYNTHETIC_HOLD_READY");
    loop {
        std::thread::park();
    }
}
