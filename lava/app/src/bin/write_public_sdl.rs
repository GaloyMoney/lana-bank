fn main() {
    println!(
        "{}",
        lava_app::server::public::graphql::schema(None).sdl().trim()
    );
}
