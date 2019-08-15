use super::bytecode::classfile::*;

pub mod asm;

pub enum JavaType {
    Boolean,
    String,
    Integer,
    Long,
    Float,
    Double,
    Array,
    Reference,
    Void
}

pub struct JavaClass {
    methods: Vec<Method>,
    fields: Vec<Field>
}

impl JavaClass {
    pub fn new() -> JavaClass {
        JavaClass {
            methods: vec![],
            fields: vec![]
        }
    }

    pub fn to_classfile(&self) -> Classfile {
        Classfile::new()
    }

    // TODO: this function should report errors better, instead of just returning nothing on error
    pub fn from_classfile(classfile: &Classfile) -> Option<JavaClass> {
        None
    }

    pub fn add_method(method: Method) {

    }
}

pub struct Field {
    name: String,
    field_type: JavaType
}

impl Field {
    pub fn new(name: String, field_type: JavaType) -> Field {
        Field {
            name: name,
            field_type: field_type
        }
    }
}

pub struct Method {
    name: String,
    return_type: JavaType
}

impl Method {
    pub fn new(name: String) -> Method {
        Method {
            name: name,
            return_type: JavaType::Void
        }
    }
}
