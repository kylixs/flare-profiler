use super::native::JavaClass;

///
/// Enumeration of the possible Java types.
///
#[derive(Debug, Eq, PartialEq)]
pub enum JavaType<'a> {
    Boolean,
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Short,
    Void,
    Class(&'a str),
    Array(Box<JavaType<'a>>)
}

impl<'a> JavaType<'a> {

    /// Convert a given type signature into a JavaType instance (if possible). None is returned
    /// if the conversation was not successful.
    pub fn parse(signature: &'a str) -> Option<JavaType<'a>> {
        match signature.len() {
            0 => None,
            1 => match &*signature {
                "B" => Some(JavaType::Byte),
                "C" => Some(JavaType::Char),
                "D" => Some(JavaType::Double),
                "F" => Some(JavaType::Float),
                "I" => Some(JavaType::Int),
                "J" => Some(JavaType::Long),
                "S" => Some(JavaType::Short),
                "V" => Some(JavaType::Void),
                "Z" => Some(JavaType::Boolean),
                _ => None
            },
            _ => {
                match signature.chars().nth(0).unwrap() {
                    '[' => {
                        let (_, local_type) = signature.split_at(1);

                        match JavaType::parse(local_type) {
                            Some(result) => Some(JavaType::Array(Box::new(result))),
                            None => None
                        }
                    },
                    'L' => Some(JavaType::Class(signature)),
                    _ => None
                }
            }
        }
    }

    ///
    /// Converts the given Java type into a conventional human-readable representation
    ///
    pub fn to_string(java_type: &JavaType) -> String {
        match *java_type {
            JavaType::Byte => "byte".to_string(),
            JavaType::Char => "char".to_string(),
            JavaType::Double => "double".to_string(),
            JavaType::Float => "float".to_string(),
            JavaType::Int => "int".to_string(),
            JavaType::Long => "long".to_string(),
            JavaType::Short => "short".to_string(),
            JavaType::Void => "void".to_string(),
            JavaType::Boolean => "boolean".to_string(),
            JavaType::Array(ref inner_type) => format!("{}[]", JavaType::to_string(inner_type)),
            JavaType::Class(cls) => cls.trim_left_matches("L").trim_right_matches(";").replace(";", "").replace("/", ".").to_string()
        }
    }
}

///
/// Represents a JNI local reference to a Java class
///
pub struct ClassId {
    pub native_id: JavaClass
}

pub struct ClassSignature {
    pub package: String, // eq Class.getPackage() : java.lang
    pub name: String, //eq Class.getName() : java.lang.String
    pub generic: String
}

impl ClassSignature {

    pub fn new(java_type: &JavaType, raw_generic: String) -> ClassSignature {
        let str = JavaType::to_string(java_type);
        match str.rfind('.') {
            Some(idx) => {
                let (pkg, name) = str.split_at(idx + 1);

                ClassSignature {
                    package: pkg.trim_right_matches(".").to_string(),
                    name: str.to_string(),
                    generic: raw_generic
                }
            },
            None => ClassSignature { package: "".to_string(), name: str.to_string(), generic: raw_generic }

        }
    }

    pub fn to_string(&self) -> String {
        self.name.to_string()
    }
}

///
/// Represents a Java class
///
pub struct Class {
    pub id: ClassId,
    pub signature: ClassSignature
}

impl Class {

    /// Constructs a new Class instance.
    pub fn new<'a>(id: ClassId, signature: JavaType<'a>) -> Class {
        Class { id: id, signature: ClassSignature::new(&signature, "".to_string()) }
    }

    /// Returns the readable name of this class
    pub fn to_string(&self) -> String {
        self.signature.to_string()
    }
}
