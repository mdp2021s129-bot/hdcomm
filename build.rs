fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/hdcomm.proto");
    tonic_build::compile_protos("proto/hdcomm.proto")?;
    Ok(())
}
