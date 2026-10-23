fn main() {
    texpio_lib::bindings::export().expect("could not export public bindings");
}
