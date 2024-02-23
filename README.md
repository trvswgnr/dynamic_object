# Dynamic Objects in Rust

Do you love Rust but are tired of being constrained by static typing when you need a map to hold values of different types? Do you wish you could have the simple syntax and flexibility of **JavaScript** objects in **Rust**? Look no further - `dynamic_object` is here to save the day!

The `dynamic_object` crate provides an easy way to create dynamic, type-erased, key-value maps. It allows you to store any value that implements the `Any` trait and retrieve it with type checking at runtime. This is particularly useful when you need a map to hold values of different types and you can't determine the types at compile time.

## Features

- **Dynamic key-value map**: Store any value that implements the `Any` trait.
- **Type checking at runtime**: Retrieve your values with type checking at runtime.
- **Macro for easy and intuitive object creation**: Use the `object` macro to create `Object` instances in a way that feels natural and intuitive.
- **Nested objects**: The `object` macro supports nested objects.
- **Order preservation**: The `Object` struct preserves the order of insertion.

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

The `object` macro provided by this crate makes it easy to create `Object` instances. It also supports nested objects.

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

`dynamic_object` does not use `unsafe` code.

## License

`dynamic_object` is licensed under the MIT license. Please see the [`LICENSE`](LICENSE) file for more information.

## Contributing

Contributions to are welcome! Please open an issue or submit a pull request.

## Contact

If you have any questions or feedback, please feel free to open an issue. You can also reach out to me on [Xitter](https://twitter.com/techsavvytravvy).
