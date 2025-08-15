use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};

pub type ModuleArgs = HashMap<String, ArgumentValue>;

/// Struct to use `exit_json`
#[derive(Clone, Serialize, Deserialize)]
struct ExitJson {
    /// Both exit and fail must contain changed parameter
    changed: bool,
    /// Both exit and fail must contain failed parameter
    failed: bool,

    /// `ExitJson` allows users to customise output of a module
    #[serde(flatten)]
    result: BTreeMap<String, Value>,
}

/// Struct to use `fail_json`
#[derive(Clone, Serialize, Deserialize)]
struct FailJson {
    /// `FailJson` must contain a msg parameter with a reason why module failed
    msg: String,
    /// Both exit and fail must contain changed parameter
    changed: bool,
    /// Both exit and fail must contain failed parameter
    failed: bool,
}

/// All internal arguments of an `AnsibleModule` struct (see: <https://docs.ansible.com/ansible/latest/dev_guide/developing_program_flow_modules.html#internal-arguments>)
/// For now they parsed and provided as is and do not change the logic of a class itself
/// You can use these values to write your own logic
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InternalArgs {
    #[serde(default)]
    #[serde(rename(deserialize = "_ansible_no_log"))]
    pub no_log: bool,

    #[serde(default)]
    #[serde(rename(deserialize = "_ansible_debug"))]
    pub debug: bool,

    #[serde(default)]
    #[serde(rename(deserialize = "_ansible_check_mode"))]
    pub check_mode: bool,

    #[serde(default)]
    #[serde(rename(deserialize = "_ansible_diff"))]
    pub diff: bool,

    #[serde(default)]
    #[serde(rename(deserialize = "_ansible_verbosity"))]
    pub verbosity: u8,

    #[serde(default)]
    #[serde(rename(deserialize = "_ansible_socket"))]
    pub socket: Option<String>,

    #[serde(default)]
    #[serde(rename(deserialize = "_ansible_target_log_info"))]
    pub target_log_info: Option<String>,

    #[serde(default)]
    #[serde(rename(deserialize = "_ansible_ignore_unknown_opts"))]
    pub ignore_unknown_opts: bool,

    #[serde(default)]
    #[serde(rename(deserialize = "_ansible_keep_remote_files"))]
    pub keep_remote_files: bool,

    #[serde(default)]
    #[serde(rename(deserialize = "_ansible_string_conversion_action"))]
    pub string_conversion_action: Option<String>,

    #[serde(default)]
    #[serde(rename(deserialize = "_ansible_version"))]
    pub version: Option<String>,

    #[serde(default)]
    #[serde(rename(deserialize = "_ansible_module_name"))]
    pub module_name: Option<String>,

    #[serde(default)]
    #[serde(rename(deserialize = "_ansible_syslog_facility"))]
    pub syslog_facility: Option<String>,

    #[serde(default)]
    #[serde(rename(deserialize = "_ansible_selinux_special_fs"))]
    pub selinux_special_fs: Vec<String>,

    #[serde(default)]
    #[serde(rename(deserialize = "_ansible_shell_executable"))]
    pub shell_executable: Option<String>,

    #[serde(default)]
    #[serde(rename(deserialize = "_ansible_tmpdir"))]
    pub tmpdir: Option<String>,

    #[serde(default)]
    #[serde(rename(deserialize = "_ansible_remote_tmp"))]
    pub remote_tmp: Option<String>,
}

/// This struct contains the input element itself and `no_log` parameter to decide
/// if it should be printed or not
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgumentValue {
    pub value: Value,
    pub(crate) no_log: bool,
}

/// Base structure for Ansible module
///
/// Depending of where the module has failed it can fail in a two ways:
/// 1. If an error occured during the first steps (parsing input json file and deserialising),
///    we just use eprintln! and exit; normally no error should occur during this
///
/// 2. If an error occured during parsing after parsing Internal and during Module arguments
///    we now have a access to internal parameters such as `no_log: true`
///    so we exit in a JSON form
///
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnsibleModule {
    /// Input params after parsing (e.g: -a msg=123)
    pub params: ModuleArgs,
    /// Internal params (see: <https://docs.ansible.com/ansible/latest/dev_guide/developing_program_flow_modules.html#internal-arguments>)
    pub internal_params: InternalArgs,
}

impl AnsibleModule {
    /// Exits a module with custom response
    /// Note: It it reccomended to use `exit_json!` macro instead of using it directly
    ///
    /// # Arguments
    ///
    /// * `result` - A `BTreeMap` of String=Value values
    /// * `changed` - Decides if module has a changed status
    ///
    /// # Examples
    ///
    /// ```
    /// use ansible_module::{AnsibleModule, AnsibleModuleBuilder, fail_json, exit_json};
    /// use serde_json::{json, Value};
    /// use std::collections::BTreeMap;
    ///
    /// let arg_spec: Value = json!({
    ///     "msg": {},
    ///     "additional": {
    ///         "no_log": true
    ///     },
    /// });
    ///
    /// let module = AnsibleModuleBuilder::new(arg_spec, None)
    ///     .build()
    ///     .unwrap_or_else(|e| fail_json!(e));
    ///
    /// let mut result = BTreeMap::new();
    /// result.insert("msg".to_string(), json!("All good!"));
    /// module.exit_json(&result, false);
    /// ```
    pub fn exit_json(self, result: &BTreeMap<String, Value>, changed: bool) -> ! {
        // Hide `no_log=true`` values
        let result: BTreeMap<String, Value> = result
            .iter()
            .map(|(k, v)| {
                // We check if Value is argument with no_log=true
                let val: Value = if let Some(arg_val) = self.params.get(k) {
                    if arg_val.no_log {
                        json!("VALUE_SPECIFIED_IN_NO_LOG_PARAMETER")
                    } else {
                        v.clone()
                    }
                } else {
                    v.clone()
                };
                (k.clone(), val)
            })
            .collect();

        let result: String = serde_json::to_string(&ExitJson {
            result,
            changed,
            failed: false,
        })
        .unwrap();

        // Presumably Ansible itself handles global no_log logic
        // But we can assure nothing is printed
        // if self.internal_params.no_log {
        //     result.clear();
        // }

        println!("{result}");

        #[cfg(test)]
        panic!("{result}");

        #[cfg(not(test))]
        std::process::exit(0);
    }

    /// Fails a module with custom response
    /// It is a static method because we do not need to handle custom messages and internal params
    /// Note: It it reccomended to use `fail_json!` macro instead of using it directly
    ///
    /// # Arguments
    ///
    /// * `msg` - A string containing reason why the module failed
    ///
    /// # Examples
    ///
    /// ```
    /// use ansible_module::{AnsibleModule, fail_json};
    ///
    /// AnsibleModule::fail_json("Something went horribly (or not) wrong!".to_string());
    /// ```
    pub fn fail_json(msg: String) -> ! {
        let result: String = serde_json::to_string(&FailJson {
            msg,
            changed: false,
            failed: true,
        })
        .unwrap();

        println!("{result}");

        #[cfg(test)]
        panic!("{result}");

        #[cfg(not(test))]
        std::process::exit(0);
    }
}
