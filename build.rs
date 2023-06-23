// Necessary because of this issue: https://github.com/rust-lang/cargo/issues/9641
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(feature = "target-esp32-none-elf") {
        embuild::build::CfgArgs::output_propagated("ESP_IDF")?;
        embuild::build::LinkArgs::output_propagated("ESP_IDF")?;
    }
    Ok(())
}
