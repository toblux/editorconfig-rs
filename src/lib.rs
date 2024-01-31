#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    os::raw::{c_int, c_void},
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version<T: Into<c_int>> {
    /// Major version number
    pub major: T,
    /// Minor version number
    pub minor: T,
    /// Patch version number
    pub patch: T,
}

impl<T: Into<c_int> + Copy> Version<T> {
    /// Safe [`Version`] constructor that panics when negative numbers are used
    pub fn new(major: T, minor: T, patch: T) -> Self {
        if c_int::is_negative(major.into())
            || c_int::is_negative(minor.into())
            || c_int::is_negative(patch.into())
        {
            panic!("Version numbers cannot be negative");
        }

        Version {
            major,
            minor,
            patch,
        }
    }
}

/// Parsing errors returned by [`EditorConfigHandle::parse`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    LineError(c_int),
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
    ///
    /// # Example
    ///
    /// ```
    /// let handle = editorconfig_rs::EditorConfigHandle::new().unwrap();
    /// let version = handle.get_version();
    /// # use editorconfig_rs::Version;
    /// # assert_eq!(version, Version::new(0, 0, 0));
    /// ```
    ///
    pub fn get_version(&self) -> Version<c_int> {
        let (mut major, mut minor, mut patch) = (-1, -1, -1);

        unsafe {
            editorconfig_sys::editorconfig_handle_get_version(
                self.handle,
                &mut major,
                &mut minor,
                &mut patch,
            );
        }

        Version::new(major, minor, patch)
    }

    /// TODO: Add comment
    ///
    /// # Example
    ///
    /// ```
    /// use editorconfig_rs::Version;
    /// let handle = editorconfig_rs::EditorConfigHandle::new().unwrap();
    /// handle.set_version(Version::new(0, 12, 5));
    /// ```
    ///
    pub fn set_version<T: Into<c_int>>(&self, version: Version<T>) {
        unsafe {
            editorconfig_sys::editorconfig_handle_set_version(
                self.handle,
                version.major.into(),
                version.minor.into(),
                version.patch.into(),
            );
        };
    }

    /// Returns the configuration filename iff it was previously set by calling
    /// [`EditorConfigHandle::set_config_filename`]; otherwise [`None`]
    ///
    /// Note: [`None`] just means the default filename `".editorconfig"` is used
    ///
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

    /// Sets a custom EditorConfig configuration filename
    ///
    /// Allows you to change the default configuration filename ".editorconfig".
    ///
    /// # Example
    ///
    /// ```
    /// let mut handle = editorconfig_rs::EditorConfigHandle::new().unwrap();
    /// handle.set_config_filename(".myeditorconfig")
    /// ```
    ///
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

    /// Searches an absolute path for the corresponding EditorConfig rules
    ///
    /// After parsing, you can get the rules by calling
    /// [`EditorConfigHandle::get_rules`].
    ///
    /// # Example
    ///
    /// ```
    /// let handle = editorconfig_rs::EditorConfigHandle::new().unwrap();
    /// let test_file_path = std::fs::canonicalize("tests").unwrap();
    /// let err = handle.parse(test_file_path);
    /// # assert!(err.is_none());
    /// ```
    ///
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

    /// Returns the [path](PathBuf) of the invalid configuration file when
    /// [parse](EditorConfigHandle::parse) returned an [error](ParseError)
    ///
    /// # Returns
    ///
    /// The [path](PathBuf) of the invalid configuration file or [`None`] if
    /// there was no error
    ///
    pub fn get_error_file(&self) -> Option<PathBuf> {
        let err_file_path =
            unsafe { editorconfig_sys::editorconfig_handle_get_err_file(self.handle) };
        if err_file_path.is_null() {
            None
        } else {
            let err_file_path = unsafe { CStr::from_ptr(err_file_path) };
            err_file_path.to_str().map(PathBuf::from).ok()
        }
    }

    /// Returns the number of rules found after parsing
    ///
    /// # Example
    ///
    /// ```
    /// let handle = editorconfig_rs::EditorConfigHandle::new().unwrap();
    /// // Parse a file or directory; otherwise `get_rule_count()` returns 0
    /// let rule_count = handle.get_rule_count();
    /// # assert_eq!(rule_count, 0);
    /// ```
    ///
    pub fn get_rule_count(&self) -> c_int {
        unsafe { editorconfig_sys::editorconfig_handle_get_name_value_count(self.handle) }
    }

    /// Returns a map of all rules found after parsing
    ///
    /// # Example
    ///
    /// ```
    /// let handle = editorconfig_rs::EditorConfigHandle::new().unwrap();
    /// let test_file_path = std::fs::canonicalize("tests").unwrap();
    /// let err = handle.parse(test_file_path);
    /// # assert!(err.is_none());
    ///
    /// let rules = handle.get_rules();
    /// # assert_eq!(rules.len(), 3);
    /// ```
    ///
    pub fn get_rules(&self) -> HashMap<String, String> {
        let rule_count = self.get_rule_count();
        let mut rules = HashMap::with_capacity(rule_count as usize);

        for rule_index in 0..rule_count {
            let (mut rule_name, mut rule_value) = (ptr::null(), ptr::null());

            unsafe {
                editorconfig_sys::editorconfig_handle_get_name_value(
                    self.handle,
                    rule_index,
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

/// Gets the error message for a [parsing error](ParseError) from the
/// underlying `libeditorconfig` C library
///
/// # Example
///
/// ```
/// use editorconfig_rs::ParseError;
///
/// let parse_err = ParseError::LineError(23);
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

/// Gets the [version](Version) number of the underlying `libeditorconfig` C library
///
/// # Example
///
/// ```
/// use editorconfig_rs::Version;
///
/// let Version{major, minor, patch} = editorconfig_rs::get_version();
/// # assert!(major >= 0);
/// # assert!(minor >= 12);
/// # assert!(patch >= 5);
/// ```
///
pub fn get_version() -> Version<c_int> {
    let (mut major, mut minor, mut patch) = (-1, -1, -1);
    unsafe {
        editorconfig_sys::editorconfig_get_version(&mut major, &mut minor, &mut patch);
    };

    Version::new(major, minor, patch)
}
