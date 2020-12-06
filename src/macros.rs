macro_rules! pub_fields {
    (
        $(#[$deri:meta])*
             struct $name:ident {
                 $($field:ident: $t:ty,)*
             }
    ) => {
        $(#[$deri])*
        pub struct $name {
            $(pub $field: $t),*
        }
    }
}
