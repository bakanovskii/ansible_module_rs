#![feature(never_type)]

pub mod ansible_module;
pub mod builder;
pub mod macros;

pub use ansible_module::AnsibleModule;
pub use builder::AnsibleModuleBuilder;
