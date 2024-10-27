fn main() {
    println!(
        "{}",
        lava_app::server::admin::graphql::schema(None).sdl().trim()
    );
}
