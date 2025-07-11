# ansible_module_rs

This is a framework to write [binary modules](https://docs.ansible.com/ansible/latest/dev_guide/developing_program_flow_modules.html#binary-modules) using Rust languange

It tries to be somewhat similar to a Python `AnsibleModule` from `ansible.module_utils.basic`

## Why

You can write binary modules in Rust if performance is critical but keep in mind that Ansible itself is not very fast so if your playbook runs slow rewriting modules from Python would not help. If your goal is to optimize the overall performance rather than optimising one particular bottlenecked module you should use builtin features such as `pipelining`, `smart gathering` and etc.

## What works for now

Compared to https://docs.ansible.com/ansible/latest/dev_guide/developing_program_flow_modules.html#argument-spec

Dependencies between module options (see: https://docs.ansible.com/ansible/latest/dev_guide/developing_program_flow_modules.html#dependencies-between-module-options):
  - [x] ~~mutually_exclusive~~
  - [x] ~~required_together~~
  - [x] ~~required_one_of~~
  - [x] ~~required_if~~
  - [x] ~~required_by~~

Module arguments:
  - [x] ~~required~~
  - [x] ~~default~~
  - [x] ~~fallback~~
  - [x] ~~choices~~
  - [x] ~~required_by~~
  - [x] ~~type validation~~
  - [ ] elements
  - [x] ~~no_log~~
  - [ ] aliases
  - [ ] options
  - [ ] apply_defaults
  - [ ] removed_in_version
  - [ ] removed_at_date
  - [ ] removed_from_collection
  - [ ] deprecated_aliases

Methods to use internal arguments (see: https://docs.ansible.com/ansible/latest/dev_guide/developing_program_flow_modules.html#internal-arguments), for now it deserializes these arguments but makes no use of them:
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
