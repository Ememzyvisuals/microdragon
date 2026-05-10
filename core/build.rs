// build.rs — compile proto files for gRPC daemon mode
fn main() {
    // Only compile protos if they exist
    let proto = "src/proto/microdragon.proto";
    if std::path::Path::new(proto).exists() {
        tonic_build::compile_protos(proto)
            .expect("Failed to compile proto");
    }
    println!("cargo:rerun-if-changed={}", proto);
    // Inject git hash at build time
    let hash = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "dev".to_string());
    println!("cargo:rustc-env=GIT_SHORT_SHA={}", hash);
}
