#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![cfg_attr(test, deny(rust_2018_idioms))]

//! # dynamic_object
//!
//! `dynamic_object` is a library that provides a dynamic, type-erased, key-value map in Rust. It allows you to store any value that implements the `Any` trait and retrieve it with type checking at runtime. This is particularly useful when you need a map to hold values of different types and you can't determine the types at compile time.
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
//! dynamic_object = { git = "https://github.com/trvswgnr/dynamic-object" }
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
//! ## Safety
//!
//! The `AnyType` struct uses `Box::leak` to create a `'static` reference, which means the value will
//! live for the entire duration of the program. Therefore, it's safe to use and doesn't require the
//! user to manage the lifetime of the value.
//!
//! ## Repository
//!
//! The source code for `dynamic_object` is available on GitHub at [github.com/trvswgnr/dynamic-object](https://github.com/trvswgnr/dynamic-object).
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
    collections::HashMap,
    ops::{Deref, DerefMut},
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
/// # Safety
///
/// The `object` macro is safe to use, but the `AnyType` struct uses raw pointers to
/// store its value, which is inherently unsafe. The `AnyType` struct takes ownership of the value
/// and leaks it, so it will never be dropped.
///
/// # Representation
///
/// The `object` macro is expanded to a series of `insert` calls on a new `Object`. The keys are
/// stringified and the values are inserted into the `Object` using the `insert` method.
#[macro_export]
macro_rules! object {
    ({}) => {
        $crate::Object::default()
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
///
/// The `AnyType` struct is a type that includes any value that implements the `Any` trait.
/// It stores a raw pointer to the value, which can be downcast to its original type.
///
/// # Examples
///
/// You can create an `AnyType` from any type that implements `Any`:
///
/// ```
/// use dynamic_object::AnyType;
/// let any = AnyType::from("Hello, world!");
/// ```
///
/// You can downcast an `AnyType` back to a reference of its original type:
///
/// ```
/// use dynamic_object::AnyType;
/// let any = AnyType::from("Hello, world!");
/// let string = any.downcast_ref::<&str>();
/// assert_eq!(string, Some(&"Hello, world!"));
/// ```
///
/// # Safety
///
/// The `AnyType` struct uses raw pointers to store its value, which is inherently unsafe. The `new`
/// function takes ownership of the value, and leaks it, so it will never be dropped. The `downcast_ref`
/// function returns a reference to the original value, but it's unsafe because of the dangers of dereferencing raw pointers.
///
/// # Representation
///
/// An `AnyType` is made up of a single component: a raw pointer to a value that implements `Any`.
/// This pointer points to a heap-allocated value.
///
/// [Any]: core::any::Any "any::Any"
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct AnyType {
    /// A raw pointer to the value.
    value: *const dyn Any,
}

impl Drop for AnyType {
    fn drop(&mut self) {
        // This is safe because the value is leaked, so it will never be dropped.
        let x = unsafe { Box::from_raw(self.value as *mut dyn Any) };
        std::mem::drop(x);
    }
}

impl AnyType {
    /// Creates a new `AnyType` from a value.
    ///
    /// This function takes ownership of the value, and leaks it, so it will never be dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use dynamic_object::AnyType;
    /// let any = AnyType::from("Hello, world!");
    /// ```
    pub fn from<T: Any>(value: T) -> Self {
        let value: *const dyn Any = Box::leak(Box::new(value));
        Self { value }
    }

    /// Creates a new `AnyType` from a default value.
    ///
    /// This function is useful when you want to create an `AnyType` from a type that implements
    /// `Any` and `Default`. It takes ownership of the value, and leaks it, so it will never be
    /// dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use dynamic_object::AnyType;
    /// let any = AnyType::new::<&str>();
    /// ```
    pub fn new<T: Any + Default>() -> Self {
        Self::from(T::default())
    }

    /// Returns a reference to the original value if it is of type `T`, or `None` if it isn't.
    ///
    /// This function contains an unsafe block because it dereferences a raw pointer, which is
    /// inherently unsafe.
    ///
    /// # Examples
    ///
    /// ```
    /// use dynamic_object::AnyType;
    /// let any = AnyType::from("Hello, world!");
    /// let string = any.downcast_ref::<&str>();
    /// assert_eq!(string, Some(&"Hello, world!"));
    /// ```
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        unsafe { (*self.value).downcast_ref() }
    }
}

/// A type-erased, key-value map.
///
/// The `Object` struct is a wrapper around a `HashMap` that allows storing any value that implements the `Any` trait.
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
/// # Safety
///
/// The `Object` struct uses `AnyType` to store its values, so the same safety concerns apply.
/// The `get_as` and `get_or_insert_as` functions return a reference to the original value, but they are unsafe
/// because of the dangers of dereferencing raw pointers.
///
/// # Downcasting
///
/// The `get_as` and `get_or_insert_as` methods attempt to downcast the value to the correct type.
/// If the value is not of the correct type, these methods will return `None`.
///
/// [Any]: core::any::Any "any::Any"
/// [AnyType]: crate::AnyType "AnyType"
/// [HashMap]: std::collections::HashMap "collections::HashMap"
#[derive(Debug, PartialEq)]
pub struct Object {
    map: HashMap<String, AnyType>,
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
        let map = HashMap::new();
        Self { map }
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
    pub fn insert<K: Into<String>, V: Any>(&mut self, key: K, value: V) {
        self.map.insert(key.into(), AnyType::from(value));
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
        self.map.get(key).and_then(|v| v.downcast_ref::<T>())
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
    /// assert_eq!(value, Some(&"value"));
    /// ```
    pub fn get_or_insert_as<T: 'static>(&mut self, key: impl Into<String>, value: T) -> Option<&T> {
        self.map
            .entry(key.into())
            .or_insert(AnyType::from(value))
            .downcast_ref::<T>()
    }
}

impl Default for Object {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for Object {
    type Target = HashMap<String, AnyType>;
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

    #[derive(Debug, PartialEq)]
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
