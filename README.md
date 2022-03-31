[Doc](https://docs.rs/bin-layout/)

Very fast! And flexible, This library used to serialize and deserialize data in binary format.

Inspaired by [bincode](https://github.com/bincode-org/bincode), But much more flexible.

### [Endianness](https://en.wikipedia.org/wiki/Endianness)

By default, the library uses little endian.
If you want to use big endian, you can set `BE` features flag. And for native endian use `NE`. For example:

```toml
[dependencies]
bin-layout = { version = "3", features = ["BE"] }
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

let old = Company {
    name: "Tesla".into(),
    cars: vec![
        Car { name: "Model S", year: 2018, is_new: true },
        Car { name: "Model X", year: 2019, is_new: false },
    ],
};

let bytes = old.encode();
let new = Company::decode(&bytes).unwrap();
```



There is two main reasons for this library to exists. 

### 1. 🚀 Performance 🚀  

There is no performance penalty for using this library. Or we can say there is zero-cost.

- Zero-copy deserialization:
    Its mean that no data is copied. Instead, the data is referenced.
    Which is only possible (safely) in rust, Other languages have to use unsafe operations.
    
    ```rust, ignore
    #[derive(DataType)]
    struct Msg<'a> {
        id: u8,
        data: &'a str,
    }
    let bytes = [42, 13, 72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 33];
    //           ^^  ^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    //           Id  Len                         Data

    let msg = Msg::decode(&bytes).unwrap();

    assert_eq!(msg.id, 42);
    assert_eq!(msg.data, "Hello, World!"); // Here, data is referenced.
    ```

- Compile time allocation:

    What if the data is fixed size? Then we don't need to allocate any memory at runtime.
    We could allocate a fixed size buffer at compile time!

    For example, The following structs, don't have any dynamic data. So we can have a fixed size buffer at compile time.

    ```rust, ignore
    #[derive(DataType)]
    struct Date {
        year: u16,
        month: u8,
        day: u8,
    }
    #[derive(DataType)]
    struct Record {
        id: u32,
        date: Date,
        value: [u8; 512],
    }

    let buf = [0; Record::SIZE]; // 520 bytes buffer.
    Record { /* ... */ }.serialize(&mut buf.as_mut().into());
    ```

    What happens if we have a dynamic data (like vector, string, etc...) ? Then we have to allocate memory at runtime.
    
    But how much memory we need to store the whole data ? When a vector is full, It creates a new vector with larger size, Then move all data to the new vector. Which is expensive.

    Well datatype has a method called `size_hint`, which calculates the total size of the data at runtime. Which is cheap to compute. `encode` method use `size_hint` function internaly.

    For example:

    ```rust, ignore
    #[derive(DataType)]
    struct Student {
        age: u8,
        roll: u32,
        name: String, // Here we have a dynamic data.
    }

    if Student::IS_DYNAMIC {
        let bytes = Student { /* ... */ }.encode();
    }
    // else, we have a fixed size buffer.
    else { /* ... */  } 
    ```
    Here more nice things is that, `IS_DYNAMIC` is a compile time constant. So compiler can optimize this code. Meaning, this is no logical `if/else` condition in executable binary!
    Thus, Zero-cost!

###  Flexibility

It work by mantaining a [Cursor](https://docs.rs/bin-layout/latest/bin_layout/struct.Cursor.html). Which is a pointer to the current position in the buffer.
And the cursor is updated when reading or writing data to the buffer.

It's very easy to implement a custom serializer/deserializer for your own data type.

For example:

```rust
use bin_layout::*;

#[derive(DataType)]
struct Bar(u16);
struct Foo { x: u8, y: Bar }

impl DataType<'_> for Foo {
    fn serialize(self, c: &mut Cursor<impl Bytes>) {
        self.x.serialize(c);
        self.y.serialize(c);
    }
    fn deserialize(c: &mut Cursor<&[u8]>) -> Result<Self> {
        Ok(Self {
            x: u8::deserialize(c)?,
            y: Bar::deserialize(c)?,
        })
    }
}
```

### Data Type

The only trait you need to implement is [DataType](https://docs.rs/bin-layout/latest/bin_layout/trait.DataType.html).

All [primitive types](https://doc.rust-lang.org/stable/rust-by-example/primitives.html) implement this trait.

`Vec`, `String`, `&[T]`, `&str` etc.. are encoded with their length value first, Following by each entry.

#### Variable-Length Integer Encoding

This encoding ensures that smaller integer values need fewer bytes to encode. Support types are `L2` and `L3`, both are encoded in little endian.

By default, `L2` (u15) is used to encode length (integer) for record. But you override it by setting `L3` (u22) in features flag.
 
Encoding algorithm is very straightforward, reserving one or two most significant bits of the first byte to encode rest of the length.

#### L2

|  MSB  | Length | Usable Bits | Range    |
| :---: | :----: | :---------: | :------- |
|   0   |   1    |      7      | 0..127   |
|   1   |   2    |     15      | 0..32767 |

#### L3

|  MSB  | Length | Usable Bits | Range      |
| :---: | :----: | :---------: | :--------- |
|   0   |   1    |      7      | 0..127     |
|  10   |   2    |     14      | 0..16383   |
|  11   |   3    |     22      | 0..4194303 |

 
For example, Binary representation of `0x_C0DE` is `0x_11_00000011_011110`
 
`L3(0x_C0DE)` is encoded in 3 bytes:
 
```yml
1st byte: 11_011110      # MSB is 11, so read next 2 bytes
2nd byte:        11
3rd byte:        11
```

Another example, `L3(107)` is encoded in just 1 byte:

```yml
1st byte: 0_1101011      # MSB is 0, So we don't have to read extra bytes.
```

#### Fixed-Length Integer Encoding

[Record](https://docs.rs/bin-layout/latest/bin_layout/struct.Record.html) can be used to represent fixed-size integer to represent the length of a record.

It accepts fixed-length unsigned interger type of `N` (`u8`, `u32`, `usize`, etc..) and a generic type of `T` (`Vec<T>`, `String` etc..)
