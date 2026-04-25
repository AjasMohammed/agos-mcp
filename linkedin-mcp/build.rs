fn main() {
    if std::env::var("LINKEDIN_EMBEDDED_CLIENT_ID").is_err() {
        println!("cargo:rustc-env=LINKEDIN_EMBEDDED_CLIENT_ID=");
    }
    println!("cargo:rerun-if-env-changed=LINKEDIN_EMBEDDED_CLIENT_ID");
}
