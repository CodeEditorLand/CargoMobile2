#[cfg(not(feature = "cli"))]
fn main() {}

#[cfg(feature = "cli")]
fn main() {
	use std::{path::PathBuf, process::Command};
	#[path = "Source/bicycle/mod.rs"]
	mod bicycle;

	let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

	use std::path::Path;

	let dir_name = concat!(".", env!("CARGO_PKG_NAME"));

	let install_dir = std::env::var("CARGO_HOME")
		.map(|p| PathBuf::from(p).join(dir_name))
		.unwrap_or_else(|_| {
			home::home_dir()
				.map(|home| home.join(".cargo").join(dir_name))
				.expect("failed to get user's home dir")
		});

	std::fs::create_dir_all(&install_dir).expect("failed to create install dir");

	// Copy version info
	match Command::new("git")
		.arg("-C")
		.arg(&manifest_dir)
		.args(["log", "-1", "--pretty=%s"])
		.output()
	{
		Ok(output) => {
			let msg = String::from_utf8_lossy(&output.stdout).to_string();

    #[cfg(windows)]
    {
        // Embed application manifest
        let resource_path = manifest_dir.join("cargo-mobile-manifest.rc");
        let manifest_path = manifest_dir.join("cargo-mobile.exe.manifest");
        println!("cargo:rerun-if-changed={}", resource_path.display());
        println!("cargo:rerun-if-changed={}", manifest_path.display());
        embed_resource::compile("cargo-mobile-manifest.rc", embed_resource::NONE)
            .manifest_optional()
            .unwrap();
    }
}
