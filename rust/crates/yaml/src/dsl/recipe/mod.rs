//! Recipe document root (`recipe.schema.json`).

pub mod field {
    pub const ID: &str = "id";
    pub const NAME: &str = "name";
    pub const DESCRIPTION: &str = "description";
    pub const ARGS: &str = "args";
    pub const MAPS: &str = "maps";
    pub const QUERIES: &str = "queries";
    pub const STEPS: &str = "steps";
    pub const POST_EXECUTION: &str = "postExecution";
}

/// Recipe `args[]` item (`#/definitions/arg`).
pub mod arg {
    pub mod field {
        pub const NAME: &str = "name";
        pub const REQUIRED: &str = "required";
        pub const INPUT_KIND: &str = "inputKind";
        pub const ABBR: &str = "abbr";
        pub const HELP: &str = "help";
        pub const DEFAULTS_TO: &str = "defaultsTo";
        pub const OPTIONS: &str = "options";
        pub const ALLOW_CUSTOM_VALUE: &str = "allowCustomValue";
        pub const CONTEXT_KEY: &str = "contextKey";

        pub mod input_kind {
            pub mod value {
                pub const TEXT: &str = "text";
                pub const FILE: &str = "file";
                pub const DIRECTORY: &str = "directory";
                pub const CHOICE: &str = "choice";
            }
        }
    }
}

/// Named query library entry under recipe `queries`.
pub mod queries {
    pub mod entry {
        pub mod field {
            pub const QUERY: &str = "query";
        }
    }
}

/// Step kinds under `steps[]` (`#/definitions/step`).
pub mod steps {
    /// Edit step (`#/definitions/editStep`).
    pub mod edit {
        pub const WIRE: &str = "edit";

        pub mod field {
            pub const PATH: &str = "path";
            pub const LANGUAGE: &str = "language";
            pub const WHEN: &str = "when";
            pub const WHEN_NOT: &str = "whenNot";
            pub const LET: &str = "let";
            pub const OPS: &str = "ops";
        }

        /// Let binding (`#/definitions/letBinding`).
        pub mod let_binding {
            pub mod field {
                pub const NAME: &str = "name";
                pub const QUERY: &str = "query";
                pub const CAPTURE: &str = "capture";
                pub const EXTRACT: &str = "extract";
                pub const ON_NO_MATCH: &str = "onNoMatch";
                pub const ON_MANY_MATCHES: &str = "onManyMatches";
                pub const JOIN: &str = "join";
                pub const AS: &str = "as";

                pub mod extract {
                    pub mod value {
                        pub const TEXT: &str = "text";
                        pub const KIND: &str = "kind";
                        pub const EXISTS: &str = "exists";
                        pub const COUNT: &str = "count";
                    }
                }

                pub mod on_no_match {
                    pub mod value {
                        pub const ERROR: &str = "error";
                        pub const USE: &str = "use";
                    }
                }

                pub mod on_many_matches {
                    pub mod value {
                        pub const ERROR: &str = "error";
                        pub const FIRST: &str = "first";
                        pub const JOIN: &str = "join";
                    }
                }
            }
        }

        /// Edit `ops` list (`#/definitions/editOp` oneOf).
        pub mod ops {
            pub const WIRE: &str = "ops";

            /// Insert op (`#/definitions/insertOp`).
            pub mod insert {
                pub const WIRE: &str = "insert";

                pub mod field {
                    pub const QUERY: &str = "query";
                    pub const CAPTURE: &str = "capture";
                    pub const ANCHOR: &str = "anchor";
                    pub const TEXT: &str = "text";

                    pub mod anchor {
                        pub mod value {
                            pub const START: &str = "start";
                            pub const END: &str = "end";
                        }
                    }
                }
            }

            /// Replace op (`#/definitions/replaceOp`).
            pub mod replace {
                pub const WIRE: &str = "replace";

                pub mod field {
                    pub const QUERY: &str = "query";
                    pub const CAPTURE: &str = "capture";
                    pub const TEXT: &str = "text";
                    pub const INCLUDE_LEADING_TRIVIA: &str = "includeLeadingTrivia";
                }
            }

            /// Remove op (`#/definitions/removeOp`).
            pub mod remove {
                pub const WIRE: &str = "remove";

                pub mod field {
                    pub const QUERY: &str = "query";
                    pub const CAPTURE: &str = "capture";
                    pub const INCLUDE_LEADING_TRIVIA: &str = "includeLeadingTrivia";
                }
            }
        }
    }

    /// Create step (`#/definitions/createStep`).
    pub mod create {
        pub const WIRE: &str = "create";

        pub mod field {
            pub const PATH: &str = "path";
            pub const TEMPLATE: &str = "template";
            pub const TEMPLATE_FILE: &str = "templateFile";
            pub const IF_EXISTS: &str = "ifExists";

            pub mod if_exists {
                pub mod value {
                    pub const FAIL: &str = "fail";
                    pub const SKIP: &str = "skip";
                }
            }
        }
    }

    /// Delete step (`#/definitions/deleteStep`).
    pub mod delete {
        pub const WIRE: &str = "delete";

        pub mod field {
            pub const PATH: &str = "path";
            pub const IF_MISSING: &str = "ifMissing";

            pub mod if_missing {
                pub mod value {
                    pub const FAIL: &str = "fail";
                    pub const SKIP: &str = "skip";
                }
            }
        }
    }

    /// Recipe reference step (`#/definitions/recipeRef` object form).
    pub mod recipe_ref {
        pub const WIRE: &str = "recipe";

        pub mod object {
            pub mod field {
                pub const ID: &str = "id";
                pub const WITH: &str = "with";
            }
        }
    }
}
