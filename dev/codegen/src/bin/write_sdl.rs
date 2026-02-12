fn main() {
    println!("{}", admin_server::graphql::schema(None).sdl().trim());
}
