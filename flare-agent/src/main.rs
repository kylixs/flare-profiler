extern crate jvmti;

use std::env;
use std::fs::File;
//use std::io::{stdout};

use jvmti::bytecode::*;
use jvmti::bytecode::printer::*;

fn main2() {
    let class = Classfile::new();

    if let Ok(mut outfile) = File::create("RustEmpty.class") {
        let mut writer = ClassWriter::new(&mut outfile);
        let _ = writer.write_class(&class);
    }
}

// The main program is a simple interface to access the bytecode parsing and generating
// functionality and as such, it's not intended for actual use.
fn main() {
    if let (Some(action), Some(class_name)) = (env::args().nth(1), env::args().nth(2)) {
        match File::open(class_name.clone()) {
            Ok(mut file) => {
                match ClassReader::read_class(&mut file) {
                    Ok(class) => {
                        match action.as_str() {
                            "read" => println!("{}", format!("{:#?}", class)),
                            "print" => println!("{}", ClassfilePrinter::render_lines(&class).iter().map(|line| format!("{}\n", line)).fold(String::new(), |mut acc, x| { acc.push_str(x.as_str()); acc})),
                            "counts" => println!("Class: {} Field count: {} Method count: {}", class_name, class.fields.len(), class.methods.len()),
                            "methods" => show_methods(class, class_name),
                            "write" => write_class(&class),
                            _ => println!("Unknown action: {}", action)
                        }
                    },
                    Err(err) => assert!(false, format!("{:?}", err))
                }

            },
            Err(err) => assert!(false, format!("{:?}", err))
        }
    } else {
        println!("Invalid arguments. Usage: jvmti [read|write] <Class file>")
    }
}

fn write_class(class: &Classfile) {
    if let Ok(mut outfile) = File::create(format!("{}.out.class", env::args().nth(2).unwrap_or(String::from("tmp.out.class")))) {
        //let mut out = stdout();
        let mut writer = ClassWriter::new(&mut outfile);
        let _ = writer.write_class(class);
    } else {
        println!("Can't open output file");
    }
}

fn show_methods(class: Classfile, class_name: String ) {
    class.methods.iter().map(|method| {
        method.attributes.iter().map(|a| {
            match a {
                &jvmti::bytecode::Attribute::Code { max_stack: _, max_locals: _, code: _, exception_table: _, ref attributes } => {
                    attributes.iter().map(|b| {
                        match b {
                            &jvmti::bytecode::Attribute::LineNumberTable(ref table) => {
                                if table.len() > 1 {
                                    let first = table[0].line_number;
                                    let last = table[table.len() - 1].line_number;

                                    let method_name = class.constant_pool.get_utf8_string(method.name_index.idx as u16).unwrap_or(String::from("Unknown"));

                                    println!("Class: {} Method: {} Length: {}", class_name, method_name, last - first);
                                }
                                ()
                            },
                            _ => ()
                        }

                    }).fold(0, |_, _| 0);
                },
                _ => ()
            }/*
            */
        }).fold(0, |_, _| 0);
    }).fold(0, |_, _| 0);
}
