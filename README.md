# dynamic_object: Dynamic, Type-Erased Key-Value Maps in Rust

Do you love Rust but are tired of being constrained by static typing when you need a map to hold values of different types? Do you wish you could have the flexibility of JavaScript objects in Rust? Look no further! `dynamic_object` is here to save the day!

`dynamic_object` is a Rust crate that provides an easy way to create dynamic, type-erased, key-value maps. It allows you to store any value that implements the `Any` trait and retrieve it with type checking at runtime. This is particularly useful when you need a map to hold values of different types and you can't determine the types at compile time.

## Features

- **Dynamic key-value map**: Store any value that implements the `Any` trait.
- **Type checking at runtime**: Retrieve your values with type checking at runtime.
- **Macro for easy and intuitive object creation**: Use the `object` macro to create `Object` instances in a way that feels natural and intuitive.

## Usage

Add `dynamic_object` to your `Cargo.toml`:

```toml
[dependencies]
dynamic_object = { git = "https://github.com/trvswgnr/dynamic_object" }
```

You can create an `Object` and insert any type that implements `Any`:

```rust
use dynamic_object::Object;
let mut object = Object::new();
object.insert("key", "value");
```

You can retrieve a reference to the original value if it is of the correct type:

```rust
use dynamic_object::Object;
let mut object = Object::new();
object.insert("key", "value");
let value = object.get_as::<&str>("key");
assert_eq!(value, Some(&"value"));
```

If it isn't of the correct type, you will get `None`.

## Object Macro

The `object` macro provided by this crate makes it easy to create `Object` instances. It supports nested objects.

```rust
use dynamic_object::object;
let obj = object!({
    key1: "value1",
    key2: {
        inner_key: "inner_value",
    },
});
assert_eq!(obj.get_as::<&str>("key1"), Some(&"value1"));
```

## Safety

The `AnyType` struct uses `Box::leak` to create a `'static` reference, which means the value will live for the entire duration of the program. Therefore, it's safe to use and doesn't require the user to manage the lifetime of the value.

## License

`dynamic_object` is licensed under the MIT license. Please see the `LICENSE` file in the GitHub repository for more information.

## Contributing

Contributions to are welcome! Please open an issue or submit a pull request.

## Contact

If you have any questions or feedback, please feel free to open an issue. You can also reach out to me on [Xitter](https://twitter.com/techsavvytravvy).
