fn main() -> anyhow::Result<()> {
    let rt = runtime::Vm::new("target/wasm32-unknown-unknown/release/actors.wasm")?;
    rt.run()
}
