extern crate version_check as rustc;

fn main() {
    if let Some((version, channel, _)) = rustc::triple() {
        if version.at_least("1.31.0") && channel.supports_features() {
            println!("cargo:rustc-cfg=pear_nightly");
        }
    }
}
