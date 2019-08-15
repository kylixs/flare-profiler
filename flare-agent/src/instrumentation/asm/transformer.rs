use super::super::super::bytecode::*;

pub struct Transformer<'a> {

    class: &'a mut Classfile
}

impl<'a> Transformer<'a> {

    pub fn new(class: &mut Classfile) -> Transformer {
        Transformer {
            class: class
        }
    }

    pub fn ensure_constant(&mut self, constant: Constant) -> ConstantPoolIndex {
        self.class.constant_pool.get_constant_index(&constant).unwrap_or(self.class.constant_pool.add_constant(constant))
    }
}
