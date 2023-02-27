use std::{
    error::Error,
    fs::{File, OpenOptions},
    io::{self, Write},
    path::Path,
};

use crate::generate;

pub struct Builder {
    options: BindgenOptions,
}

pub struct BindgenOptions {
    pub input_bindgen_file: String,

    /// add original extern call type prefix to rust wrapper,
    /// `return {rust_method_type_path}::foo()`
    pub rust_method_type_path: String,

    /// add method prefix to rust wrapper,
    /// `pub extern "C" fn {rust_method_prefix}foo()`
    pub rust_method_prefix: String,

    /// add file header string to rust wrapper,
    /// `mod lz4;`, `use super::lz4;`
    pub rust_file_header: String,

    /// configure C# file namespace(default is `CsBindgen`),
    /// "namespace {csharp_namespace}"
    pub csharp_namespace: String,

    /// configure C# class name(default is `NativeMethods`),
    /// `public static unsafe partial class {csharp_class_name}`
    pub csharp_class_name: String,

    /// configure C# load dll name,
    /// `[DllImport({csharp_dll_name})]`
    pub csharp_dll_name: String,

    /// configure C# calling method name prefix,
    /// `public static extern void {csharp_method_prefix}foo()`
    pub csharp_method_prefix: String,

    /// configure c_long to {csharp_c_long_convert} type,
    /// default is `Int32`.
    pub csharp_c_long_convert: String,

    /// configure c_long to {csharp_c_long_convert} type,
    /// default is `UInt32`.
    pub csharp_c_ulong_convert: String,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            options: BindgenOptions {
                input_bindgen_file: "".to_string(),
                rust_method_type_path: "".to_string(),
                rust_method_prefix: "".to_string(),
                rust_file_header: "".to_string(),
                csharp_namespace: "CsBindgen".to_string(),
                csharp_class_name: "NativeMethods".to_string(),
                csharp_dll_name: "".to_string(),
                csharp_method_prefix: "".to_string(),
                csharp_c_long_convert: "Int32".to_string(),
                csharp_c_ulong_convert: "UInt32".to_string(),
            },
        }
    }

    /// Change an input .rs file(such as generated from bindgen) to generate binding.
    pub fn input_bindgen_file<T: Into<String>>(mut self, input_bindgen_file: T) -> Builder {
        self.options.input_bindgen_file = input_bindgen_file.into();
        self
    }

    // TODO:method chain methods...

    /// add method prefix to rust wrapper,
    /// `pub extern "C" fn {rust_method_prefix}foo()`
    pub fn rust_method_prefix<T: Into<String>>(mut self, rust_method_prefix: T) -> Builder {
        self.options.rust_method_prefix = rust_method_prefix.into();
        self
    }

    // pub fn generate_csharp_file<T: AsRef<Path>>(&self, csharp_output_path: T) -> io::Result<()> {
    //     let mut file = OpenOptions::new()
    //         .write(true)
    //         .truncate(true)
    //         .create(true)
    //         .open(csharp_output_path.as_ref())?;

    //     let code = self.generate();
    //     file.write_all(code.as_bytes())?;
    //     file.flush()?;

    //     Ok(())
    // }

    pub fn generate_to_file<P: AsRef<Path>>(
        &self,
        rust_output_path: P,
        csharp_output_path: P,
    ) -> Result<(), Box<dyn Error>> {
        let mut rust_file = make_file(rust_output_path)?;
        let mut csharp_file = make_file(csharp_output_path)?;

        let (rust, csharp) = generate(&self.options)?;

        rust_file.write_all(rust.as_bytes())?;
        rust_file.flush()?;

        csharp_file.write_all(csharp.as_bytes())?;
        csharp_file.flush()?;

        Ok(())
    }
}

fn make_file<P: AsRef<Path>>(path: P) -> io::Result<File> {
    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)?;
    Ok(file)
}