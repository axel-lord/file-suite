#![doc = include_str!("../README.md")]

use ::std::str::FromStr;

use ::convert_case::{Case, Casing};
use ::quote::format_ident;
use ::serde_json::Value;

/// Convert tools.json to rust code.
pub fn tool_json_to_rust(json: String) -> String {
    let tools = ::serde_json::Value::from_str(&json).unwrap();
    let tools = match tools {
        Value::Array(tools) => tools,
        other => panic!("tool json should be an array of strings, is:\n{other:#?}"),
    };
    let tools = tools
        .into_iter()
        .map(|tool| match tool {
            Value::String(tool) => tool,
            other => panic!("tool json should be an array of strings, one value is:\n{other:#?}"),
        })
        .collect::<Vec<_>>();
    let (tools_pascal, modules, tool_names, module_names) = tools
        .iter()
        .map(|tool| {
            let kebab = tool.to_case(Case::Kebab);
            let pascal = kebab.from_case(Case::Kebab).to_case(Case::Pascal);
            let snake = kebab.from_case(Case::Kebab).to_case(Case::Snake);
            (
                format_ident!("{pascal}"),
                format_ident!("{snake}"),
                kebab,
                snake,
            )
        })
        .collect::<(Vec<_>, Vec<_>, Vec<_>, Vec<_>)>();

    ::prettyplease::unparse(&::syn::parse_quote! {
        /// Modules to allow logging for.
        pub const MODULES: &[&str] = &[ "file_suite" #(, #module_names)* ];

        /// Get cli and used modules from tool name.
        pub fn get_cli(name: &str) -> (fn() -> &'static dyn Start, &'static [&'static str]) {
            // Quick path for compilation tool.
            if name == "file-suite" {
                return (|| startable::<Cli>(), MODULES);
            }
            match name {
                #( #tool_names => (|| startable::<::#modules::Cli>(), &[ #module_names ]), )*
                _ => (|| startable::<Cli>(), MODULES),
            }
        }

        // Selection of cli tool.
        #[derive(Debug, Subcommand, Run)]
        #[run(error = Report)]
        enum CliSubcmd {
            // generate completions for a tool
            Completions(CmpSubcmd),
            #( #tools_pascal(::#modules::Cli), )*
        }

        /// Module to generate completions for
        #[derive(Debug, ValueEnum, Clone, Copy, PartialEq, Eq, Hash, Default)]
        enum CompletionTarget {
            #[default]
            FileSuite,
            #( #tools_pascal, )*
        }

        impl CompletionTarget {
            /// Get a startable based on the selected tool.
            fn startable(self) -> &'static dyn Start {
                match self {
                    Self::FileSuite => startable::<Cli>(),
                    #(
                    Self::  #tools_pascal => startable::<::#modules::Cli>(),
                    )*
                }
            }

            /// Get name of startable.
            const fn mod_name(self) -> &'static str {
                match self {
                    Self::FileSuite => "file_suite",
                    #(
                    Self:: #tools_pascal => #module_names,
                    )*
                }
            }
        }
    })
}
