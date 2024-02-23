#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(missing_debug_implementations)]
#![cfg_attr(test, deny(rust_2018_idioms))]

//! # dynamic_object
//!
//! `dynamic_object` provides an easy way to create dynamic, type-erased, key-value maps in Rust.
//! It allows you to store any value that implements the `Any` trait and retrieve it with type checking at runtime.
//! This is particularly useful when you need a map to hold values of different types and you can't determine the types at compile time.
//!
//! It also provides a macro for easy and intuitive object creation.
//!
//! ## Features
//!
//! - Dynamic key-value map
//! - Type checking at runtime
//! - Macro for easy and intuitive object creation
//!
//! ## Usage
//!
//! Add `dynamic_object` to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! dynamic_object = { git = "https://github.com/trvswgnr/dynamic_object" }
//! ```
//!
//! You can create an `Object` and insert any type that implements `Any`:
//!
//! ```rust
//! use dynamic_object::Object;
//! let mut object = Object::new();
//! object.insert("key", "value");
//! ```
//!
//! You can retrieve a reference to the original value if it is of the correct type:
//!
//! ```rust
//! use dynamic_object::Object;
//! let mut object = Object::new();
//! object.insert("key", "value");
//! let value = object.get_as::<&str>("key");
//! assert_eq!(value, Some(&"value"));
//! ```
//!
//! If it isn't of the correct type, you will get `None`.
//!
//! ## Object Macro
//!
//! The `object` macro provided by this crate makes it easy to create `Object` instances.
//! It supports nested objects.
//!
//! ```rust
//! use dynamic_object::object;
//! let obj = object!({
//!     key1: "value1",
//!     key2: {
//!         inner_key: "inner_value",
//!     },
//! });
//! assert_eq!(obj.get_as::<&str>("key1"), Some(&"value1"));
//! ```
//!
//! ## Repository
//!
//! The source code for `dynamic_object` is available on GitHub at [github.com/trvswgnr/dynamic_object](https://github.com/trvswgnr/dynamic_object).
//!
//! ## Contributions
//!
//! Contributions to `dynamic_object` are welcome! Please submit a pull request on GitHub.
//!
//! ## License
//!
//! `dynamic_object` is licensed under the MIT license. Please see the `LICENSE` file in the GitHub repository for more information.

use std::{
    any::Any,
    cmp::Ordering,
    collections::BTreeMap,
    ops::{Deref, DerefMut},
    fmt::{self, Debug, Formatter},
};

/// Creates a new `Object`.
///
/// # Examples
///
/// ```
/// use dynamic_object::object;
/// let obj = object!({
///     key1: "value1",
///     key2: {
///         inner_key: "inner_value",
///     },
/// });
/// assert_eq!(obj.get_as::<&str>("key1"), Some(&"value1"));
/// ```
///
/// This will create an `Object` with two keys: "key1" and "key2". "key1" maps to the string "value1",
/// and "key2" maps to another `Object` with a single key "inner_key" that maps to "inner_value".
///
/// # Syntax
///
/// The `object` macro has two forms:
///
/// - `object!({})` creates an empty `Object`.
/// - `object!({ key: value, ... })` creates an `Object` with the given keys and values.
///   The keys must be identifiers, and the values can be any expression.
///   If a value is surrounded by `{}` it is treated as another `Object`.
///
/// # Representation
///
/// The `object` macro is expanded to a series of `insert` calls on a new `Object`. The keys are
/// stringified and the values are inserted into the `Object` using the `insert` method.
#[macro_export]
macro_rules! object {
    ({}) => {
        $crate::Object::new()
    };
    ({
        $key:ident: { $($inner:tt)* }, $($rest:tt)*
    }) => {
        {
            let mut map = object!({ $($rest)* });
            map.insert(stringify!($key), $crate::object!({ $($inner)* }));
            map
        }
    };
    ({
        $key:ident: $value:expr, $($rest:tt)*
    }) => {
        {
            let mut map = $crate::object!({ $($rest)* });
            map.insert(stringify!($key), $value);
            map
        }
    };
}

/// A type-erased value.
pub trait AnyType: Any {
    /// Upcast to `Any`.
    fn as_any(&self) -> &dyn Any;

    /// Upcast to `Any` mutably.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Compare with another type-erased value.
    fn dyn_cmp(&self, other: &dyn AnyType) -> Option<Ordering>;

    /// Write the `Debug` representation.
    fn dyn_debug(&self, f: &mut Formatter<'_>) -> fmt::Result;
}

impl<T: Any + Debug + PartialOrd> AnyType for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn dyn_cmp(&self, other: &dyn AnyType) -> Option<Ordering> {
        other
            .as_any()
            .downcast_ref::<T>()
            .and_then(|other| self.partial_cmp(other))
    }

    fn dyn_debug(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl dyn AnyType + '_ {
    /// Convenience method.
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }

    /// Convenience method.
    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.as_any_mut().downcast_mut::<T>()
    }
}

impl PartialOrd for dyn AnyType + '_ {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.dyn_cmp(other)
    }
}

impl PartialEq for dyn AnyType + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other).is_some_and(Ordering::is_eq)
    }
}

impl Debug for dyn AnyType + '_ {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.dyn_debug(f)
    }
}

/// A type-erased key-value map.
///
/// The `Object` struct is a wrapper around a `BTreeMap` that allows storing any value that implements the `Any` trait.
/// It provides methods for inserting and retrieving values, with type checking at runtime.
///
/// # Examples
///
/// You can create an `Object` and insert any type that implements `Any`:
///
/// ```
/// use dynamic_object::Object;
/// let mut object = Object::new();
/// object.insert("key", "value");
/// ```
///
/// You can retrieve a reference to the original value if it is of the correct type:
///
/// ```
/// use dynamic_object::Object;
/// let mut object = Object::new();
/// object.insert("key", "value");
/// let value = object.get_as::<&str>("key");
/// assert_eq!(value, Some(&"value"));
/// ```
///
/// If it isn't of the correct type, you will get `None`.
///
/// # Downcasting
///
/// The `get_as` and `get_or_insert_as` methods attempt to downcast the value to the correct type.
/// If the value is not of the correct type, these methods will return `None`.
///
/// [Any]: core::any::Any "any::Any"
/// [AnyType]: crate::AnyType "AnyType"
/// [BTreeMap]: std::collections::BTreeMap "collections::BTreeMap"
#[derive(Default, Debug, PartialEq, PartialOrd)]
pub struct Object {
    map: BTreeMap<String, Box<dyn AnyType>>,
}

impl Object {
    /// Creates a new `Object`.
    ///
    /// # Examples
    ///
    /// ```
    /// use dynamic_object::Object;
    /// let object = Object::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a key-value pair into the `Object`.
    ///
    /// # Examples
    ///
    /// ```
    /// use dynamic_object::Object;
    /// let mut object = Object::new();
    /// object.insert("key", "value");
    /// ```
    pub fn insert<K: Into<String>, V: AnyType>(&mut self, key: K, value: V) {
        self.map.insert(key.into(), Box::new(value));
    }

    /// Returns a reference to the value corresponding to the key if it is of type `T`, or `None` if it isn't.
    ///
    /// If the value is not of type `T`, this method will return `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use dynamic_object::Object;
    /// let mut object = Object::new();
    /// object.insert("key", "value");
    /// let value = object.get_as::<&str>("key");
    /// assert_eq!(value, Some(&"value"));
    /// ```
    pub fn get_as<T: 'static>(&self, key: &str) -> Option<&T> {
        self.map.get(key).and_then(|v| (**v).as_any().downcast_ref::<T>())
    }

    /// Returns a reference to the value corresponding to the key if it is of type `T`, or inserts it if it doesn't exist.
    ///
    /// If the value is not of type `T`, this method will return `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use dynamic_object::Object;
    /// let mut object = Object::new();
    /// let value = object.get_or_insert_as("key", "value");
    /// assert_eq!(value, Some(&mut "value"));
    /// ```
    pub fn get_or_insert_as<T: AnyType>(&mut self, key: impl Into<String>, value: T) -> Option<&mut T> {
        let bx = self.map.entry(key.into()).or_insert_with(|| Box::new(value));

        (**bx).as_any_mut().downcast_mut::<T>()
    }
}

impl Deref for Object {
    type Target = BTreeMap<String, Box<dyn AnyType>>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl DerefMut for Object {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, PartialOrd)]
    struct Foo {
        bar: i32,
    }

    #[test]
    fn works_with_empty_object() {
        let mut empty = object!({});
        assert_eq!(empty.get_as::<&str>("foo"), None);
        empty.insert("foo", "bar");
        assert_eq!(empty.get_as::<&str>("foo"), Some(&"bar"));
    }

    #[test]
    fn works_with_simple_object() {
        let x = object!({
            foo: Foo { bar: 123 },
            bar: 123,
        });
        assert_eq!(x.get_as::<Foo>("foo"), Some(&Foo { bar: 123 }));
        assert_eq!(x.get_as::<i32>("bar"), Some(&123));
    }

    #[test]
    fn works_with_nested_object() {
        let y = object!({
            foo: "fpp",
            bar: 123,
            baz: {
                inner: "value",
            },
        });
        assert_eq!(y.get_as::<&str>("foo"), Some(&"fpp"));
        assert_eq!(y.get_as::<i32>("bar"), Some(&123));
        let baz = y.get_as::<Object>("baz");
        assert!(baz.is_some());
        let baz_inner = baz.and_then(|v| v.get_as::<&str>("inner"));
        assert!(baz_inner.is_some());
        assert_eq!(baz_inner, Some(&"value"));
    }

    #[test]
    fn works_with_deeply_nested_object() {
        let z = object!({
            a: "xyz",
            b: 69,
            c: {
                inner: "value",
                another: {
                    inner: 420,
                    d: "tr4vvyr00lz",
                },
            },
        });
        assert_eq!(z.get_as::<&str>("a"), Some(&"xyz"));
        assert_eq!(z.get_as::<i32>("b"), Some(&69));
        let c = z.get_as::<Object>("c");
        assert!(c.is_some());
        let c_inner = c.and_then(|v| v.get_as::<&str>("inner"));
        assert_eq!(c_inner, Some(&"value"));
        let c_inner_another = c.and_then(|v| v.get_as::<Object>("another"));
        assert!(c_inner_another.is_some());
        let c_inner_another_inner = c_inner_another.and_then(|v| v.get_as::<i32>("inner"));
        assert_eq!(c_inner_another_inner, Some(&420));
        let c_inner_another_d = c_inner_another.and_then(|v| v.get_as::<&str>("d"));
        assert_eq!(c_inner_another_d, Some(&"tr4vvyr00lz"));
    }

    #[test]
    fn works_with_mixed_object() {
        let mixed = object!({
            a: "xyz",
            b: 69,
            c: {
                inner: "value",
                another: {
                    inner: 420,
                    d: "tr4vvyr00lz",
                },
            },
            d: Foo { bar: 123 },
        });
        assert_eq!(mixed.get_as::<&str>("a"), Some(&"xyz"));
        assert_eq!(mixed.get_as::<i32>("b"), Some(&69));
        let c = mixed.get_as::<Object>("c");
        assert!(c.is_some());
        let c_inner = c.and_then(|v| v.get_as::<&str>("inner"));
        assert_eq!(c_inner, Some(&"value"));
        let c_inner_another = c.and_then(|v| v.get_as::<Object>("another"));
        assert!(c_inner_another.is_some());
        let c_inner_another_inner = c_inner_another.and_then(|v| v.get_as::<i32>("inner"));
        assert_eq!(c_inner_another_inner, Some(&420));
        let c_inner_another_d = c_inner_another.and_then(|v| v.get_as::<&str>("d"));
        assert_eq!(c_inner_another_d, Some(&"tr4vvyr00lz"));
        assert_eq!(mixed.get_as::<Foo>("d"), Some(&Foo { bar: 123 }));
    }

    #[test]
    fn works_like_a_map() {
        let mut map = object!({});
        assert!(map.get("foo").is_none());
        map.insert("foo", "bar");
        map.get_or_insert_as::<&str>("foo", "lol");
        assert_eq!(
            map.get("foo").and_then(|v| v.downcast_ref::<&str>()),
            Some(&"bar")
        );
        map.get_or_insert_as::<&str>("bar", "baz");
        assert_eq!(
            map.get("bar").and_then(|v| v.downcast_ref::<&str>()),
            Some(&"baz")
        );
    }
}
