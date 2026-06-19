use slint_build::compile;

fn main() {
    compile("src/bin/client/ui.slint").expect("Couldn't compile the UI!");
}
