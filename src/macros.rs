/// Macros for convenient way to exit_json from module
///
/// # Examples
///
/// ```
/// use ansible_module::{AnsibleModule, exit_json};
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
/// let module = AnsibleModule::new(arg_spec, None, None, None, None, None, None).unwrap();
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
        $self.exit_json(m, $changed)
    };
    ($self:expr, $changed:expr) => {
        let m = ::std::collections::BTreeMap::new();
        $self.exit_json(m, $changed)
    };
    ($self:expr, $($k:literal => $v:expr),+) => {
        let mut m = ::std::collections::BTreeMap::new();
        $(
            m.insert($k.to_string(), $v);
        )+
        $self.exit_json(m, false)
    };
    ($self:expr) => {
        let m = ::std::collections::BTreeMap::new();
        $self.exit_json(m, false)
    };
}

/// Macros for convenient way to fail_json from module
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
