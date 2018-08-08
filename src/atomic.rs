use {
    std::{
        fmt::{self, Formatter},
        sync::{
            atomic::{AtomicUsize, Ordering}
        }
    },
    serde::{
        de::{self, Visitor},
        Deserialize, Serialize, Deserializer, Serializer
    }
};

#[cfg(target_pointer_width = "64")]
#[derive(Default)]
pub struct AtomicU64(AtomicUsize);

#[cfg(not(target_pointer_width = "64"))]
pub struct AtomicU64(compile_error!("AtomicUsize is not 64 bit"));

pub trait Atomic {
    type Value;

    fn get(self) -> Self::Value;
    fn update_infallible<F: Fn(Self::Value, Self::Value) -> Self::Value>(self, Self::Value, F) -> Self::Value;
    fn update_fallible<F: Fn(Self::Value, Self::Value) -> Option<Self::Value>>(self, Self::Value, F) -> Option<Self::Value>;
}

impl<'a> Atomic for &'a AtomicU64 {
    type Value = u64;

    fn get(self) -> Self::Value {
        self.0.load(Ordering::Relaxed) as _
    }

    fn update_infallible<F: Fn(Self::Value, Self::Value) -> Self::Value>(self, value: Self::Value, f: F) -> Self::Value {
        let mut old = self.0.load(Ordering::Relaxed);

        loop {
            let new = f(old as _, value);

            match self.0.compare_exchange_weak(old, new as _, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => { return new; }
                Err(new) => { old = new; }
            }
        }
    }

    fn update_fallible<F: Fn(Self::Value, Self::Value) -> Option<Self::Value>>(self, value: Self::Value, f: F) -> Option<Self::Value> {
        let mut old = self.0.load(Ordering::Relaxed);

        while let Some(new) = f(old as _, value) {
            match self.0.compare_exchange_weak(old, new as _, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => { return Some(new); }
                Err(new) => { old = new; }
            }
        }

        None
    }
}

impl<'a> Atomic for &'a mut AtomicU64 {
    type Value = u64;

    fn get(self) -> Self::Value {
        *self.0.get_mut() as _
    }

    fn update_infallible<F: Fn(Self::Value, Self::Value) -> Self::Value>(self, value: Self::Value, f: F) -> Self::Value {
        let new = f(*self.0.get_mut() as _, value);
        *self.0.get_mut() = new as _;
        new
    }

    fn update_fallible<F: Fn(Self::Value, Self::Value) -> Option<Self::Value>>(self, value: Self::Value, f: F) -> Option<Self::Value> {     
        match f(*self.0.get_mut() as _, value) {
            Some(new) => {
                *self.0.get_mut() = new as _;
                Some(new)
            }
            None => { None }
        }
    }
}

impl<'de> Deserialize<'de> for AtomicU64 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct U64Visitor;

        impl<'de> Visitor<'de> for U64Visitor {
            type Value = u64;

            fn expecting(&self, fmt: &mut Formatter) -> fmt::Result {
                write!(fmt, "an u64")
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(v)
            }
        }

        deserializer.deserialize_u64(U64Visitor).map(Into::into)
    }
}

impl From<u64> for AtomicU64 {
    fn from(v: u64) -> Self {
        AtomicU64(AtomicUsize::new(v as _))
    }
}

impl Serialize for AtomicU64 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u64(self.get())
    }
}
