/// Macros for convenient way to `exit_json` from module
///
/// # Examples
///
/// ```
/// use ansible_module::{AnsibleModule, AnsibleModuleBuilder, fail_json, exit_json};
/// use serde_json::{json, Value};
///
/// let arg_spec: Value = json!({
///     "msg": {
///         "type": "str"
///     } ,
///     "additional": {
///         "type": "int",
///         "no_log": true
///     },
/// });
///
/// let module = AnsibleModuleBuilder::new(arg_spec, None)
///     .build()
///     .unwrap_or_else(|e| fail_json!(e));
///
///
/// exit_json!(module);
/// exit_json!(module, true);
/// exit_json!(module, "msg" => json!("hello world!"));
/// exit_json!(
///     module,
///     false,
///     "msg" => json!("Already done!"),
///     "additional" => json!(52)
/// );
/// exit_json!(module, false, "msg" => json!("Already done!"));
/// ```
///
#[macro_export]
macro_rules! exit_json {
    ($self:expr, $changed:expr, $($k:literal => $v:expr),+) => {
        let mut m = ::std::collections::BTreeMap::new();
        $(
            m.insert($k.to_string(), $v);
        )+
        $self.exit_json(&m, $changed)
    };
    ($self:expr, $changed:expr) => {
        let m = ::std::collections::BTreeMap::new();
        $self.exit_json(&m, $changed)
    };
    ($self:expr, $($k:literal => $v:expr),+) => {
        let mut m = ::std::collections::BTreeMap::new();
        $(
            m.insert($k.to_string(), $v);
        )+
        $self.exit_json(&m, false)
    };
    ($self:expr) => {
        let m = ::std::collections::BTreeMap::new();
        $self.exit_json(&m, false)
    };
}

/// Macros for convenient way to `fail_json` from module
///
/// # Examples
///
/// ```
/// use ansible_module::{AnsibleModule, fail_json};
///
/// fail_json!();
/// fail_json!("Something went horribly (or not) wrong!".to_string());
/// ```
///
#[macro_export]
macro_rules! fail_json {
    ($msg: expr) => {
        AnsibleModule::fail_json($msg)
    };
    () => {
        AnsibleModule::fail_json("".to_string())
    };
}

#[cfg(test)]
mod tests {
    use crate::{AnsibleModule, AnsibleModuleBuilder, exit_json};
    use serde_json::{Value, json};
    use std::io::Write;
    use std::vec;
    use tempfile::NamedTempFile;

    #[test]
    #[should_panic(expected = r#"{"changed":false,"failed":false,"also":52,"msg":"Bye bye!"}"#)]
    fn check_exit_json_macro() {
        let input_string: String = r#"{}"#.to_string();
        let arg_spec: Value = json!({
            "api_url": {
                "type": "str"
            },
        });

        let mut file: NamedTempFile = NamedTempFile::new().unwrap();
        writeln!(file, "{input_string}").unwrap();
        let input_args: Vec<String> = vec![
            "module_name".to_string(),
            file.path().to_str().unwrap().to_string(),
        ];

        let module: AnsibleModule = AnsibleModuleBuilder::new(arg_spec, Some(input_args))
            .build()
            .unwrap();

        exit_json!(
            module,
            "msg" => json!("Bye bye!"),
            "also" => json!(52)
        );
    }

    #[test]
    #[should_panic(
        expected = r#"{"msg":"Something went horribly wrong!","changed":false,"failed":true}"#
    )]
    fn check_fail_json_macro() {
        fail_json!("Something went horribly wrong!".to_string());
    }
}
