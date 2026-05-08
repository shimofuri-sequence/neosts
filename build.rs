#[cfg(windows)]
fn main() {
    let mut resource = winresource::WindowsResource::new();

    resource.set_icon("assets/neosts.ico");
    resource.set_version_info(
        winresource::VersionInfo::PRODUCTVERSION,
        windows_version_u64(),
    );
    resource.set_version_info(winresource::VersionInfo::FILEVERSION, windows_version_u64());

    if let Err(error) = resource.compile() {
        panic!("failed to compile Windows resources: {error}");
    }
}

#[cfg(not(windows))]
fn main() {}

#[cfg(windows)]
fn windows_version_u64() -> u64 {
    let major = env!("CARGO_PKG_VERSION_MAJOR").parse::<u64>().unwrap_or(0);
    let minor = env!("CARGO_PKG_VERSION_MINOR").parse::<u64>().unwrap_or(0);
    let patch = env!("CARGO_PKG_VERSION_PATCH").parse::<u64>().unwrap_or(0);
    let build = option_env!("CARGO_PKG_VERSION_PRE")
        .filter(|value| !value.is_empty())
        .and_then(|value| value.split('.').next())
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);

    (major << 48) | (minor << 32) | (patch << 16) | build
}
