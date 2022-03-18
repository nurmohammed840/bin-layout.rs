[Doc](https://docs.rs/bin-layout/)

Very fast! And flexible, This library used to serialize and deserialize data in binary format.

Inspaired by [bincode](https://github.com/bincode-org/bincode), But much more flexible.

### [Endianness](https://en.wikipedia.org/wiki/Endianness)

By default, the library uses little endian.
If you want to use big endian, you can set `BE` features flag. And for native endian use `NE`. For example:

```toml
[dependencies]
bin-layout = { version = "2", features = ["BE"] }
```

### Example

```rust
use bin_layout::DataType;

#[derive(DataType)]
struct Car<'a> {
    name: &'a str,  // Zero-Copy deserialization
    year: u16,
    is_new: bool,
}

#[derive(DataType)]
struct Company<'a> { name: String, cars: Vec<Car<'a>> }

let company = Company {
    name: "Tesla".into(),
    cars: vec![
        Car { name: "Model S", year: 2018, is_new: true },
        Car { name: "Model X", year: 2019, is_new: false },
    ],
};

let mut buf = [0; 64];

company.encode(buf.as_mut()).unwrap();
let company = Company::decode(buf.as_ref()).unwrap();
```

### Data Type

The only trait you need to implement is [DataType](https://docs.rs/bin-layout/latest/bin_layout/trait.DataType.html).

All [primitive types](https://doc.rust-lang.org/stable/rust-by-example/primitives.html) implement this trait.

`Vec`, `String`, `&[T]`, `&str` etc.. are encoded with their length value (`U22`) first, Following by each entry.


#### Dynamic Length Size

Support types are `U15`, `U22` and `U29`.

Default is `U22`. But you override it by setting `LF15`, `L29` or `U32` in features flag.
  
Those types are used to represent the length of a variable-length record.
 
Encoding algorithm is very simple, If  LSB (least significant bit) is set, then
read the next byte, last byte does not contain LSB.
 
For example, Binary representation of `0x_C0DE` is `0x_11_0000001_1011110`
 
`U22(0x_C0DE)` is encoded in 3 bytes:
 
```yml
1st byte: 1_1011110      # LSB is 1, so read next byte
2nd byte: 1_0000001      # LSB is 1, continue reading
3rd byte:        11
```

#### Fixed Size

`Record` can be used to represent fixed-size records. 

It accepts a generic type of length (`u8`, `u16` or `u32`) and a type of variable-length record type (`Vec<T>`, `String` etc..)