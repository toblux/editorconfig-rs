#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    os::raw::c_void,
    path::{Path, PathBuf},
    ptr,
};

use editorconfig_sys::{
    EDITORCONFIG_PARSE_MEMORY_ERROR, EDITORCONFIG_PARSE_NOT_FULL_PATH,
    EDITORCONFIG_PARSE_VERSION_TOO_NEW,
};

/// EditorConfig handle
pub struct EditorConfigHandle {
    handle: *mut c_void,
    config_filename: Option<CString>,
}

/// EditorConfig version
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Version {
    /// Major version number
    pub major: i32,
    /// Minor version number
    pub minor: i32,
    /// Patch version number
    pub patch: i32,
}

/// Errors returned by [`EditorConfigHandle::parse`]
#[derive(Debug, PartialEq)]
pub enum ParseError {
    /// TODO: Add comment
    VersionTooNewError,
    /// TODO: Add comment
    MemoryError,
    /// [`EditorConfigHandle::parse`] must be called with an absolute path and
    /// returns this error if it was called with a relative path instead
    NotFullPathError,
    /// [`EditorConfigHandle::parse`] returns this error if your config file is
    /// invalid including the line number where the error occured
    LineError(i32),
}

impl EditorConfigHandle {
    /// Creates a new [`EditorConfigHandle`]
    ///
    /// # Example
    ///
    /// ```
    /// let handle = editorconfig_rs::EditorConfigHandle::new();
    /// # assert!(handle.is_ok());
    /// ```
    ///
    pub fn new() -> Result<Self, &'static str> {
        let handle = unsafe { editorconfig_sys::editorconfig_handle_init() };
        if handle.is_null() {
            Err("Failed to create EditorConfigHandle")
        } else {
            Ok(EditorConfigHandle {
                handle,
                config_filename: None,
            })
        }
    }

    /// TODO: Add comment
    pub fn get_version(&self) -> Version {
        let mut major = -1;
        let mut minor = -1;
        let mut patch = -1;

        unsafe {
            editorconfig_sys::editorconfig_handle_get_version(
                self.handle,
                &mut major,
                &mut minor,
                &mut patch,
            );
        }

        Version {
            major,
            minor,
            patch,
        }
    }

    /// TODO: Add comment
    pub fn set_version(&self, version: Version) {
        unsafe {
            editorconfig_sys::editorconfig_handle_set_version(
                self.handle,
                version.major,
                version.minor,
                version.patch,
            );
        };
    }

    /// TODO: Add comment
    pub fn get_config_filename(&self) -> Option<String> {
        let filename =
            unsafe { editorconfig_sys::editorconfig_handle_get_conf_file_name(self.handle) };
        if filename.is_null() {
            None
        } else {
            let filename = unsafe { CStr::from_ptr(filename) };
            let filename = filename.to_str().map(|s| s.to_owned());
            filename.ok()
        }
    }

    /// TODO: Add comment
    pub fn set_config_filename(&mut self, filename: &str) {
        let err_msg = format!("Failed to create CString from filename: {}", filename);
        let filename = CString::new(filename).expect(&err_msg);
        unsafe {
            editorconfig_sys::editorconfig_handle_set_conf_file_name(
                self.handle,
                filename.as_ptr(),
            );
        };

        // Store the CString so it lives as long as the handle
        self.config_filename = Some(filename);
    }

    /// TODO: Add comment
    pub fn parse<P: AsRef<Path>>(&self, absolute_path: P) -> Option<ParseError> {
        let absolute_path = absolute_path.as_ref().to_str().expect("Invalid UTF-8 path");
        let err_msg = format!("Failed to create CString from path: {}", absolute_path);
        let absolute_path = CString::new(absolute_path).expect(&err_msg);

        let err_num =
            unsafe { editorconfig_sys::editorconfig_parse(absolute_path.as_ptr(), self.handle) };
        match err_num {
            0 => None,
            EDITORCONFIG_PARSE_VERSION_TOO_NEW => Some(ParseError::VersionTooNewError),
            EDITORCONFIG_PARSE_MEMORY_ERROR => Some(ParseError::MemoryError),
            EDITORCONFIG_PARSE_NOT_FULL_PATH => Some(ParseError::NotFullPathError),
            _ if err_num > 0 => Some(ParseError::LineError(err_num)),
            _ => unreachable!(),
        }
    }

    /// Returns the path of the erroneous config file
    ///
    /// When [`EditorConfigHandle::parse`] returns a [`ParseError`], use this
    /// method to determine the path of the erroneous config file.
    ///
    /// # Returns
    ///
    /// A [`PathBuf`] with the path of the erroneous config file or [`None`] if
    /// there was no error
    ///
    pub fn get_error_file(&self) -> Option<PathBuf> {
        let err_file_path =
            unsafe { editorconfig_sys::editorconfig_handle_get_err_file(self.handle) };
        if err_file_path.is_null() {
            None
        } else {
            let err_file_path = unsafe { CStr::from_ptr(err_file_path) };
            err_file_path.to_str().map(|p| PathBuf::from(p)).ok()
        }
    }

    /// Returns the number of rules found after parsing the config file
    ///
    /// # Example
    ///
    /// ```
    /// let handle = editorconfig_rs::EditorConfigHandle::new().unwrap();
    /// // Parse a file here or `get_rule_count` returns 0 instead
    /// let rule_count = handle.get_rule_count();
    /// # assert_eq!(rule_count, 0);
    /// ```
    ///
    pub fn get_rule_count(&self) -> u16 {
        unsafe { editorconfig_sys::editorconfig_handle_get_name_value_count(self.handle) as u16 }
    }

    /// TODO: Add comment
    pub fn get_rules(&self) -> HashMap<String, String> {
        let rule_count = self.get_rule_count();
        let mut rules = HashMap::with_capacity(rule_count.into());
        let (mut rule_name, mut rule_value) = (ptr::null(), ptr::null());

        for rule_index in 0..rule_count {
            unsafe {
                editorconfig_sys::editorconfig_handle_get_name_value(
                    self.handle,
                    rule_index.into(),
                    &mut rule_name,
                    &mut rule_value,
                );
            }

            if rule_name.is_null() || rule_value.is_null() {
                panic!("rule name or value should never be null");
            }

            if let (Ok(rule_name), Ok(rule_value)) = (
                unsafe { CStr::from_ptr(rule_name) }
                    .to_str()
                    .map(|s| s.to_owned()),
                unsafe { CStr::from_ptr(rule_value) }
                    .to_str()
                    .map(|s| s.to_owned()),
            ) {
                rules.insert(rule_name, rule_value);
            }

            rule_name = ptr::null();
            rule_value = ptr::null();
        }

        rules
    }
}

impl Drop for EditorConfigHandle {
    fn drop(&mut self) {
        unsafe {
            editorconfig_sys::editorconfig_handle_destroy(self.handle);
        }
    }
}

/// Gets the error message for a [`ParseError`] from the underlying libeditorconfig C library
///
/// # Example
///
/// ```
/// let parse_err = editorconfig_rs::ParseError::LineError(23);
/// if let Some(err_msg) = editorconfig_rs::get_error_message(parse_err) {
///     println!("Error parsing .editorconfig at line 23: {}", err_msg);
/// }
/// # else { panic!(); }
/// ```
///
pub fn get_error_message(parse_error: ParseError) -> Option<String> {
    let err_num = match parse_error {
        ParseError::VersionTooNewError => EDITORCONFIG_PARSE_VERSION_TOO_NEW,
        ParseError::MemoryError => EDITORCONFIG_PARSE_MEMORY_ERROR,
        ParseError::NotFullPathError => EDITORCONFIG_PARSE_NOT_FULL_PATH,
        ParseError::LineError(line_num) => line_num,
    };

    let err_msg = unsafe { editorconfig_sys::editorconfig_get_error_msg(err_num) };
    if err_msg.is_null() {
        None
    } else {
        let err_msg = unsafe { CStr::from_ptr(err_msg) };
        let err_msg = err_msg.to_str().map(|s| s.to_owned());
        err_msg.ok()
    }
}

/// Gets the [`Version`] number of the underlying libeditorconfig C library
///
/// # Example
///
/// ```
/// let editorconfig_rs::Version{major, minor, patch} = editorconfig_rs::get_version();
/// # assert!(major >= 0);
/// # assert!(minor >= 12);
/// # assert!(patch >= 5);
/// ```
///
pub fn get_version() -> Version {
    let mut major = -1;
    let mut minor = -1;
    let mut patch = -1;

    unsafe {
        editorconfig_sys::editorconfig_get_version(&mut major, &mut minor, &mut patch);
    };

    Version {
        major,
        minor,
        patch,
    }
}
