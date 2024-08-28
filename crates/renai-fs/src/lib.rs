/// Datbase object.
/// ```rust
/// let db = Database::new(".env");
/// db.fetch([
///     "crypto",
///     "economic",
///     "people",
///     "stocks",
/// ]).await
/// ```
pub mod db;
pub mod schema;
