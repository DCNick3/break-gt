use include_dir::Dir;
pub static STATIC_DIST_DIR: Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/../frontend/dist");
