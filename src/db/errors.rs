use burrow_db::define_errors;

define_errors!(
    PostError {
        TokenNotFound => "P0000",
        PostNotFound => "P0001"
    }
);
