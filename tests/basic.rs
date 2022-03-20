use bin_layout::{DataType, Cursor, ErrorKind, Record};

use Subject::*;
#[derive(PartialEq, Debug, Clone)]
enum Subject<'a> {
    Math,
    Physics,
    Chemistry,
    Other(u16, Record<u8, &'a str>),
}

impl<'de> DataType<'de> for Subject<'de> {
    fn serialize(self, view: &mut Cursor<impl AsMut<[u8]>>) -> Result<(), ErrorKind> {
        let code: u16 = match self {
            Math => 302,
            Physics => 317,
            Chemistry => 345,
            Other(id, name) => {
                id.serialize(view)?;
                return Record::serialize(name, view);
            }
        };
        code.serialize(view)
    }
    fn deserialize(view: &mut Cursor<&'de [u8]>) -> Result<Self, ErrorKind> {
        let name = match u16::deserialize(view)? {
            302 => Math,
            317 => Physics,
            345 => Chemistry,
            id => return Ok(Other(id, Record::deserialize(view)?)),
        };
        Ok(name)
    }
}

#[derive(PartialEq, DataType, Debug, Clone)]
struct Student<'a> {
    age: u8,
    name: &'a str,
    gender: bool,
    roll: u8,
}

#[derive(DataType, PartialEq, Debug, Clone)]
struct Class<'a> {
    name: &'a str,
    subjects: [Subject<'a>; 4],
    students: Record<u8, Vec<Student<'a>>>,
}

#[test]
fn basic() {
    let old_class = Class {
        name: "Mango",
        subjects: [Physics, Chemistry, Other(321, "Engish II".into()), Math],
        students: vec![
            Student { age: 21, name: "John", gender: true, roll: 73 },
            Student { age: 20, name: "Jui", gender: false, roll: 36 },
        ]
        .into(),
    };

    let mut buf = [0; 50];

    let mut writer = Cursor::new(buf.as_mut());
    old_class.clone().serialize(&mut writer).unwrap();
    assert_eq!(writer.offset, 40); // 40 bytes written

    let mut reader = Cursor::new(buf.as_ref());
    let new_class = Class::deserialize(&mut reader).unwrap();
    assert_eq!(reader.offset, 40); // 40 bytes read
    
    assert_eq!(old_class, new_class);
}
