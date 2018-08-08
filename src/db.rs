use {
    std::{
        borrow::{Cow},
        collections::{HashMap},
        fmt::{self, Formatter},
        sync::{Arc, RwLock}
    },
    serde::{
        de::{MapAccess, Visitor},
        ser::{SerializeMap},
        Deserialize, Serialize, Deserializer, Serializer
    },
    atomic::{AtomicU64, Atomic}
};

#[derive(Clone, Default)]
pub struct Db {
    map: Arc<RwLock<HashMap<String, ChannelDb>>>
}

#[derive(Clone, Default)]
pub struct ChannelDb {
    map: Arc<RwLock<HashMap<String, AtomicU64>>>
}

impl Db {
    pub fn channel_db<'a, I: Into<Cow<'a, str>>>(&self, key: I) -> ChannelDb {
        let key = key.into();

        if let Some(map) = self.map.read().unwrap().get(&*key) {
            return map.clone();
        }

        self.map.write().unwrap().entry(key.into_owned()).or_insert_with(Default::default).clone()
    }

    pub fn is_last(&self) -> bool {
        if (Arc::strong_count(&self.map), Arc::weak_count(&self.map)) != (1, 0) {
            return false;
        }

        for v in self.map.read().unwrap().values() {
            if (Arc::strong_count(&v.map), Arc::weak_count(&v.map)) != (1, 0) {
                return false;
            }
        }

        true
    }
}

impl<'de> Deserialize<'de> for Db {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct MapVisitor;

        impl<'de> Visitor<'de> for MapVisitor {
            type Value = HashMap<String, ChannelDb>;

            fn expecting(&self, fmt: &mut Formatter) -> fmt::Result {
                write!(fmt, "a map<String, ChannelDb>")
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map_access: M) -> Result<Self::Value, M::Error> {
                let mut map = HashMap::with_capacity(map_access.size_hint().unwrap_or(0));
                
                while let Some((key, value)) = map_access.next_entry()? {
                    map.insert(key, value);
                }

                Ok(map)
            }
        }

        deserializer.deserialize_map(MapVisitor).map(|map| Db { map: Arc::new(RwLock::new(map)) })
    }
}

impl Serialize for Db {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let map = self.map.read().unwrap();
        let mut serializer = serializer.serialize_map(Some(map.len()))?;

        for (key, value) in map.iter() {
            serializer.serialize_entry(key, value)?;
        }

        serializer.end()
    }
}

impl ChannelDb {
    pub fn get(&self, key: &str) -> u64 {
        self.map.read().unwrap().get(key).map(|p| p.get()).unwrap_or(0)
    }

    pub fn update_infallible<'a, K: Into<Cow<'a, str>>, F: Fn(u64, u64) -> u64>(&self, key: K, value: u64, f: F) -> u64 {
        let key = key.into();
        
        if value == 0 {
            return self.get(&key);
        }

        if let Some(points) = self.map.read().unwrap().get(&*key) {
            return points.update_infallible(value, f);
        }

        self.map.write().unwrap().entry(key.into_owned()).or_insert_with(Default::default).update_infallible(value, f)
    }

    pub fn update_fallible<'a, K: Into<Cow<'a, str>>, F: Fn(u64, u64) -> Option<u64>>(&self, key: K, value: u64, f: F) -> Option<u64> {
        let key = key.into();
        
        if value == 0 {
            return Some(self.get(&key));
        }

        if let Some(points) = self.map.read().unwrap().get(&*key) {
            return points.update_fallible(value, f);
        }

        self.map.write().unwrap().entry(key.into_owned()).or_insert_with(Default::default).update_fallible(value, f)
    }
}

impl<'de> Deserialize<'de> for ChannelDb {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct MapVisitor;

        impl<'de> Visitor<'de> for MapVisitor {
            type Value = HashMap<String, AtomicU64>;

            fn expecting(&self, fmt: &mut Formatter) -> fmt::Result {
                write!(fmt, "a map<String, Points>")
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map_access: M) -> Result<Self::Value, M::Error> {
                let mut map = HashMap::with_capacity(map_access.size_hint().unwrap_or(0));
                
                while let Some((key, value)) = map_access.next_entry()? {
                    map.insert(key, value);
                }

                Ok(map)
            }
        }

        deserializer.deserialize_map(MapVisitor).map(|map| ChannelDb { map: Arc::new(RwLock::new(map)) })
    }
}

impl Serialize for ChannelDb {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let map = self.map.read().unwrap();
        let mut serializer = serializer.serialize_map(Some(map.len()))?;

        for (key, value) in map.iter() {
            serializer.serialize_entry(key, &value.get())?;
        }

        serializer.end()
    }
}
