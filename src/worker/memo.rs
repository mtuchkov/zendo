//! A memo of values that need IO, such as whether a file exists or the branch of a checkout: the UI
//! thread reads the last value from memory, and a worker computes again the keys that were asked in
//! the last seconds (IO-1, IO-6).
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, Weak};
use std::time::{Duration, Instant};

/// A key that nobody asked for this long goes.
const FORGET: Duration = Duration::from_secs(10);

struct Entry<V> {
    value: Option<V>,
    asked: Instant,
    /// When the key came.
    since: Instant,
}

struct Inner<K, V> {
    map: Mutex<HashMap<K, Entry<V>>>,
    /// A new key: the worker computes it at once.
    new: Condvar,
    /// The time between two rounds, in ms.
    every: AtomicU64,
    /// The rounds that the worker ended: a test waits for them.
    rounds: AtomicU64,
}

impl<K, V> Inner<K, V> {
    fn lock(&self) -> MutexGuard<'_, HashMap<K, Entry<V>>> {
        self.map.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

pub struct Memo<K, V> {
    inner: Arc<Inner<K, V>>,
}

impl<K, V> Memo<K, V>
where
    K: Hash + Eq + Clone + Send + 'static,
    V: Clone + PartialEq + Send + 'static,
{
    /// A memo whose worker computes `f` for each asked key, at once for a new key and then each
    /// `every`. The lock is never held while `f` runs.
    pub fn new(f: fn(&K) -> V, every: Duration) -> Memo<K, V> {
        let ms = u64::try_from(every.as_millis()).unwrap_or(u64::MAX);
        let inner = Arc::new(Inner { map: Mutex::new(HashMap::new()), new: Condvar::new(), every: AtomicU64::new(ms), rounds: AtomicU64::new(0) });
        let weak = Arc::downgrade(&inner);
        let _ = std::thread::Builder::new().name("zendo-memo".into()).spawn(move || run(&weak, f));
        Memo { inner }
    }

    /// Compute each `every` from now on: a test makes it short.
    pub fn set_every(&self, every: Duration) {
        self.inner.every.store(u64::try_from(every.as_millis()).unwrap_or(u64::MAX), Ordering::Relaxed);
        self.inner.new.notify_one();
    }

    /// The rounds that the worker ended.
    pub fn rounds(&self) -> u64 {
        self.inner.rounds.load(Ordering::Relaxed)
    }

    /// How long `key` waits for its first value; None when it has one or was not asked.
    pub fn waited(&self, key: &K) -> Option<std::time::Duration> {
        self.inner.lock().get(key).filter(|e| e.value.is_none()).map(|e| e.since.elapsed())
    }

    /// Seed `key` with a value read before the first frame (IO-7); the worker keeps it fresh.
    pub fn put(&self, key: K, value: V) {
        self.inner.lock().insert(key, Entry { value: Some(value), asked: Instant::now(), since: Instant::now() });
    }

    /// The last value of `key`, from memory; None until the worker computed it.
    pub fn get(&self, key: &K) -> Option<V> {
        let mut map = self.inner.lock();
        match map.get_mut(key) {
            Some(e) => {
                e.asked = Instant::now();
                e.value.clone()
            }
            None => {
                map.insert(key.clone(), Entry { value: None, asked: Instant::now(), since: Instant::now() });
                self.inner.new.notify_one();
                None
            }
        }
    }
}

/// Compute the asked keys, forget the old ones; wake the main loop when a value changed. Ends when
/// the memo goes.
fn run<K, V>(weak: &Weak<Inner<K, V>>, f: fn(&K) -> V)
where
    K: Hash + Eq + Clone,
    V: Clone + PartialEq,
{
    loop {
        let Some(inner) = weak.upgrade() else { return };
        let keys: Vec<K> = {
            let mut map = inner.lock();
            map.retain(|_, e| e.asked.elapsed() < FORGET);
            if map.values().all(|e| e.value.is_some()) {
                let every = Duration::from_millis(inner.every.load(Ordering::Relaxed));
                map = inner.new.wait_timeout(map, every).map(|(m, _)| m).unwrap_or_else(|e| e.into_inner().0);
            }
            map.keys().cloned().collect()
        };
        drop(inner);
        let values: Vec<(K, V)> = keys.into_iter().map(|k| {
            let v = f(&k);
            (k, v)
        }).collect();
        let Some(inner) = weak.upgrade() else { return };
        let mut changed = false;
        let mut map = inner.lock();
        for (k, v) in values {
            if let Some(e) = map.get_mut(&k) && e.value.as_ref() != Some(&v) {
                e.value = Some(v);
                changed = true;
            }
        }
        drop(map);
        inner.rounds.fetch_add(1, Ordering::Relaxed);
        if changed {
            super::wake().notify_one();
        }
    }
}

#[cfg(test)]
#[path = "memo_tests.rs"]
mod tests;
