# ansible_module_rs

This is a framework to write [binary modules](https://docs.ansible.com/ansible/latest/dev_guide/developing_program_flow_modules.html#binary-modules) using Rust languange

It tries to be somewhat similar to a Python `AnsibleModule` from `ansible.module_utils.basic`

## What works for now

Compared to https://docs.ansible.com/ansible/latest/dev_guide/developing_program_flow_modules.html#argument-spec

- [x] Dependencies between module options (see: https://docs.ansible.com/ansible/latest/dev_guide/developing_program_flow_modules.html#dependencies-between-module-options):
  - [x] mutually_exclusive
  - [x] required_together
  - [x] required_one_of
  - [x] required_if
  - [x] required_by

- [ ] Module arguments:
  - [x] required
  - [x] default
  - [x] fallback
  - [x] choices
  - [x] required_by
  - [ ] type validation
  - [ ] elements
  - [ ] no_log
  - [ ] aliases
  - [ ] options
  - [ ] apply_defaults
  - [ ] removed_in_version
  - [ ] removed_at_date
  - [ ] removed_from_collection
  - [ ] deprecated_aliases

- [ ] Methods to use internal arguments (see: https://docs.ansible.com/ansible/latest/dev_guide/developing_program_flow_modules.html#internal-arguments), for now it deserializes these arguments but makes no use of them:
  - [ ] no_log
  - [ ] verbosity
  - [ ] check_mode
  - [ ] diff
  - [ ] verbosity
  - [ ] socket
  - [ ] target_log_info
  - [ ] ignore_unknown_opts
  - [ ] keep_remote_files
  - [ ] string_conversion_action
  - [ ] version
  - [ ] module_name
  - [ ] syslog_facility
  - [ ] selinux_special_fs
  - [ ] shell_executable
  - [ ] tmpdir
  - [ ] remote_tmp
