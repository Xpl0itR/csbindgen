use std::{
    error::Error,
    fs::{File, OpenOptions},
    io::{self, Write},
    path::Path,
};

use crate::{generate, GenerateKind};

pub struct Builder {
    options: BindgenOptions,
}

pub struct BindgenOptions {
    pub input_bindgen_file: String,
    pub input_extern_file: String,
    pub method_filter: fn(method_name: String) -> bool,
    pub rust_method_type_path: String,
    pub rust_method_prefix: String,
    pub rust_file_header: String,
    pub csharp_namespace: String,
    pub csharp_class_name: String,
    pub csharp_dll_name: String,
    pub csharp_entry_point_prefix: String,
    pub csharp_method_prefix: String,
    pub csharp_c_long_convert: String,
    pub csharp_c_ulong_convert: String,
    pub csharp_if_symbol: String,
    pub csharp_if_dll_name: String,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            options: BindgenOptions {
                input_bindgen_file: "".to_string(),
                input_extern_file: "".to_string(),
                method_filter: |x| !x.starts_with('_'),
                rust_method_type_path: "".to_string(),
                rust_method_prefix: "csbindgen_".to_string(),
                rust_file_header: "".to_string(),
                csharp_namespace: "CsBindgen".to_string(),
                csharp_class_name: "NativeMethods".to_string(),
                csharp_dll_name: "".to_string(),
                csharp_entry_point_prefix: "".to_string(),
                csharp_method_prefix: "".to_string(),
                csharp_c_long_convert: "int".to_string(),
                csharp_c_ulong_convert: "uint".to_string(),
                csharp_if_symbol: "".to_string(),
                csharp_if_dll_name: "".to_string(),
            },
        }
    }
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Change an input .rs file(such as generated from bindgen) to generate binding.
    pub fn input_bindgen_file<T: Into<String>>(mut self, input_bindgen_file: T) -> Builder {
        self.options.input_bindgen_file = input_bindgen_file.into();
        self
    }

    /// Change an input .rs file for collect extern methods to C# binding.
    pub fn input_extern_file<T: Into<String>>(mut self, input_extern_file: T) -> Builder {
        self.options.input_extern_file = input_extern_file.into();
        self
    }

    /// Filter generate method callback, default is `!x.starts_with('_')`
    pub fn method_filter(mut self, method_filter: fn(method_name: String) -> bool) -> Builder {
        self.options.method_filter = method_filter;
        self
    }

    /// add original extern call type prefix to rust wrapper,
    /// `return {rust_method_type_path}::foo()`
    pub fn rust_method_type_path<T: Into<String>>(mut self, rust_method_type_path: T) -> Builder {
        self.options.rust_method_type_path = rust_method_type_path.into();
        self
    }

    /// add method prefix to rust wrapper, default is `csbindgen_`
    /// `pub extern "C" fn {rust_method_prefix}foo()`
    pub fn rust_method_prefix<T: Into<String>>(mut self, rust_method_prefix: T) -> Builder {
        self.options.rust_method_prefix = rust_method_prefix.into();
        self
    }

    /// add file header string to rust wrapper,
    /// `mod lz4;`, `use super::lz4;`
    pub fn rust_file_header<T: Into<String>>(mut self, rust_file_header: T) -> Builder {
        self.options.rust_file_header = rust_file_header.into();
        self
    }

    /// configure C# file namespace(default is `CsBindgen`),
    /// "namespace {csharp_namespace}"
    pub fn csharp_namespace<T: Into<String>>(mut self, csharp_namespace: T) -> Builder {
        self.options.csharp_namespace = csharp_namespace.into();
        self
    }

    /// configure C# class name(default is `NativeMethods`),
    /// `public static unsafe partial class {csharp_class_name}`
    pub fn csharp_class_name<T: Into<String>>(mut self, csharp_class_name: T) -> Builder {
        self.options.csharp_class_name = csharp_class_name.into();
        self
    }

    /// configure C# load dll name,
    /// `[DllImport({csharp_dll_name})]`
    pub fn csharp_dll_name<T: Into<String>>(mut self, csharp_dll_name: T) -> Builder {
        self.options.csharp_dll_name = csharp_dll_name.into();
        self
    }

    /// configure C# DllImport EntryPoint prefix,
    /// `[DllImport(, EntryPoint ="{csharp_entry_point_prefix}foo")]`
    pub fn csharp_entry_point_prefix<T: Into<String>>(
        mut self,
        csharp_entry_point_prefix: T,
    ) -> Builder {
        self.options.csharp_entry_point_prefix = csharp_entry_point_prefix.into();
        self
    }

    /// configure C# calling method name prefix,
    /// `public static extern void {csharp_method_prefix}foo()`
    pub fn csharp_method_prefix<T: Into<String>>(mut self, csharp_method_prefix: T) -> Builder {
        self.options.csharp_method_prefix = csharp_method_prefix.into();
        self
    }

    /// configure c_long to {csharp_c_long_convert} type,
    /// default is `int`.
    pub fn csharp_c_long_convert<T: Into<String>>(mut self, csharp_c_long_convert: T) -> Builder {
        self.options.csharp_c_long_convert = csharp_c_long_convert.into();
        self
    }

    /// configure c_ulong to {csharp_c_ulong_convert} type,
    /// default is `uint`.
    pub fn csharp_c_ulong_convert<T: Into<String>>(mut self, csharp_c_ulong_convert: T) -> Builder {
        self.options.csharp_c_ulong_convert = csharp_c_ulong_convert.into();
        self
    }

    pub fn csharp_dll_name_if<T: Into<String>>(mut self, if_symbol: T, if_dll_name: T) -> Builder {
        self.options.csharp_if_symbol = if_symbol.into();
        self.options.csharp_if_dll_name = if_dll_name.into();
        self
    }

    pub fn generate_csharp_file<P: AsRef<Path>>(
        &self,
        csharp_output_path: P,
    ) -> Result<(), Box<dyn Error>> {
        if !self.options.input_bindgen_file.is_empty() {
            let (_, csharp) = generate(GenerateKind::InputBindgen, &self.options)?;

            let mut csharp_file = make_file(csharp_output_path.as_ref())?;
            csharp_file.write_all(csharp.as_bytes())?;
            csharp_file.flush()?;
        }

        if !self.options.input_extern_file.is_empty() {
            let (_, csharp) = generate(GenerateKind::InputExtern, &self.options)?;

            let mut csharp_file = make_file(csharp_output_path.as_ref())?;
            csharp_file.write_all(csharp.as_bytes())?;
            csharp_file.flush()?;
        }

        Ok(())
    }

    pub fn generate_to_file<P: AsRef<Path>>(
        &self,
        rust_output_path: P,
        csharp_output_path: P,
    ) -> Result<(), Box<dyn Error>> {
        if !self.options.input_bindgen_file.is_empty() {
            let (rust, csharp) = generate(GenerateKind::InputBindgen, &self.options)?;

            if let Some(rust) = rust {
                let mut rust_file = make_file(rust_output_path.as_ref())?;

                rust_file.write_all(rust.as_bytes())?;
                rust_file.flush()?;
            }

            let mut csharp_file = make_file(csharp_output_path.as_ref())?;
            csharp_file.write_all(csharp.as_bytes())?;
            csharp_file.flush()?;
        }

        if !self.options.input_extern_file.is_empty() {
            let (rust, csharp) = generate(GenerateKind::InputExtern, &self.options)?;

            if let Some(rust) = rust {
                let mut rust_file = make_file(rust_output_path.as_ref())?;

                rust_file.write_all(rust.as_bytes())?;
                rust_file.flush()?;
            }

            let mut csharp_file = make_file(csharp_output_path.as_ref())?;
            csharp_file.write_all(csharp.as_bytes())?;
            csharp_file.flush()?;
        }

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
