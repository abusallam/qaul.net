use crate::Identity;
use alexandria::{
    query::{Query, QueryResult, Subscription as Sub},
    record::RecordRef,
    Library, Session,
};
use async_std::sync::Arc;
use std::marker::PhantomData;

/// A unique, randomly generated subscriber ID
pub type SubId = Identity;

/// A generic subscription which can stream data from libqaul
pub struct Subscription<T>
where
    T: From<RecordRef>,
{
    store: Arc<Library>,
    session: Session,
    inner: Sub,
    _none: PhantomData<T>,
}

impl<T> Subscription<T>
where
    T: From<RecordRef>,
{
    pub(crate) fn new(store: &Arc<Library>, session: Session, inner: Sub) -> Self {
        Self {
            store: Arc::clone(store),
            session,
            inner,
            _none: PhantomData,
        }
    }

    /// Poll for the next return from the subscription
    pub async fn next(&self) -> Option<T> {
        let path = self.inner.next().await;
        match self
            .store
            .query(self.session, Query::path(path))
            .await
            .unwrap()
        {
            QueryResult::Single(rec) => Some(rec.into()),
            _ => None,
        }
    }
}