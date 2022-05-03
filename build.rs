fn main() {
    #[cfg(target_env = "msvc")]
    embed_resource::compile("icons.rc")
}
