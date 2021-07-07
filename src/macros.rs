/*!
Macros

For working with
    - `postgres`
*/

// -------------
// postgres
// -------------

/// Attempts to execute an `insert`, using provided and returned columns
/// to return a populated instance of the associated model `struct`.
///
/// Returns a `Result` containing the given model
///
/// # macro syntax
///
/// ```rust,ignore
/// try_query_to_model!(
///     query-expr-to-execute ;
///     model-type ;
///     struct-field: row-index, * ;
///     struct-field: value, *
/// )
/// ```
///
/// # Example
///
/// ```rust,ignore
/// impl NewPaste {
///     fn create(self, conn: &Connection) -> Result<Paste> {
///         let stmt = "insert into pastes (key, content, content_type) values ($1, $2, $3) \
///                     returning id, date_created, date_viewed";
///         try_query_to_model!(conn.query(stmt, &[&self.key, &self.content, &self.content_type]) ;
///                              Paste ;
///                              id: 0, date_created: 1, date_viewed: 2 ;
///                              key: self.key, content: self.content, content_type: self.content_type)
///     }
/// }
/// ```
macro_rules! try_query_to_model {
    ($query:expr ;
     $model:ident ;
     $($rowvar:ident : $rowindex:expr),* ;
     $($var:ident : $arg:expr),*) => {
        match $query {
            Ok(rows) => {
                match rows.iter().next() {
                    Some(row) => Ok($model {
                        $(
                            $rowvar: row.get($rowindex),
                         )*
                        $(
                            $var : $arg,
                         )*
                    }),
                    None =>
                        return Err(
                            error::helpers::does_not_exist(
                                format!("{} not found", $model::table_name())
                            )
                        )
                }
            }
            Err(e) => {
                Err(error::Error::from(e))
            }
        }
    }
}

/// Convert all rows returned into the associated model type
/// and collect them in a `Vec`
///
/// Returns a `Result<Vec<T>>` containing the given model
///
/// # Example
///
/// ```rust,ignore
/// fn find_all(text: &str, conn: &Connection) -> Result<Vec<Paste>> {
///     let stmt = "select * from pastes where content like '%' || $1 || '%'";
///     try_query_vec!(conn.query(stmt, &[&text]), Paste)
/// }
/// ```
macro_rules! try_query_vec {
    ($query:expr, $model:ident) => {
        match $query {
            Err(e) => Err(Error::from(e)),
            Ok(rows) => Ok(rows.iter().map($model::from_row).collect()),
        }
    };
}

/// Takes the first row returned and converts it into the
/// associated model type.
///
/// Returns a `Result<T>` containing the given model
///
/// Errors:
/// - If more than one row is returned, returns `ErrorKind::MultipleRecords`
/// - If no rows are returned, returns `ErrorKind::DoesNotExist`
///
/// # Example
///
/// ```rust,ignore
/// fn touch_and_get(key: &str, conn: &Connection) -> Result<Paste> {
///     let stmt = "update pastes set date_viewed = NOW() where key = $1 \
///                 returning id, key, content, content_type, date_created, date_viewed";
///     try_query_one!(conn.query(stmt, &[&key]), Paste)
/// }
/// ```
macro_rules! try_query_one {
    ($query:expr, $model:ident) => {
        match $query {
            Err(e) => Err(error::Error::from(e)),
            Ok(rows) => {
                let mut rows = rows.iter();
                let record = match rows.next() {
                    None => {
                        return Err(error::helpers::does_not_exist(format!(
                            "{} not found",
                            $model::table_name()
                        )))
                    }
                    Some(row) => Ok($model::from_row(row)),
                };
                match rows.next() {
                    None => record,
                    Some(_) => {
                        return Err(error::helpers::multiple_records(format!(
                            "Multiple rows returned from table: {}, expected one",
                            $model::table_name()
                        )))
                    }
                }
            }
        }
    };
}

/// Attempts to execute some statement that returns a single row
/// of some `type` that implements `FromSql`
///
/// # Example
///
/// ```rust,ignore
/// fn exists(conn: &Connection, key: &str) -> Result<bool> {
///     let stmt = "select exists(select 1 from pastes where key = $1)";
///     try_query_aggregate!(conn.query(stmt, &[&key]), bool)
/// }
/// ```
macro_rules! try_query_aggregate {
    ($query:expr, $row_type:ty) => {
        match $query {
            Err(e) => Err(Error::from(e)),
            Ok(rows) => match rows.iter().next() {
                None => Err(error::helpers::does_not_exist("Record not found")),
                Some(row) => {
                    let val: $row_type = row.get(0);
                    Ok(val)
                }
            },
        }
    };
}
