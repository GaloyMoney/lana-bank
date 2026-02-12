fn main() {
    println!("{}", customer_server::graphql::schema(None).sdl().trim());
}
