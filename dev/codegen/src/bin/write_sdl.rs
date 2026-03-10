fn main() {
    use async_graphql::SDLExportOptions;

    println!(
        "{}",
        admin_server::graphql::schema(None, Default::default())
            .sdl_with_options(
                SDLExportOptions::new()
                    .sorted_fields()
                    .sorted_arguments()
                    .sorted_enum_items(),
            )
            .trim()
    );
}
