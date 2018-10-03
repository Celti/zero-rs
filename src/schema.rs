table! {
    items (id) {
        id -> Int8,
        content -> Bytea,
        filename -> Text,
        mimetype -> Text,
        digest -> Text,
        label -> Text,
        destruct -> Bool,
        private -> Bool,
        is_url -> Bool,
        sunset -> Nullable<Timestamptz>,
        timestamp -> Nullable<Timestamptz>,
    }
}
