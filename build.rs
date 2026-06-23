use std::{collections::HashMap, env, path::PathBuf};

use slint_build::{CompilerConfiguration, compile_with_config};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let library_paths = HashMap::from([(
        "client_ui".to_string(),
        manifest_dir.join("src/ui/client-ui.slint"),
    )]);
    let config = CompilerConfiguration::new().with_library_paths(library_paths);
    compile_with_config("src/ui/main.slint", config).expect("Couldn't compile!");
}
