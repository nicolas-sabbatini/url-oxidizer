// @generated automatically by Diesel CLI.

diesel::table! {
    url_map (path) {
        path -> Text,
        url -> Text,
    }
}
