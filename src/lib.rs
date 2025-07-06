#![feature(never_type)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::diverging_sub_expression)]

pub mod ansible_module;
pub mod macros;

pub use ansible_module::AnsibleModule;
