use ansible_module::{AnsibleModule, AnsibleModuleBuilder, exit_json, fail_json};
use base64::{Engine, engine::general_purpose};
use serde_json::{Value, json};
use std::{fs::File, io::Read, path::PathBuf};

fn read_file_to_b64(source: &PathBuf) -> Result<String, String> {
    if !source.exists() {
        return Err(format!("File not found: {}", source.display()));
    }

    if source.is_dir() {
        return Err(format!(
            "Source is a directory and must be a file: {}",
            source.display()
        ));
    }

    let mut file: File = match File::open(source) {
        Ok(file) => file,
        Err(e) => {
            return Err(format!("Unable to slurp file: {e}"));
        }
    };
    let mut buffer: Vec<u8> = Vec::new();
    if let Err(e) = file.read_to_end(&mut buffer) {
        return Err(format!("Unable to slurp file: {e}"));
    }

    Ok(general_purpose::STANDARD.encode(buffer))
}

// See <https://github.com/ansible/ansible/blob/devel/lib/ansible/modules/slurp.py>
fn main() {
    let arg_spec: Value = json!({
        "src": {
            "type": "str",
            "required": true
        }
    });
    let module: AnsibleModule = AnsibleModuleBuilder::new(arg_spec, None)
        .build()
        .unwrap_or_else(|e| fail_json!(e));

    // Safe to unwrap here, all check were validated in the AnsibleModule::new(0)
    let src_arg: &str = module.params.get("src").unwrap().value.as_str().unwrap();
    let path_buf: PathBuf = PathBuf::from(src_arg);

    match read_file_to_b64(&path_buf) {
        Ok(b64_str) => {
            exit_json!(
                module,
                "content" => json!(b64_str),
                "encoding" => json!("base64"),
                "source" => json!(path_buf.display().to_string())
            );
        }
        Err(e) => {
            fail_json!(e)
        }
    }
}
