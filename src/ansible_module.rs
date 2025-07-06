use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashMap},
    env,
    fs::read_to_string,
    vec,
};

use crate::fail_json;

pub type ModuleArgs = HashMap<String, ArgumentValue>;
pub type ArgumentSpec = HashMap<String, Argument>;
pub type MutuallyExclusive = Vec<(String, String)>;
pub type RequiredTogether = MutuallyExclusive;
pub type RequiredOneOf = MutuallyExclusive;
pub type RequiredIf = Vec<(String, Value, Vec<String>, bool)>;
pub type RequiredBy = Vec<(String, Vec<String>)>;

/// Struct to use exit_json
#[derive(Clone, Serialize, Deserialize)]
struct ExitJson {
    /// Both exit and fail must contain changed parameter
    changed: bool,
    /// Both exit and fail must contain failed parameter
    failed: bool,

    /// ExitJson allows users to customise output of a module
    #[serde(flatten)]
    result: BTreeMap<String, Value>,
}

/// Struct to use fail_json
#[derive(Clone, Serialize, Deserialize)]
struct FailJson {
    /// FailJson must contain a msg parameter with a reason why module failed
    msg: String,
    /// Both exit and fail must contain changed parameter
    changed: bool,
    /// Both exit and fail must contain failed parameter
    failed: bool,
}

/// All internal arguments of an AnsibleModule struct (see: <https://docs.ansible.com/ansible/latest/dev_guide/developing_program_flow_modules.html#internal-arguments>)
/// For now they parsed and provided as is and do not change the logic of a class itself
/// You can use these values to write your own logic
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// This struct contains the input element itself and no_log parameter to decide
/// if it should be printed or not
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgumentValue {
    pub value: Value,
    no_log: bool,
}

/// This enum contains all types that of an Argument
/// See <https://docs.ansible.com/ansible/latest/dev_guide/developing_program_flow_modules.html#argument-spec> for reference
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ArgumentType {
    Bool,
    Str,
    Float,
    Int,
    Uint,
    List,
    Dict,
}

impl ArgumentType {
    /// This method checks if input value has correct type in corresponding arg spec
    fn check_type_correct(&self, val: &Value) -> bool {
        match self {
            ArgumentType::Bool => val.is_boolean(),
            ArgumentType::Str => val.is_string(),
            ArgumentType::Float => val.is_f64(),
            ArgumentType::Int => val.is_i64(),
            ArgumentType::Uint => val.is_u64(),
            ArgumentType::List => val.is_array(),
            ArgumentType::Dict => val.is_object(),
        }
    }
}

/// Module argument structure (see <https://docs.ansible.com/ansible/latest/dev_guide/developing_program_flow_modules.html#argument-spec>)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Argument {
    #[serde(rename(deserialize = "type"))]
    value_type: ArgumentType,

    /// Is argument required
    #[serde(default)]
    required: bool,

    /// Hide argument or not
    #[serde(default)]
    no_log: bool,

    /// Default value for an argument
    default: Option<Value>,
    /// Environment variable to fallback if required=true but not present
    fallback: Option<String>,
    /// Vector of valid values for an argument
    choices: Option<Vec<Value>>,
    // Not implemented yet
    // aliases: Option<Vec<Value>>,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnsibleModule {
    /// Input params after parsing (e.g: -a msg=123)
    pub params: ModuleArgs,
    /// Internal params (see: <https://docs.ansible.com/ansible/latest/dev_guide/developing_program_flow_modules.html#internal-arguments>)
    pub internal_params: InternalArgs,
}

impl AnsibleModule {
    pub fn new(
        argument_spec: Value,
        mutually_exclusive: Option<MutuallyExclusive>,
        required_together: Option<RequiredTogether>,
        required_one_of: Option<RequiredOneOf>,
        required_if: Option<RequiredIf>,
        required_by: Option<RequiredBy>,
        // In modules itself you can None this, but for tests we must write env::args manually
        all_input_args: Option<Vec<String>>,
    ) -> Result<Self, !> {
        // Must be iterable
        if !argument_spec.is_object() {
            fail_json!("Wrong argument spec format, must be a valid JSON object".to_string());
        }

        let all_input_args: Vec<String> = if let Some(args) = all_input_args {
            args
        } else {
            env::args().collect()
        };

        let all_input_args: Value = match AnsibleModule::parse_input_json(all_input_args) {
            Ok(val) => val,
            Err(e) => fail_json!(e),
        };

        // Now we parse internal arguments that start with an underscore (_)
        let internal_args: InternalArgs =
            match AnsibleModule::parse_internal_args(all_input_args.clone()) {
                Ok(val) => val,
                Err(e) => fail_json!(format!(
                    "Could not parse internal arguments from {all_input_args}: {e}"
                )),
            };

        // Now we parse module arguments that DO NOT start with an underscore (_)
        // After parsed we compare arg spec with input module args
        let argument_spec: ArgumentSpec = match serde_json::from_value(argument_spec.clone()) {
            Ok(arg_spec) => arg_spec,
            Err(e) => fail_json!(e.to_string()),
        };
        let module_args: ModuleArgs = match AnsibleModule::parse_module_args(
            argument_spec,
            all_input_args,
            mutually_exclusive,
            required_together,
            required_one_of,
            required_if,
            required_by,
        ) {
            Ok(val) => val,
            Err(e) => fail_json!(e.to_string()),
        };

        Ok(AnsibleModule {
            params: module_args,
            internal_params: internal_args,
        })
    }

    /// Parsers all arguments that were passed to a binary
    /// and parses the resulting string to a valid JSON objects
    ///
    /// # Arguments
    ///
    /// * `all_input_args` - Only for testing, so we can pass env::args manually
    pub(crate) fn parse_input_json(all_input_args: Vec<String>) -> Result<Value, String> {
        // Module must be executed only in a form:
        // <module_name> <json_file> (e.g.: ./assert input.json)
        let program: &str = &all_input_args[0];
        let input_file_name: &str = match all_input_args.len() {
            2 => &all_input_args[1],
            _ => {
                return Err(format!(
                    "Module '{program}' expects exactly one argument!\n \
                    No module arguments file provided"
                ));
            }
        };

        // Now try to read from file with all ansible arguments
        let json_string: String = match read_to_string(input_file_name) {
            Ok(file_content) => file_content,
            Err(e) => {
                return Err(format!(
                    "Could not read input json file '{input_file_name}': {e}"
                ));
            }
        };

        let all_input_args: Value = match serde_json::from_str(&json_string) {
            Ok(val) => val,
            Err(e) => {
                return Err(format!(
                    "Could not parse JSON from input {json_string}: {e}"
                ));
            }
        };
        // Must be iterable too
        if !all_input_args.is_object() {
            return Err(format!("{all_input_args} must be an object"));
        }
        Ok(all_input_args)
    }

    /// Parsers all internal arguments from input JSON Value
    pub(crate) fn parse_internal_args(all_input_args: Value) -> Result<InternalArgs, String> {
        // Split input arguments to Internal and Module arguments
        let internal_args: InternalArgs = match serde_json::from_value(all_input_args.clone()) {
            Ok(arg_spec) => arg_spec,
            Err(e) => {
                return Err(format!(
                    "Could not parse input internal arguments JSON \
                    from input {all_input_args}: {e}"
                ));
            }
        };
        Ok(internal_args)
    }

    /// Parsers all module arguments from all_input_args and compares to argument_spec
    pub(crate) fn parse_module_args(
        argument_spec: ArgumentSpec,
        all_input_args: Value,
        mutually_exclusive: Option<MutuallyExclusive>,
        required_together: Option<RequiredTogether>,
        required_one_of: Option<RequiredOneOf>,
        required_if: Option<RequiredIf>,
        required_by: Option<RequiredBy>,
    ) -> Result<ModuleArgs, String> {
        let mut module_args: HashMap<String, Value> = HashMap::new();
        for (k, v) in all_input_args.as_object().unwrap().iter() {
            if !k.starts_with('_') {
                module_args.insert(k.clone(), v.clone());
            }
        }

        // Check mutually exclusive
        if let Some(mutually_exclusive) = mutually_exclusive {
            for (k, v) in mutually_exclusive.iter() {
                if module_args.contains_key(k) | module_args.contains_key(v) {
                    return Err(format!("Arguments '{k}' and '{v}' are mutually exclusive"));
                }
            }
        }

        // Check required together
        if let Some(required_together) = required_together {
            for (k, v) in required_together.iter() {
                if !(module_args.contains_key(k) & module_args.contains_key(v)) {
                    return Err(format!("Arguments '{k}' and '{v}' are required together"));
                }
            }
        }

        // Check required one of
        if let Some(required_one_of) = required_one_of {
            for (k, v) in required_one_of.iter() {
                if !(module_args.contains_key(k) | module_args.contains_key(v)) {
                    return Err(format!(
                        "At least one of the arguments '{k}' and '{v}' must be present"
                    ));
                }
            }
        }

        // TODO: Refactor and better messages
        // Check required if
        if let Some(required_if) = required_if {
            for (k, v, args, any) in required_if.iter() {
                // If not it means it it is not present anyways so we skip
                if let Some(key) = module_args.get(k) {
                    // If not equals we skip
                    if key == v {
                        // All means all args must be present
                        if *any {
                            let any_present: bool =
                                args.iter().any(|x| module_args.contains_key(x));
                            if !any_present {
                                return Err(format!(
                                    "No arguments required by '{k}'='{v}' are present"
                                ));
                            }
                        } else {
                            let all_present: bool =
                                args.iter().all(|x| module_args.contains_key(x));
                            if !all_present {
                                return Err(format!(
                                    "Not all arguments required by '{k}'='{v}' are present"
                                ));
                            }
                        }
                    }
                }
            }
        }

        // Check required by
        if let Some(required_by) = required_by {
            for (k, args) in required_by.iter() {
                // If not it means it it is not present anyways so we skip
                // We don't need the value itself, only names
                if module_args.contains_key(k) {
                    let all_present: bool = args.iter().all(|x| module_args.contains_key(x));
                    if !all_present {
                        return Err(format!(
                            "Arguments required by '{k}' '{args:?}' are not present"
                        ));
                    }
                }
            }
        }

        let mut result_params: ModuleArgs = HashMap::new();
        // Now we can compare arg spec with input arguments
        let mut missing_required_args: Vec<String> = vec![];
        for (arg_name, arg_spec) in argument_spec.iter() {
            // Check if required and not present
            if arg_spec.required & !module_args.contains_key(arg_name) {
                // Try to fallback with environment variable
                if let Some(env_var) = &arg_spec.fallback {
                    match env::var(env_var) {
                        Ok(val) => {
                            let value: Value = val.into();
                            result_params.insert(
                                arg_name.clone(),
                                ArgumentValue {
                                    value,
                                    no_log: arg_spec.no_log,
                                },
                            );
                        }
                        Err(e) => {
                            return Err(format!(
                                "'{arg_name}' is required but missing, tried \
                                fallback to {env_var} but got error: '{e}'"
                            ));
                        }
                    }
                } else {
                    missing_required_args.push(arg_name.clone());
                }
            }

            // Lastly we find the value and compare
            if let Some(arg) = module_args.get(arg_name) {
                // Check if value is in choices
                if let Some(choices) = &arg_spec.choices {
                    if choices.contains(arg) {
                        result_params.insert(
                            arg_name.clone(),
                            ArgumentValue {
                                value: arg.clone(),
                                no_log: arg_spec.no_log,
                            },
                        );
                    } else {
                        return Err(format!(
                            "Argument '{arg_name}' can only have '{choices:?}' values"
                        ));
                    }
                } else {
                    result_params.insert(
                        arg_name.clone(),
                        ArgumentValue {
                            value: arg.clone(),
                            no_log: arg_spec.no_log,
                        },
                    );
                }
            } else if let Some(default_val) = &arg_spec.default {
                result_params.insert(
                    arg_name.clone(),
                    ArgumentValue {
                        value: default_val.clone(),
                        no_log: arg_spec.no_log,
                    },
                );
            }
        }

        if !missing_required_args.is_empty() {
            return Err(format!(
                "missing required arguments: {missing_required_args:?}"
            ));
        }

        // Before inserting the value into the actual result we check for types
        for (arg_name, value) in result_params.iter() {
            if let Some(arg_spec) = argument_spec.get(arg_name) {
                let is_type_correct: bool = arg_spec.value_type.check_type_correct(&value.value);
                if !is_type_correct {
                    return Err(format!(
                        "'{arg_name}' expected to be of type '{:?}', but got {}",
                        arg_spec.value_type, value.value
                    ));
                }
            }
        }

        // At last check if there are unknown arguments and complete
        let unknown_args: Vec<String> = module_args
            .keys()
            .filter(|key| !result_params.contains_key(*key))
            .cloned()
            .collect();
        if !unknown_args.is_empty() {
            return Err(format!(
                "Unknown arguments for module found: '{unknown_args:?}'"
            ));
        }

        Ok(result_params)
    }

    /// Exits a module with custom response
    /// Note: It it reccomended to use `exit_json!` macro instead of using it directly
    ///
    /// # Arguments
    ///
    /// * `result` - A BTreeMap of String=Value values
    /// * `changed` - Decides if module has a changed status
    ///
    /// # Examples
    ///
    /// ```
    /// use ansible_module::{AnsibleModule, exit_json};
    /// use serde_json::{json, Value};
    /// use std::collections::BTreeMap;
    ///
    /// let arg_spec: Value = json!({
    ///     "msg": {},
    ///     "additional": {
    ///         "no_log": true
    ///     },
    /// });
    /// let module = AnsibleModule::new(arg_spec, None, None, None, None, None, None).unwrap();
    /// let mut result = BTreeMap::new();
    /// result.insert("msg".to_string(), json!("All good!"));
    /// module.exit_json(result, false);
    /// ```
    pub fn exit_json(self, result: BTreeMap<String, Value>, changed: bool) -> ! {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exit_json;
    use serde_json::{Value, json};
    use std::io::Write;
    use std::vec;
    use tempfile::NamedTempFile;

    #[test]
    fn check_required() {
        let input_string: String = r#"
            {
                "api_url": "localhost"
            }"#
        .to_string();
        let arg_spec: Value = json!({
            "api_url": {
                "type": "str",
                "required": true
            },
            "username": {
                "type": "str",
                "required": false
            },
            "password": {
                "type": "str"
            }
        });

        let mut file: NamedTempFile = NamedTempFile::new().unwrap();
        writeln!(file, "{input_string}").unwrap();
        let input_args: Vec<String> = vec![
            "module_name".to_string(),
            // TODO: Refactor this monstruosity
            file.path().to_str().unwrap().to_string(),
        ];

        let module: AnsibleModule =
            AnsibleModule::new(arg_spec, None, None, None, None, None, Some(input_args)).unwrap();

        assert_eq!("localhost", module.params.get("api_url").unwrap().value);
        assert!(!module.params.contains_key("username"));
        assert!(!module.params.contains_key("password"));
    }

    #[test]
    #[should_panic(
        expected = r#"{"msg":"missing required arguments: [\"api_url\"]","changed":false,"failed":true}"#
    )]
    fn check_required_fail() {
        let arg_spec: Value = json!({
            "api_url": {
                "type": "str",
                "required": true
            },
            "password": {
                "type": "str"
            }
        });
        let input_string: String = r#"
            {
                "password": "123"
            }"#
        .to_string();

        let mut file: NamedTempFile = NamedTempFile::new().unwrap();
        writeln!(file, "{input_string}").unwrap();
        let input_args: Vec<String> = vec![
            "module_name".to_string(),
            file.path().to_str().unwrap().to_string(),
        ];

        let _: ! = AnsibleModule::new(arg_spec, None, None, None, None, None, Some(input_args))
            .unwrap_err();
    }

    #[test]
    #[should_panic(
        expected = r#"{"changed":false,"failed":false,"api_url":"VALUE_SPECIFIED_IN_NO_LOG_PARAMETER"}"#
    )]
    fn check_no_log() {
        let input_string: String = r#"
            {
                "api_url": "localhost",
                "_ansible_no_log": true
            }"#
        .to_string();
        let arg_spec: Value = json!({
            "api_url": {
                "type": "str",
                "required": true,
                "no_log": true
            },
        });

        let mut file: NamedTempFile = NamedTempFile::new().unwrap();
        writeln!(file, "{input_string}").unwrap();
        let input_args: Vec<String> = vec![
            "module_name".to_string(),
            // TODO: Refactor this monstruosity
            file.path().to_str().unwrap().to_string(),
        ];

        let module: AnsibleModule =
            AnsibleModule::new(arg_spec, None, None, None, None, None, Some(input_args)).unwrap();
        exit_json!(module, "api_url" => module.params.get("api_url").unwrap().clone().value);
    }

    #[test]
    fn check_default() {
        let arg_spec: Value = json!({
            "api_url": {
                "type": "int",
                "default": 123
            },
        });
        let input_string: String = r#"{}"#.to_string();

        let mut file: NamedTempFile = NamedTempFile::new().unwrap();
        writeln!(file, "{input_string}").unwrap();
        let input_args: Vec<String> = vec![
            "module_name".to_string(),
            file.path().to_str().unwrap().to_string(),
        ];

        let module: AnsibleModule =
            AnsibleModule::new(arg_spec, None, None, None, None, None, Some(input_args)).unwrap();
        assert_eq!(123, module.params.get("api_url").unwrap().value);
    }

    #[test]
    fn check_choices() {
        let arg_spec: Value = json!({
            "api_url": {
                "type": "str",
                "choices": [
                    "localhost",
                    "127.0.0.1",
                    "::1"
                ]
            },
        });
        let input_string: String = r#"
            {
                "api_url": "localhost"
            }"#
        .to_string();

        let mut file: NamedTempFile = NamedTempFile::new().unwrap();
        writeln!(file, "{input_string}").unwrap();
        let input_args: Vec<String> = vec![
            "module_name".to_string(),
            file.path().to_str().unwrap().to_string(),
        ];

        let module: AnsibleModule =
            AnsibleModule::new(arg_spec, None, None, None, None, None, Some(input_args)).unwrap();
        assert_eq!("localhost", module.params.get("api_url").unwrap().value);
    }

    #[test]
    #[should_panic(
        expected = r#"{"msg":"Argument 'api_url' can only have '[String(\"localhost\"), String(\"127.0.0.1\"), String(\"::1\")]' values","changed":false,"failed":true}"#
    )]
    fn check_choices_fail() {
        let arg_spec: Value = json!({
            "api_url": {
                "type": "str",
                "choices": [
                    "localhost",
                    "127.0.0.1",
                    "::1"
                ]
            },
        });
        let input_string: String = r#"
            {
                "api_url": "192.0.2.1"
            }"#
        .to_string();

        let mut file: NamedTempFile = NamedTempFile::new().unwrap();
        writeln!(file, "{input_string}").unwrap();
        let input_args: Vec<String> = vec![
            "module_name".to_string(),
            file.path().to_str().unwrap().to_string(),
        ];

        let _: ! = AnsibleModule::new(arg_spec, None, None, None, None, None, Some(input_args))
            .unwrap_err();
    }

    // UNSAFE TO USE WITH TEST BELOW
    // TODO: Make it work
    //
    // #[test]
    // fn check_fallback() {
    //     // Safety: string is hard-coded + it's a test
    //     unsafe {
    //         env::set_var("TEST_API_URL", "Hello");
    //     }
    //     let arg_spec: Value = json!({
    //         "api_url": {
    //             "required": true,
    //             "fallback": "TEST_API_URL"
    //         },
    //     });

    //     let input_string: String = r#"
    //         {
    //             "ANSIBLE_MODULE_ARGS": {}
    //         }"#.to_string();

    //     let params: (ModuleArgs, InternalArgs) = AnsibleModule::parse_args(
    //         arg_spec,
    //         input_string,
    //         None,
    //         None,
    //         None,
    //         None,
    //         None,
    //     );
    //     // Safety: string is hard-coded + it's a test
    //     unsafe {
    //         env::remove_var("TEST_API_URL");
    //     }
    //     assert_eq!("Hello", *params.0.get("api_url").unwrap());
    // }

    #[test]
    #[should_panic(
        expected = r#"{"msg":"'api_url' is required but missing, tried fallback to TEST_API_URL but got error: 'environment variable not found'","changed":false,"failed":true}"#
    )]
    fn check_fallback_fail() {
        let arg_spec: Value = json!({
            "api_url": {
                "type": "str",
                "required": true,
                "fallback": "TEST_API_URL"
            },
        });
        let input_string: String = r#"{}"#.to_string();

        let mut file: NamedTempFile = NamedTempFile::new().unwrap();
        writeln!(file, "{input_string}").unwrap();
        let input_args: Vec<String> = vec![
            "module_name".to_string(),
            file.path().to_str().unwrap().to_string(),
        ];

        let _: ! = AnsibleModule::new(arg_spec, None, None, None, None, None, Some(input_args))
            .unwrap_err();
    }

    #[test]
    #[should_panic(
        expected = r#"{"msg":"Arguments 'api_url' and 'url' are mutually exclusive","changed":false,"failed":true}"#
    )]
    fn check_mutually_exclusive_fail() {
        let mutually_exclusive: MutuallyExclusive =
            vec![("api_url".to_string(), "url".to_string())];
        let arg_spec: Value = json!({
            "api_url": {
                "type": "str"
            },
            "url": {
                "type": "str"
            },
        });
        let input_string: String = r#"
            {
                "api_url": 123,
                "url": 123
            }"#
        .to_string();

        let mut file: NamedTempFile = NamedTempFile::new().unwrap();
        writeln!(file, "{input_string}").unwrap();
        let input_args: Vec<String> = vec![
            "module_name".to_string(),
            file.path().to_str().unwrap().to_string(),
        ];

        let _: ! = AnsibleModule::new(
            arg_spec,
            Some(mutually_exclusive),
            None,
            None,
            None,
            None,
            Some(input_args),
        )
        .unwrap_err();
    }

    #[test]
    #[should_panic(
        expected = r#"{"msg":"Arguments 'api_url' and 'url' are required together","changed":false,"failed":true}"#
    )]
    fn check_required_together_fail() {
        let required_together: RequiredTogether = vec![("api_url".to_string(), "url".to_string())];
        let arg_spec: Value = json!({
            "api_url": {
                "type": "str"
            },
            "url": {
                "type": "str"
            },
        });
        let input_string: String = r#"
            {
                "api_url": 123
            }"#
        .to_string();

        let mut file: NamedTempFile = NamedTempFile::new().unwrap();
        writeln!(file, "{input_string}").unwrap();
        let input_args: Vec<String> = vec![
            "module_name".to_string(),
            file.path().to_str().unwrap().to_string(),
        ];

        let _: ! = AnsibleModule::new(
            arg_spec,
            None,
            Some(required_together),
            None,
            None,
            None,
            Some(input_args),
        )
        .unwrap_err();
    }

    #[test]
    #[should_panic(
        expected = r#"{"msg":"At least one of the arguments 'api_url' and 'url' must be present","changed":false,"failed":true}"#
    )]
    fn check_required_one_of_fail() {
        let required_one_of: RequiredOneOf = vec![("api_url".to_string(), "url".to_string())];
        let arg_spec: Value = json!({
            "api_url": {
                "type": "str"
            },
            "url": {
                "type": "str"
            },
        });
        let input_string: String = r#"{}"#.to_string();

        let mut file: NamedTempFile = NamedTempFile::new().unwrap();
        writeln!(file, "{input_string}").unwrap();
        let input_args: Vec<String> = vec![
            "module_name".to_string(),
            file.path().to_str().unwrap().to_string(),
        ];

        let _: ! = AnsibleModule::new(
            arg_spec,
            None,
            None,
            Some(required_one_of),
            None,
            None,
            Some(input_args),
        )
        .unwrap_err();
    }

    #[test]
    #[should_panic(
        expected = r#"{"msg":"Not all arguments required by 'login'='true' are present","changed":false,"failed":true}"#
    )]
    fn check_required_if_all_fail() {
        let required_if: RequiredIf = vec![(
            "login".to_string(),
            Value::Bool(true),
            vec!["user".to_string(), "password".to_string()],
            false,
        )];
        let arg_spec: Value = json!({
            "api_url": {
                "type": "str"
            },
            "url": {
                "type": "str"
            },
            "login": {
                "type": "bool"
            }
        });
        let input_string: String = r#"
            {
                "login": true,
                "user": "John"
            }"#
        .to_string();

        let mut file: NamedTempFile = NamedTempFile::new().unwrap();
        writeln!(file, "{input_string}").unwrap();
        let input_args: Vec<String> = vec![
            "module_name".to_string(),
            file.path().to_str().unwrap().to_string(),
        ];

        let _: ! = AnsibleModule::new(
            arg_spec,
            None,
            None,
            None,
            Some(required_if),
            None,
            Some(input_args),
        )
        .unwrap_err();
    }

    #[test]
    #[should_panic(
        expected = r#"{"msg":"No arguments required by 'login'='true' are present","changed":false,"failed":true}"#
    )]
    fn check_required_if_any_fail() {
        let required_if: RequiredIf = vec![(
            "login".to_string(),
            Value::Bool(true),
            vec!["user".to_string(), "password".to_string()],
            true,
        )];
        let arg_spec: Value = json!({
            "api_url": {
                "type": "str"
            },
            "url": {
                "type": "str"
            },
            "login": {
                "type": "bool"
            }
        });
        let input_string: String = r#"
            {
                "login": true
            }"#
        .to_string();

        let mut file: NamedTempFile = NamedTempFile::new().unwrap();
        writeln!(file, "{input_string}").unwrap();
        let input_args: Vec<String> = vec![
            "module_name".to_string(),
            file.path().to_str().unwrap().to_string(),
        ];

        let _: ! = AnsibleModule::new(
            arg_spec,
            None,
            None,
            None,
            Some(required_if),
            None,
            Some(input_args),
        )
        .unwrap_err();
    }

    #[test]
    #[should_panic(
        expected = r#"{"msg":"Arguments required by 'login' '[\"user\", \"password\"]' are not present","changed":false,"failed":true}"#
    )]
    fn check_required_by_fail() {
        let required_by: RequiredBy = vec![(
            "login".to_string(),
            vec!["user".to_string(), "password".to_string()],
        )];
        let arg_spec: Value = json!({
            "api_url": {
                "type": "str"
            },
            "url": {
                "type": "str"
            },
            "login": {
                "type": "bool"
            }
        });
        let input_string: String = r#"
            {
                "login": true
            }"#
        .to_string();

        let mut file: NamedTempFile = NamedTempFile::new().unwrap();
        writeln!(file, "{input_string}").unwrap();
        let input_args: Vec<String> = vec![
            "module_name".to_string(),
            file.path().to_str().unwrap().to_string(),
        ];

        let _: ! = AnsibleModule::new(
            arg_spec,
            None,
            None,
            None,
            None,
            Some(required_by),
            Some(input_args),
        )
        .unwrap_err();
    }

    #[test]
    fn check_internal_args() {
        let arg_spec: Value = json!({
            "hello": {
                "type": "str"
            }
        });
        let input_string: String = r#"
            {
                "_ansible_check_mode": false,
                "_ansible_no_log": false,
                "_ansible_debug": false,
                "_ansible_diff": false,
                "_ansible_verbosity": 0,
                "_ansible_version": "2.18.2",
                "_ansible_module_name": "ansible_mod",
                "_ansible_syslog_facility": "LOG_USER",
                "_ansible_selinux_special_fs": [
                    "fuse",
                    "nfs"
                ],
                "_ansible_string_conversion_action": "warn",
                "_ansible_socket": null,
                "_ansible_shell_executable": "/bin/sh",
                "_ansible_keep_remote_files": false,
                "_ansible_tmpdir": "/home/alexander/.ansible/tmp/ansible-tmp-1751129048.123-321-123/",
                "_ansible_remote_tmp": "~/.ansible/tmp",
                "_ansible_ignore_unknown_opts": false,
                "_ansible_target_log_info": null
            }"#.to_string();

        let mut file: NamedTempFile = NamedTempFile::new().unwrap();
        writeln!(file, "{input_string}").unwrap();
        let input_args: Vec<String> = vec![
            "module_name".to_string(),
            file.path().to_str().unwrap().to_string(),
        ];

        let module: AnsibleModule =
            AnsibleModule::new(arg_spec, None, None, None, None, None, Some(input_args)).unwrap();
        let params: InternalArgs = module.internal_params;

        assert!(!params.no_log);
        assert!(!params.debug);
        assert!(!params.check_mode);
        assert!(!params.diff);
        assert_eq!(params.verbosity, 0_u8);
        assert_eq!(params.socket, None);
        assert_eq!(params.target_log_info, None);
        assert!(!params.ignore_unknown_opts);
        assert!(!params.keep_remote_files);
        assert_eq!(params.string_conversion_action.unwrap(), "warn".to_string());
        assert_eq!(params.version.unwrap(), "2.18.2".to_string());
        assert_eq!(params.module_name.unwrap(), "ansible_mod".to_string());
        assert_eq!(params.syslog_facility.unwrap(), "LOG_USER".to_string());
        assert_eq!(
            params.selinux_special_fs,
            vec!["fuse".to_string(), "nfs".to_string()]
        );
        assert_eq!(
            params.tmpdir.unwrap(),
            "/home/alexander/.ansible/tmp/ansible-tmp-1751129048.123-321-123/".to_string()
        );
        assert_eq!(params.remote_tmp.unwrap(), "~/.ansible/tmp".to_string());
    }

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
            // TODO: Refactor this monstruosity
            file.path().to_str().unwrap().to_string(),
        ];

        let module: AnsibleModule =
            AnsibleModule::new(arg_spec, None, None, None, None, None, Some(input_args)).unwrap();

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

    #[test]
    #[should_panic(
        expected = r#"{"msg":"'uint' expected to be of type 'Uint', but got -1","changed":false,"failed":true}"#
    )]
    fn check_type_fail() {
        let arg_spec: Value = json!({
            "uint": {
                "type": "uint",
                "default": 123
            },
        });
        let input_string: String = r#"
            {
                "uint": -1
            }"#
        .to_string();

        let mut file: NamedTempFile = NamedTempFile::new().unwrap();
        writeln!(file, "{input_string}").unwrap();
        let input_args: Vec<String> = vec![
            "module_name".to_string(),
            file.path().to_str().unwrap().to_string(),
        ];

        let _: ! = AnsibleModule::new(arg_spec, None, None, None, None, None, Some(input_args))
            .unwrap_err();
    }
}
