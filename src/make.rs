/// Trait that this structure can
/// be made without argument
pub trait Make<T> {
    /// Make a new structure
    ///
    /// Just like the well-known `new` method
    /// with different name to avoid name collision
    /// after rust reference coercion
    fn make() -> T;
}
