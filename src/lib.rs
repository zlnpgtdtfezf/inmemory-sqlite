use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use std::ops::Deref;

use rusqlite::*;

use thread_local::ThreadLocal;

static COUNTER: AtomicU64 = AtomicU64::new(0u64);

pub fn open_shared(name: &str) -> Result<Connection> {
    let uri = format!("file:{}?mode=memory&cache=shared", name);
    Connection::open(uri)
}

pub fn new_shared() -> Result<Connection> {
    open_shared(&format!(
        "shared_{}",
        COUNTER.fetch_add(1u64, Ordering::AcqRel)
    ))
}

pub struct SyncSqliteConnection {
    connection: ThreadLocal<Connection>,
    name: String,
}

impl SyncSqliteConnection {
    pub fn new() -> Self {
        let name = format!("shared_{}", COUNTER.fetch_add(1u64, Ordering::AcqRel));
        SyncSqliteConnection {
            connection: ThreadLocal::new(),
            name: name,
        }
    }

    pub fn open(name: String) -> Self {
        SyncSqliteConnection {
            connection: ThreadLocal::new(),
            name: name,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn force(&self) -> &Connection {
        self.connection.get_or(|| {
            open_shared(&self.name()).expect(
                "ERROR: Creating the connection to the sqlite in memory database has failed!",
            )
        })
    }
}

impl Deref for SyncSqliteConnection {
    type Target = Connection;
    fn deref(&self) -> &Self::Target {
        self.force()
    }
}

impl Clone for SyncSqliteConnection {
    fn clone(&self) -> Self {
        SyncSqliteConnection::open(self.name().clone())
    }

    fn clone_from(&mut self, source: &Self) {
        self.name = source.name().clone();
        self.connection.clear();
    }
}

mod test {

    #[test]
    fn testnew() {
        crate::SyncSqliteConnection::new();
    }

    #[test]
    fn testnewrealconnection() {
        let c = crate::SyncSqliteConnection::new();

        c.force();
    }

    #[test]
    fn test_open() {
        let dummy = crate::SyncSqliteConnection::new();
        dummy.force();

        let c1 = crate::SyncSqliteConnection::new();
        c1.force();

        let c2 = crate::SyncSqliteConnection::open(c1.name().clone());
        c2.force();

        assert_eq!(c1.name(), c2.name());
        assert_ne!(dummy.name(), c1.name());
    }

    #[test]
    fn test_clone() {
        let c1 = crate::SyncSqliteConnection::new();
        c1.force();

        let c2 = c1.clone();
        assert_eq!(c1.name(), c2.name());
    }
}
