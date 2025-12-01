use std::borrow::Cow;

trait Storage<K, V> {
    fn set(&mut self, key: K, val: V);
    fn get(&self, key: K) -> Option<&V>;
    fn remove(&mut self, key: K) -> Option<V>;
}

#[derive(Debug, Clone, PartialEq)]
struct User {
    id: u64,
    email: Cow<'static, str>,
    activated: bool,
}

// ---------- STATIC DISPATCH VERSION ----------
struct UserRepositoryStatic<S: Storage<u64, User>> {
    storage: S,
}

impl<S: Storage<u64, User>> UserRepositoryStatic<S> {
    fn new(storage: S) -> Self {
        Self { storage }
    }

    fn add_user(&mut self, user: User) {
        self.storage.set(user.id, user);
    }

    fn get_user(&self, id: u64) -> Option<&User> {
        self.storage.get(id)
    }

    fn update_user(&mut self, user: User) {
        self.storage.set(user.id, user);
    }

    fn remove_user(&mut self, id: u64) -> Option<User> {
        self.storage.remove(id)
    }
}

// ---------- DYNAMIC DISPATCH VERSION ----------
struct UserRepositoryDynamic {
    storage: Box<dyn Storage<u64, User>>,
}

impl UserRepositoryDynamic {
    fn new(storage: Box<dyn Storage<u64, User>>) -> Self {
        Self { storage }
    }

    fn add_user(&mut self, user: User) {
        self.storage.set(user.id, user);
    }

    fn get_user(&self, id: u64) -> Option<&User> {
        self.storage.get(id)
    }

    fn update_user(&mut self, user: User) {
        self.storage.set(user.id, user);
    }

    fn remove_user(&mut self, id: u64) -> Option<User> {
        self.storage.remove(id)
    }
}

// ---------- Simple in-memory storage ----------
#[derive(Default)]
struct MemoryStorage {
    data: std::collections::HashMap<u64, User>,
}

impl Storage<u64, User> for MemoryStorage {
    fn set(&mut self, key: u64, val: User) {
        self.data.insert(key, val);
    }

    fn get(&self, key: u64) -> Option<&User> {
        self.data.get(&key)
    }

    fn remove(&mut self, key: u64) -> Option<User> {
        self.data.remove(&key)
    }
}

// ---------- TESTS ----------
#[cfg(test)]
mod tests {
    use super::*;

    fn example_user(id: u64) -> User {
        User {
            id,
            email: "test@example.com".into(),
            activated: true,
        }
    }

    #[test]
    fn static_repo_works() {
        let storage = MemoryStorage::default();
        let mut repo = UserRepositoryStatic::new(storage);

        repo.add_user(example_user(1));
        assert!(repo.get_user(1).is_some());

        repo.remove_user(1);
        assert!(repo.get_user(1).is_none());
    }

    #[test]
    fn dynamic_repo_works() {
        let storage = Box::new(MemoryStorage::default());
        let mut repo = UserRepositoryDynamic::new(storage);

        repo.add_user(example_user(2));
        assert!(repo.get_user(2).is_some());
    }
}