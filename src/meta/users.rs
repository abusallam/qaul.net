//! The library user table
//!
//! Users and scopes in Alexandria are secret.  They have a public Id, which get's opened with

use crate::{
    crypto::{
        aes::{Constructor, Key},
        asym::KeyPair,
        DetachedKey, Encrypted,
    },
    error::{Error, Result},
    Id,
};
use async_std::sync::Arc;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The hash of an Id which is used for external representation
pub(crate) type Hid = Id;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct User {
    /// The nested user token
    pub(crate) id: Id,
    /// The users' asymmetric encryption pair
    ///
    /// At the moment, this is used in a very symmetric way, but in
    /// the future there are ways to create zones and drop-in
    /// encryption.
    pub(crate) key: KeyPair,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserWithKey {
    #[serde(skip)]
    key: Option<Arc<Key>>,
    inner: User,
}

impl UserWithKey {
    fn new(pw: &str, id: Id) -> Self {
        let key = KeyPair::new();
        let aes_key = Arc::new(Key::from_pw(pw, &id.to_string()));

        Self {
            key: Some(aes_key),
            inner: User { id, key },
        }
    }
}

impl DetachedKey<Key> for UserWithKey {
    fn key(&self) -> Option<Arc<Key>> {
        self.key.as_ref().map(|key| Arc::clone(&key))
    }
}

/// A table of users in the database
#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct UserTable(BTreeMap<Hid, Encrypted<UserWithKey, Key>>);

impl UserTable {
    /// Create a new empty user table
    pub(crate) fn new() -> Self {
        Default::default()
    }

    /// Load data from disk
    pub(crate) fn load(data: &[u8]) -> Self {
        unimplemented!()
        // deserialize(data).unwrap()
    }

    /// Add a new user to the user table
    pub(crate) fn insert(&mut self, id: Id, pw: &str) -> Result<()> {
        if self.0.contains_key(&id) {
            return Err(Error::UserAlreadyExists);
        }
        let with_key = UserWithKey::new(pw, id);
        self.0.insert(id, Encrypted::new(with_key));
        Ok(())
    }

    /// Unlock a user entry in place
    ///
    /// The provided Id will be hashed, to corresponds to a `Hid`,
    /// which provides a layer of anonymity for users in the database.
    pub(crate) fn open(&mut self, id: Id, pw: &str) -> Result<()> {
        let k = Key::from_pw(pw, &id.to_string());
        match self.0.get_mut(&id) {
            Some(ref mut e) => e.open(&k),
            None => Err(Error::UnlockFailed { id: id.to_string() }),
        }
    }

    /// Re-seal the user metadata structure in place
    pub(crate) fn close(&mut self, id: Id) -> Result<()> {
        match self.0.get_mut(&id) {
            Some(e) => Ok(e.close_detached()?),
            None => Err(Error::UnlockFailed { id: id.to_string() }),
        }
    }
}

#[test]
fn open_empty() {
    let mut u = UserTable::new();
    assert_eq!(u.open(Id::random(), "cool_pw").is_err(), true);
}

#[test]
fn create_clone_open() {
    let mut u = UserTable::new();
    let id = Id::random();
    let pw = "car horse battery staple";
    u.insert(id, pw).unwrap();
    u.close(id).unwrap();
    u.open(id, pw).unwrap();
}
