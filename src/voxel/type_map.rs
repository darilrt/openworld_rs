#[derive(Default, Clone, Debug)]
pub struct Map<T, V>(Vec<(T, V)>);

impl<T, V> Map<T, V>
where
    T: PartialEq,
{
    pub fn new() -> Self {
        Self(Vec::new())
    }

    #[allow(dead_code)]
    pub fn get(&self, key: &T) -> Option<&V> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn get_mut(&mut self, key: &T) -> Option<&mut V> {
        self.0.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn contains_key(&self, key: &T) -> bool {
        self.0.iter().any(|(k, _)| k == key)
    }

    pub fn insert(&mut self, key: T, value: V) -> Option<V> {
        if let Some((_, v)) = self.0.iter_mut().find(|(k, _)| k == &key) {
            Some(std::mem::replace(v, value))
        } else {
            self.0.push((key, value));
            None
        }
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, key: &T) -> Option<V> {
        if let Some(index) = self.0.iter().position(|(k, _)| k == key) {
            Some(self.0.remove(index).1)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> impl Iterator<Item = (&T, &V)> {
        self.0.iter().map(|(k, v)| (k, v))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
