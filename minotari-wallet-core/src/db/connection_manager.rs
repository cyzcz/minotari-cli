//! r2d2 connection manager for a SQLite database file.

use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};

type InitFn = fn(&mut Connection) -> rusqlite::Result<()>;

#[derive(Debug)]
pub struct SqliteConnectionManager {
    path: PathBuf,
    init: Option<InitFn>,
}

impl SqliteConnectionManager {
    pub fn file<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            init: None,
        }
    }

    /// Runs `init` on every new connection before the pool hands it out.
    pub fn with_init(self, init: InitFn) -> Self {
        Self {
            init: Some(init),
            ..self
        }
    }
}

impl r2d2::ManageConnection for SqliteConnectionManager {
    type Connection = Connection;
    type Error = rusqlite::Error;

    fn connect(&self) -> rusqlite::Result<Connection> {
        let mut conn = Connection::open_with_flags(&self.path, OpenFlags::default())?;
        if let Some(init) = self.init {
            init(&mut conn)?;
        }
        Ok(conn)
    }

    fn is_valid(&self, conn: &mut Connection) -> rusqlite::Result<()> {
        conn.execute_batch("SELECT 1;")
    }

    fn has_broken(&self, _: &mut Connection) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use r2d2::ManageConnection;

    #[test]
    fn connect_runs_init_and_connection_is_valid() {
        let dir = tempfile::tempdir().unwrap();
        let manager = SqliteConnectionManager::file(dir.path().join("test.db"))
            .with_init(|c| c.execute_batch("PRAGMA foreign_keys = ON;"));

        let mut conn = manager.connect().unwrap();
        let foreign_keys: i64 = conn.query_row("PRAGMA foreign_keys", [], |row| row.get(0)).unwrap();

        assert_eq!(foreign_keys, 1);
        manager.is_valid(&mut conn).unwrap();
        assert!(dir.path().join("test.db").exists());
    }
}
