use super::super::bytecode::classfile::Classfile as ClassfileImpl;

pub mod transformer;

pub enum ClassfileVersion {
    Java1_5,
    Java1_6,
    Java1_7,
    Java1_8,
    Java1_9
}

pub struct Class {
    version: ClassfileVersion,
    constant_pool: ConstantPool
}

impl Class {
    pub fn new() -> Class {
        const DEFAULT_VERSION: ClassfileVersion = ClassfileVersion::Java1_8;

        Class {
            version: DEFAULT_VERSION,
            constant_pool: ConstantPool::new()
        }
    }

    pub fn set_version(&mut self, new_version: ClassfileVersion) -> () {
        self.version = new_version;
    }

    pub fn to_classfile(&self) -> ClassfileImpl {
        let mut cf = ClassfileImpl::new();

        cf.version.major_version = match &self.version {
            Java1_5 => 49,
            Java1_6 => 50,
            Java1_7 => 51,
            Java1_8 => 52,
            Java1_9 => 53
        };

        cf.version.minor_version = 0;

        cf
    }

    /// Return mutable reference to stored constant pool
    pub fn constant_pool(&mut self) -> &mut ConstantPool {
        &mut self.constant_pool
    }
}

pub struct ConstantPool {
}

impl ConstantPool {
    pub fn new() -> ConstantPool {
        ConstantPool {}
    }

    pub fn add_utf8_constant(&mut self, content: String) {

    }

    pub fn add_string_constant(&mut self, content: String) {

    }
}

pub struct Method {
}

impl Method {
    pub fn new() -> Method {
        Method {}
    }


}
