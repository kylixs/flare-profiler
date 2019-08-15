use super::classfile::*;

pub struct ClassfilePrinter;

impl ClassfilePrinter {
    pub fn render_lines(classfile: &Classfile) -> Vec<String> {
        let mut lines = vec![];

        lines.push(format!("class {}", ClassfilePrinter::resolve_class(&classfile.this_class, &classfile.constant_pool)));
        lines.push(format!("Minor version: {}", classfile.version.minor_version));
        lines.push(format!("Major version: {}", classfile.version.major_version));
        lines.push(format!("Flags: {}", ClassfilePrinter::render_flags(&classfile.access_flags)));
        lines.push(format!("Constant pool:"));

        let mut i: i32 = -1;

        let _: Vec<()> = ClassfilePrinter::render_constant_pool(&classfile.constant_pool).iter()
            .map(|constant| {
                i = i + 1;
                format!("{:>5} = {}", format!("#{}", i), constant)
            })
            .map(|line| lines.push(line))
            .collect();

        let _: Vec<()> = ClassfilePrinter::render_methods(&classfile).iter()
            .map(|method| {
                format!("{}", method)
            })
            .map(|line| lines.push(line))
            .collect();

        lines
    }

    pub fn render_flags(flags: &AccessFlags) -> String {
        let mut flag_vec = vec![];

        if flags.has_flag(ClassAccessFlags::Public as u16) {
            flag_vec.push("ACC_PUBLIC ");
        }

        if flags.has_flag(ClassAccessFlags::Super as u16) {
            flag_vec.push("ACC_SUPER ");
        }

        // TODO implement other access flags

        flag_vec.iter().fold(String::new(), |mut acc, x| { acc.push_str(x); acc })
    }

    pub fn render_constant_pool(constant_pool: &ConstantPool) -> Vec<String> {
        constant_pool.constants.iter().map(|constant| {
            ClassfilePrinter::render_constant(&constant, constant_pool)
        }).collect()
    }

    pub fn render_constant(constant: &Constant, pool: &ConstantPool) -> String {
        match constant {
            &Constant::Utf8(ref content) => format!("Utf8               {}", String::from_utf8_lossy(content.as_slice())),
            &Constant::Integer(value) => format!("Integer            {}", value),
            &Constant::Float(value) => format!("Float               {}", value),
            &Constant::Long(value) => format!("Long               {}", value),
            &Constant::Double(value) => format!("Double              {}", value),
            &Constant::Class(ref index) => format!("Class              #{:<14}// {}", index.idx, ClassfilePrinter::resolve_utf8(index, pool)),
            &Constant::FieldRef { class_index: ref ci, name_and_type_index: ref ni } => format!("FieldRef           {:<14} // {}.{}", format!("#{}.#{}", ci.idx, ni.idx), ClassfilePrinter::resolve_class(ci, pool), ClassfilePrinter::resolve_name_and_type(ni, &pool)),
            &Constant::MethodRef { class_index: ref ci, name_and_type_index: ref ni } => format!("MethodRef          {:<14} // {}.{}", format!("#{}.#{}", ci.idx, ni.idx), ClassfilePrinter::resolve_class(ci, pool), ClassfilePrinter::resolve_name_and_type(ni, pool)),
            &Constant::InterfaceMethodRef { class_index: ref ci, name_and_type_index: ref ni } => format!("InterfaceMethodRef {:<14} // {}.{}", format!("#{}.#{}", ci.idx, ni.idx), ClassfilePrinter::resolve_class(ci, pool), ClassfilePrinter::resolve_name_and_type(ni, pool)),
            &Constant::String(ref cpi) => format!("String             #{:<14}// {}", cpi.idx, ClassfilePrinter::resolve_utf8(cpi, pool)),
            &Constant::NameAndType { name_index: ref ni, descriptor_index: ref dp } => format!("NameAndType        {:<14} // {}:{}", format!("#{}:#{}", ni.idx, dp.idx), ClassfilePrinter::resolve_utf8(ni, pool), ClassfilePrinter::resolve_utf8(dp, pool)),
            &Constant::MethodHandle { reference_kind: ref kind, reference_index: ref ri } => format!("MethodHandle       {} #{}", ClassfilePrinter::resolve_reference_kind(kind), ri.idx),
            &Constant::MethodType(ref cpi) => format!("MethodType         #{}", cpi.idx),
            &Constant::InvokeDynamic { bootstrap_method_attr_index: ref bi, name_and_type_index: ref ni } => format!("InvokeDynamic      #{}.{}", bi.idx, ClassfilePrinter::resolve_name_and_type(ni, pool)),
            &Constant::Unknown(value) => format!("Unknown constant        {}", value),
            &Constant::Placeholder => format!("Placeholder")
        }
    }

    pub fn render_utf8(index: &ConstantPoolIndex, cp: &ConstantPool) -> Option<String> {
        cp.get_utf8_string(index.idx as u16)
    }

    pub fn resolve_utf8(index: &ConstantPoolIndex, cp: &ConstantPool) -> String {
        ClassfilePrinter::render_utf8(index, cp).unwrap_or(String::from("<Not UTF8>"))
    }

    pub fn resolve_class(index: &ConstantPoolIndex, cp: &ConstantPool) -> String {
        cp.resolve_index(index).map(|constant| match constant {
            &Constant::Class(ref idx) => ClassfilePrinter::resolve_utf8(idx, cp),
            _ => String::from("<Not a class>")
        }).unwrap_or(String::from("<Not found>"))
    }

    pub fn resolve_name_and_type(nandt: &ConstantPoolIndex, cp: &ConstantPool) -> String {
        cp.resolve_index(nandt).map(|constant| match constant {
            &Constant::NameAndType { name_index: ref ni, descriptor_index: ref di } => format!("{}:{}", ClassfilePrinter::resolve_utf8(ni, cp), ClassfilePrinter::resolve_utf8(di, cp)),
            _ => String::from("<Not a name and type index>")
        }).unwrap_or(String::from("<Not found>"))
    }

    pub fn resolve_method_reference(method_reference: &ConstantPoolIndex, cp: &ConstantPool) -> String {
        cp.resolve_index(method_reference).map(|constant| match constant {
            &Constant::MethodRef { class_index: ref ci, name_and_type_index: ref ni } => format!("{}:{}", ClassfilePrinter::resolve_class(ci, cp), ClassfilePrinter::resolve_name_and_type(ni, cp)),
            _ => String::from("Not a method reference>")
        }).unwrap_or(String::from("<Not found>"))
    }

    pub fn resolve_reference_kind(kind: &ReferenceKind) -> String {
        String::from(match kind {
            &ReferenceKind::GetField => "GetField",
            &ReferenceKind::GetStatic => "GetStatic",
            &ReferenceKind::InvokeInterface => "InvokeInterface",
            &ReferenceKind::InvokeSpecial => "InvokeSpecial",
            &ReferenceKind::InvokeStatic => "InvokeStatic",
            &ReferenceKind::InvokeVirtual => "InvokeVirtual",
            &ReferenceKind::NewInvokeSpecial => "NewInvokeSpecial",
            &ReferenceKind::PutField => "PutField",
            &ReferenceKind::PutStatic => "PutStatic",
            _ => "Unknown"
        })
    }

    pub fn render_methods(classfile: &Classfile) -> Vec<String> {
        classfile.methods.iter().flat_map(|method| ClassfilePrinter::render_method(method, &classfile.constant_pool)).collect()
    }

    pub fn render_method(method: &Method, cp: &ConstantPool) -> Vec<String> {
        let mut lines = vec![];

        lines.push(format!("  {}()", ClassfilePrinter::resolve_utf8(&method.name_index, cp)));
        lines.push(format!("    Descriptor: {}", ClassfilePrinter::resolve_utf8(&method.descriptor_index, cp)));
        // TODO display access flags
        let _: Vec<()> = method.attributes.iter().flat_map(|code_attr| ClassfilePrinter::render_attribute(code_attr, cp)).map(|line| lines.push(line)).collect();

        lines.push(String::from(""));

        lines
    }

    pub fn render_attribute(code: &Attribute, cp: &ConstantPool) -> Vec<String> {
        let mut lines = vec![];

        match code {
            &Attribute::Code { max_stack: ref ms, max_locals: ref ml, code: ref c, exception_table: ref et, attributes: ref attributes } => {
                let mut instr_pointer: usize = 0;

                lines.push(String::from("    Code: "));
                lines.push(format!("      stack={} locals={} args={}", ms, ml, "???"));
                let _: Vec<()> = c.iter().map(|instr| (instr.len(), match instr {
                    &Instruction::AALOAD => format!("aaload"),
                    &Instruction::AASTORE => format!("aastore"),
                    &Instruction::ACONST_NULL => format!("aconst_null"),
                    &Instruction::ALOAD(value) => format!("aload {}", value),
                    &Instruction::ALOAD_0 => format!("aload_0"),
                    &Instruction::ALOAD_1 => format!("aload_1"),
                    &Instruction::ALOAD_2 => format!("aload_2"),
                    &Instruction::ALOAD_3 => format!("aload_3"),
                    &Instruction::ASTORE(value) => format!("astore {}", value),
                    &Instruction::ATHROW => format!("athrow"),
                    &Instruction::BALOAD => format!("baload"),
                    &Instruction::BASTORE => format!("bastore"),
                    &Instruction::BIPUSH(value) => format!("bipush {}", value),
                    &Instruction::CALOAD => format!("caload"),
                    &Instruction::CASTORE => format!("castore"),
                    &Instruction::CHECKCAST(value) => format!("checkcast {}", value),
                    &Instruction::D2F => format!("d2f"),
                    &Instruction::D2I => format!("d2i"),
                    &Instruction::D2L => format!("d2l"),
                    &Instruction::DADD => format!("dadd"),
                    &Instruction::DALOAD => format!("daload"),
                    &Instruction::DASTORE => format!("dastore"),
                    &Instruction::DCMPL => format!("dcmpl"),
                    &Instruction::DCMPG => format!("dcmpg"),
                    &Instruction::DCONST_0 => format!("dconst_0"),
                    &Instruction::DCONST_1 => format!("dconst_1"),
                    &Instruction::DDIV => format!("ddiv"),
                    &Instruction::DLOAD(value) => format!("dload {}", value),
                    &Instruction::DLOAD_0 => format!("dload_0"),
                    &Instruction::DLOAD_1 => format!("dload_1"),
                    &Instruction::DLOAD_2 => format!("dload_2"),
                    &Instruction::DLOAD_3 => format!("dload_3"),
                    &Instruction::DMUL => format!("dmul"),
                    &Instruction::DNEG => format!("dneg"),
                    &Instruction::DREM => format!("drem"),
                    &Instruction::DRETURN => format!("dreturn"),
                    &Instruction::DSTORE(value) => format!("dstore {}", value),
                    &Instruction::DSTORE_0 => format!("dstore_0"),
                    &Instruction::DSTORE_1 => format!("dstore_1"),
                    &Instruction::DSTORE_2 => format!("dstore_2"),
                    &Instruction::DSTORE_3 => format!("dstore_3"),
                    &Instruction::DSUB => format!("dsub"),
                    &Instruction::DUP => format!("dup"),
                    &Instruction::DUP_X1 => format!("dup_x1"),
                    &Instruction::DUP_X2 => format!("dup_x2"),
                    &Instruction::DUP2 => format!("dup2"),
                    &Instruction::DUP2_X1 => format!("dup2_x1"),
                    &Instruction::DUP2_X2 => format!("dup2_x2"),
                    &Instruction::F2D => format!("f2d"),
                    &Instruction::F2I => format!("f2i"),
                    &Instruction::F2L => format!("f2l"),
                    &Instruction::FADD => format!("fadd"),
                    &Instruction::FALOAD => format!("faload"),
                    &Instruction::FASTORE => format!("fastore"),
                    &Instruction::FCMPL => format!("fcmpl"),
                    &Instruction::FCMPG => format!("fcmpg"),
                    &Instruction::FCONST_0 => format!("fconst_0"),
                    &Instruction::FCONST_1 => format!("fconst_1"),
                    &Instruction::FCONST_2 => format!("fconst_2"),
                    &Instruction::FDIV => format!("fdiv"),
                    &Instruction::FLOAD(value) => format!("fload {}", value),
                    &Instruction::FLOAD_0 => format!("fload_0"),
                    &Instruction::FLOAD_1 => format!("fload_1"),
                    &Instruction::FLOAD_2 => format!("fload_2"),
                    &Instruction::FLOAD_3 => format!("fload_3"),
                    &Instruction::FMUL => format!("fmul"),
                    &Instruction::FNEG => format!("fneg"),
                    &Instruction::FREM => format!("frem"),
                    &Instruction::FRETURN => format!("freturn"),
                    &Instruction::FSTORE(value) => format!("fstore {}", value),
                    &Instruction::FSTORE_0 => format!("fstore_0"),
                    &Instruction::FSTORE_1 => format!("fstore_1"),
                    &Instruction::FSTORE_2 => format!("fstore_2"),
                    &Instruction::FSTORE_3 => format!("fstore_3"),
                    &Instruction::FSUB => format!("fsub"),
                    &Instruction::GETFIELD(value) => format!("getfield {}", value),
                    &Instruction::GETSTATIC(value) => format!("getstatic {}", value),
                    &Instruction::GOTO(value) => format!("goto {}", value),
                    &Instruction::GOTO_W(value) => format!("goto_w {}", value),
                    &Instruction::I2B => format!("i2b"),
                    &Instruction::I2C => format!("i2c"),
                    &Instruction::I2D => format!("i2d"),
                    &Instruction::I2F => format!("i2f"),
                    &Instruction::I2L => format!("i2l"),
                    &Instruction::I2S => format!("i2s"),
                    &Instruction::IADD => format!("iadd"),
                    &Instruction::IALOAD => format!("iaload"),
                    &Instruction::IAND => format!("iand"),
                    &Instruction::IASTORE => format!("iastore"),
                    &Instruction::ICONST_M1 => format!("iconst_m1"),
                    &Instruction::ICONST_0 => format!("iconst_0"),
                    &Instruction::ICONST_1 => format!("iconst_1"),
                    &Instruction::ICONST_2 => format!("iconst_2"),
                    &Instruction::ICONST_3 => format!("iconst_3"),
                    &Instruction::ICONST_4 => format!("iconst_4"),
                    &Instruction::ICONST_5 => format!("iconst_5"),
                    &Instruction::IDIV => format!("idiv"),
                    &Instruction::IF_ACMPEQ(value) => format!("if_acmpeq"),
                    &Instruction::IF_ACMPNE(value) => format!("if_acmpne"),
                    &Instruction::IF_ICMPEQ(value) => format!("if_icmpeq"),
                    &Instruction::IF_ICMPNE(value) => format!("if_icmpne"),
                    &Instruction::IF_ICMPLT(value) => format!("if_icmplt"),
                    &Instruction::IF_ICMPGE(value) => format!("if_icmpge"),
                    &Instruction::IF_ICMPGT(value) => format!("if_icmpgt"),
                    &Instruction::IF_ICMPLE(value) => format!("if_icmple"),
                    &Instruction::IFEQ(value) => format!("ifeq"),
                    &Instruction::IFNE(value) => format!("ifne"),
                    &Instruction::IFLT(value) => format!("iflt"),
                    &Instruction::IFGE(value) => format!("ifge"),
                    &Instruction::IFGT(value) => format!("ifgt"),
                    &Instruction::IFLE(value) => format!("ifle"),
                    &Instruction::IFNONNULL(value) => format!("ifnonnull"),
                    &Instruction::IFNULL(value) => format!("ifnull"),
                    &Instruction::IINC(value, increment) => format!("iinc"),
                    &Instruction::ILOAD(value) => format!("iload"),
                    &Instruction::ILOAD_0 => format!("iload_0"),
                    &Instruction::ILOAD_1 => format!("iload_1"),
                    &Instruction::ILOAD_2 => format!("iload_2"),
                    &Instruction::ILOAD_3 => format!("iload_3"),
                    &Instruction::IMUL => format!("imul"),
                    &Instruction::INEG => format!("ineg"),
                    &Instruction::INSTANCEOF(value) => format!("instanceof"),
                    &Instruction::INVOKEDYNAMIC(value) => format!("invokedynamic #{}", value),
                    &Instruction::INVOKEINTERFACE(value, index) => format!("invokeinterface #{}", value),
                    &Instruction::INVOKESPECIAL(value) => format!("invokespecial {}", ClassfilePrinter::resolve_method_reference(&ConstantPoolIndex::new(value as usize), cp)),
                    &Instruction::INVOKESTATIC(value) => format!("invokestatic {}", ClassfilePrinter::resolve_method_reference(&ConstantPoolIndex::new(value as usize), cp)),
                    &Instruction::INVOKEVIRTUAL(value) => format!("invokevirtual {}", ClassfilePrinter::resolve_method_reference(&ConstantPoolIndex::new(value as usize), cp)),
                    &Instruction::IOR => format!("ior"),
                    &Instruction::IREM => format!("irem"),
                    &Instruction::IRETURN => format!("ireturn"),
                    &Instruction::ISHL => format!("ishl"),
                    &Instruction::ISHR => format!("ishr"),
                    &Instruction::ISTORE(value) => format!("istore {}", value),
                    &Instruction::ISTORE_0 => format!("istore_0"),
                    &Instruction::ISTORE_1 => format!("istore_1"),
                    &Instruction::ISTORE_2 => format!("istore_2"),
                    &Instruction::ISTORE_3 => format!("istore_3"),
                    &Instruction::ISUB => format!("isub"),
                    &Instruction::IUSHR => format!("iushr"),
                    &Instruction::IXOR => format!("ixor"),
                    &Instruction::JSR(value) => format!("jsr"),
                    &Instruction::JSR_W(value) => format!("jsr_w"),
                    &Instruction::L2D => format!("l2d"),
                    &Instruction::L2F => format!("l2f"),
                    &Instruction::L2I => format!("l2i"),
                    &Instruction::LADD => format!("ladd"),
                    &Instruction::LALOAD => format!("laload"),
                    &Instruction::LAND => format!("land"),
                    &Instruction::LASTORE => format!("lastore"),
                    &Instruction::LCMP => format!("lcmp"),
                    &Instruction::LCONST_0 => format!("lconst_0"),
                    &Instruction::LCONST_1 => format!("lconst_1"),
                    &Instruction::LDC(value) => format!("ldc"),
                    &Instruction::LDC_W(value) => format!("ldc_w"),
                    &Instruction::LDC2_W(value) => format!("ldc2_w"),
                    &Instruction::LDIV => format!("ldiv"),
                    &Instruction::LLOAD(value) => format!("lload"),
                    &Instruction::LLOAD_0 => format!("lload_0"),
                    &Instruction::LLOAD_1 => format!("lload_1"),
                    &Instruction::LLOAD_2 => format!("lload_2"),
                    &Instruction::LLOAD_3 => format!("lload_3"),
                    &Instruction::LMUL => format!("lmul"),
                    &Instruction::LNEG => format!("lneg"),
                    &Instruction::LOOKUPSWITCH(value, ref table) => format!("lookupswitch"),
                    &Instruction::LOR => format!("lor"),
                    &Instruction::LREM => format!("lrem"),
                    &Instruction::LRETURN => format!("lreturn"),
                    &Instruction::LSHL => format!("lshl"),
                    &Instruction::LSHR => format!("lshr"),
                    &Instruction::LSTORE(value) => format!("lstore {}", value),
                    &Instruction::LSTORE_0 => format!("lstore_0"),
                    &Instruction::LSTORE_1 => format!("lstore_1"),
                    &Instruction::LSTORE_2 => format!("lstore_2"),
                    &Instruction::LSTORE_3 => format!("lstore_3"),
                    &Instruction::LSUB => format!("lsub"),
                    &Instruction::LUSHR => format!("lushr"),
                    &Instruction::LXOR => format!("lxor"),
                    &Instruction::MONITORENTER => format!("monitorenter"),
                    &Instruction::MONITOREXIT => format!("monitorexit"),
                    &Instruction::MULTIANEWARRAY(value, size) => format!("multianewarray"),
                    &Instruction::NEW(value) => format!("new"),
                    &Instruction::NEWARRAY(value) => format!("newarray"),
                    &Instruction::NOP => format!("nop"),
                    &Instruction::POP => format!("pop"),
                    &Instruction::POP2 => format!("pop2"),
                    &Instruction::PUTFIELD(value) => format!("putfield"),
                    &Instruction::PUTSTATIC(value) => format!("putstatic"),
                    &Instruction::RET(value) => format!("ret"),
                    &Instruction::RETURN => format!("return"),
                    &Instruction::SALOAD => format!("saload"),
                    &Instruction::SASTORE => format!("sastore"),
                    &Instruction::SIPUSH(value) => format!("sipush {}", value),
                    &Instruction::SWAP => format!("swap"),
                    &Instruction::TABLESWITCH(value, _, _, _) => format!("tableswitch"),
                    &Instruction::IINC_W(value, increment) => format!("iinc_w"),
                    &Instruction::ILOAD_W(value) => format!("iload_w {}", value),
                    &Instruction::FLOAD_W(value) => format!("fload_w {}", value),
                    &Instruction::ALOAD_W(value) => format!("aload_w {}", value),
                    &Instruction::LLOAD_W(value) => format!("lload_w {}", value),
                    &Instruction::DLOAD_W(value) => format!("dload_w {}", value),
                    &Instruction::ISTORE_W(value) => format!("istore_w {}", value),
                    &Instruction::FSTORE_W(value) => format!("fstore_w {}", value),
                    &Instruction::ASTORE_W(value) => format!("astore_w {}", value),
                    &Instruction::LSTORE_W(value) => format!("lstore_w {}", value),
                    &Instruction::DSTORE_W(value) => format!("dstore_w {}", value),
                    &Instruction::RET_W(value) => format!("ret_w {}", value),
                    &Instruction::PADDED_INSTRUCTION(value) => format!("padded_instruction {}", value),
                    &Instruction::WTF(value) => format!("wtf {}", value),
                    _ => format!("instr")
                })).map(|line| {
                    lines.push(format!("     {:>4}: {}", instr_pointer, line.1));
                    instr_pointer = instr_pointer + line.0
                }).collect();

                let _: Vec<()> = attributes.iter().flat_map(|att| ClassfilePrinter::render_attribute(att, cp)).map(|line| format!("  {}", line)).map(|line| lines.push(line)).collect();
            },
            &Attribute::LineNumberTable(ref table) => {
                lines.push(String::from("    LineNumberTable"));

                let _: Vec<()> = ClassfilePrinter::render_line_number_table(table).iter().map(|line_number| lines.push(format!("      {}", line_number))).collect();
            },
            &Attribute::ConstantValue(ref cpi) => { lines.push(format!("    ConstantValue #{}", cpi.idx)); },
            &Attribute::StackMapTable(ref table) => {
                lines.push(format!("    StackMapTable"));
                let _: Vec<()> = table.iter().map(|frame| ClassfilePrinter::render_stack_map_frame(frame)).map(|line| lines.push(format!("      {}", line))).collect();
            },
            &Attribute::AnnotationDefault(_) => { lines.push(format!("    AnnotationDefault")); },
            &Attribute::BootstrapMethods(_) => { lines.push(format!("    BootstrapMethods")); },
            &Attribute::LocalVariableTable(ref table) => {
                lines.push(String::from("    LocalVariableTable"));
                let _: Vec<()> = table.iter().map(|local_var| ClassfilePrinter::render_local_variable(local_var)).map(|line| lines.push(format!("    {}", line))).collect();
            },
            &Attribute::LocalVariableTypeTable(ref table) => {
                lines.push(String::from("    LocalVariableTypeTable"));
                let _: Vec<()> = table.iter().map(|var_type| ClassfilePrinter::render_local_variable_type(var_type)).map(|line| lines.push(format!("    {}", line))).collect();
            },
            &Attribute::Deprecated => { lines.push(format!("    Deprecated")); },
            _ => {
                lines.push(format!("RandomAttribute"));
                ()
            },
    //Code { max_stack: u16, max_locals: u16, code: Vec<Instruction>, exception_table: Vec<ExceptionHandler>, attributes: Vec<Attribute> },
        }

        lines
    }

    pub fn render_stack_map_frame(frame: &StackMapFrame) -> String {
        match frame {
            &StackMapFrame::SameFrame { tag: tag } => format!("SameFrame {}", tag),
            &StackMapFrame::SameLocals1StackItemFrame { tag: tag, stack: _ /*VerificationType*/ } => format!("SameLocals1StackItemFrame {}", tag),
            &StackMapFrame::SameLocals1StackItemFrameExtended { offset_delta: offset, stack: _ /*VerificationType*/ } => format!("SameLocals1StackItemFrameExtended {}", offset),
            &StackMapFrame::ChopFrame { tag: tag, offset_delta: offset } => format!("ChopFrame {} {}", tag, offset),
            &StackMapFrame::SameFrameExtended { offset_delta: offset } => format!("SameFrameExtended {}", offset),
            &StackMapFrame::AppendFrame { tag: tag, offset_delta: offset, locals: _ /*Vec<VerificationType>*/ } => format!("AppendFrame {} {}", tag, offset),
            &StackMapFrame::FullFrame { offset_delta: offset, locals: _ /*Vec<VerificationType>*/, stack: _ /*Vec<VerificationType>*/ } => format!("FullFrame {}", offset),
            &StackMapFrame::FutureUse { tag: tag } => format!("FutureUse")
        }
    }

    pub fn render_local_variable(variable: &LocalVariableTable) -> String {
        format!("{}", variable.index)
    }

    pub fn render_local_variable_type(variable_type: &LocalVariableTypeTable) -> String {
        format!("{}", variable_type.index)
    }

    pub fn render_line_number_table(table: &Vec<LineNumberTable>) -> Vec<String> {
        table.iter().map(|line| format!("line {}: {}", line.line_number, line.start_pc)).collect()
    }
}
