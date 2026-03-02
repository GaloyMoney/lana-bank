fn main() {
    println!(
        "{}",
        admin_server::graphql::schema(None, Default::default())
            .sdl()
            .trim()
    );
}
