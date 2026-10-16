use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub grammar);

#[cfg(test)]
mod test {
    use super::grammar;
    use crate::ast;

    #[test]
    fn parse_input() {
        let parser = grammar::InputParser::new();

        assert!(parser.parse("state counter = 3;").is_ok_and(|result| result
            == ast::ScopeDeclaration {
                kind: ast::ScopeKind::Global,
                children: vec![ast::InScopeDeclaration::State(ast::StateDeclaration {
                    name: "counter".to_owned(),
                    value: ast::Literal::UInt(3)
                })]
            }));
        assert!(parser
            .parse("alias Flex = widget::FlexBox;")
            .is_ok_and(|result| result
                == ast::ScopeDeclaration {
                    kind: ast::ScopeKind::Global,
                    children: vec![ast::InScopeDeclaration::Alias(ast::AliasDeclaration {
                        name: "Flex".to_owned(),
                        replacement: ast::AliasReplacement::Path(ast::TypePath(vec![
                            "widget".to_owned(),
                            "FlexBox".to_owned()
                        ]))
                    })]
                }));

        assert!(parser
            .parse(
                "
                state body_state = \"value\";
                alias TickBorder = Border(size = 8);

                FlexBox(
                    direction = \"horizontal\",
                    border = TickBorder(),
                ) {
                    Text(value = body_state) {}
                }"
            )
            .is_ok_and(|result| result
                == ast::ScopeDeclaration {
                    kind: ast::ScopeKind::Global,
                    children: vec![
                        ast::InScopeDeclaration::State(ast::StateDeclaration {
                            name: "body_state".to_owned(),
                            value: ast::Literal::QuotedString("\"value\"".to_owned())
                        }),
                        ast::InScopeDeclaration::Alias(ast::AliasDeclaration {
                            name: "TickBorder".to_owned(),
                            replacement: ast::AliasReplacement::Type(ast::TypeDeclaration {
                                type_path: ast::TypePath(vec!["Border".to_owned()]),
                                fields: vec![ast::Field {
                                    name: "size".to_owned(),
                                    value: ast::FieldValue::Literal(ast::Literal::UInt(8)),
                                }]
                            })
                        }),
                        ast::InScopeDeclaration::Widget(ast::ScopeDeclaration {
                            kind: ast::ScopeKind::Widget(ast::WidgetDeclaration {
                                type_path: ast::TypePath(vec!["FlexBox".to_owned()]),
                                fields: vec![
                                    ast::Field {
                                        name: "direction".to_owned(),
                                        value: ast::FieldValue::Literal(
                                            ast::Literal::QuotedString("\"horizontal\"".to_owned())
                                        )
                                    },
                                    ast::Field {
                                        name: "border".to_owned(),
                                        value: ast::FieldValue::Type(ast::TypeDeclaration {
                                            type_path: ast::TypePath(vec!["TickBorder".to_owned()]),
                                            fields: vec![]
                                        })
                                    }
                                ]
                            }),
                            children: vec![ast::InScopeDeclaration::Widget(
                                ast::ScopeDeclaration {
                                    kind: ast::ScopeKind::Widget(ast::WidgetDeclaration {
                                        type_path: ast::TypePath(vec!["Text".to_owned()]),
                                        fields: vec![ast::Field {
                                            name: "value".to_owned(),
                                            value: ast::FieldValue::State("body_state".to_owned())
                                        }]
                                    }),
                                    children: vec![]
                                }
                            )]
                        })
                    ]
                }));

        // INFO: Empty input is not acceptable.
        assert!(parser.parse("").is_err());
        assert!(parser
            .parse(
                "
                state body_state = \"value\";
                alias TickBorder = Border(size = 8)

                FlexBox(
                    direction = \"horizontal\",
                    border = TickBorder(),
                ) {
                    Text(value = body_state) {}
                }"
            )
            .is_err());
        assert!(parser
            .parse(
                "
                state body_state = \"value\"
                alias TickBorder = Border(size = 8);

                FlexBox(
                    direction = \"horizontal\",
                    border = TickBorder(),
                ) {
                    Text(value = body_state) {}
                }"
            )
            .is_err());

        // INFO: Widgets are declared with "{}" introducing a scope, even if a widget cannot have children.
        assert!(parser
            .parse(
                "
                state body_state = \"value\"
                alias TickBorder = Border(size = 8);

                FlexBox(
                    direction = \"horizontal\",
                    border = TickBorder(),
                ) {
                    Text(value = body_state)
                }"
            )
            .is_err());
    }

    #[test]
    fn parse_alias_declaration() {
        let parser = grammar::AliasDeclarationParser::new();

        assert!(parser
            .parse("alias Row = widget::FlexBox(direction = \"vertical\") {};")
            .is_ok_and(|result| result
                == ast::AliasDeclaration {
                    name: "Row".to_owned(),
                    replacement: ast::AliasReplacement::Widget(ast::ScopeDeclaration {
                        kind: ast::ScopeKind::Widget(ast::WidgetDeclaration {
                            type_path: ast::TypePath(vec![
                                "widget".to_owned(),
                                "FlexBox".to_owned()
                            ]),
                            fields: vec![ast::Field {
                                name: "direction".to_owned(),
                                value: ast::FieldValue::Literal(ast::Literal::QuotedString(
                                    "\"vertical\"".to_owned()
                                ))
                            }],
                        }),
                        children: vec![],
                    })
                }));

        assert!(parser
            .parse("alias TickBorder = Border(size = 8);")
            .is_ok_and(|result| result
                == ast::AliasDeclaration {
                    name: "TickBorder".to_owned(),
                    replacement: ast::AliasReplacement::Type(ast::TypeDeclaration {
                        type_path: ast::TypePath(vec!["Border".to_owned()]),
                        fields: vec![ast::Field {
                            name: "size".to_owned(),
                            value: ast::FieldValue::Literal(ast::Literal::UInt(8)),
                        }]
                    })
                }));

        assert!(parser
            .parse("alias Flex = widget::FlexBox;")
            .is_ok_and(|result| result
                == ast::AliasDeclaration {
                    name: "Flex".to_owned(),
                    replacement: ast::AliasReplacement::Path(ast::TypePath(vec![
                        "widget".to_owned(),
                        "FlexBox".to_owned()
                    ]))
                }));

        assert!(parser
            .parse("alias _ = widget::FlexBox(direction = \"vertical\") {};")
            .is_err());
        assert!(parser.parse("alias _ = SomeShit;").is_err());
        assert!(parser.parse("alias 123 = Property();").is_err());
        assert!(parser.parse("some_alias = Property();").is_err());
        assert!(parser.parse("some_alias = Property()").is_err());
        assert!(parser
            .parse("alias Row = widget::FlexBox(direction = \"vertical\") {}")
            .is_err());
    }

    #[test]
    fn parse_alias_replacement() {
        let parser = grammar::AliasReplacementParser::new();

        assert!(parser.parse("widget::FlexBox").is_ok_and(|result| result
            == ast::AliasReplacement::Path(ast::TypePath(vec![
                "widget".to_owned(),
                "FlexBox".to_owned()
            ]))));

        assert!(parser.parse("widget::FlexBox()").is_ok_and(|result| result
            == ast::AliasReplacement::Type(ast::TypeDeclaration {
                type_path: ast::TypePath(vec!["widget".to_owned(), "FlexBox".to_owned()]),
                fields: vec![],
            })));

        assert!(parser
            .parse("widget::FlexBox(direction = \"horizontal\")")
            .is_ok_and(|result| result
                == ast::AliasReplacement::Type(ast::TypeDeclaration {
                    type_path: ast::TypePath(vec!["widget".to_owned(), "FlexBox".to_owned()]),
                    fields: vec![ast::Field {
                        name: "direction".to_owned(),
                        value: ast::FieldValue::Literal(ast::Literal::QuotedString(
                            "\"horizontal\"".to_owned()
                        ))
                    }],
                })));

        assert!(parser
            .parse(
                "widget::FlexBox(direction = \"horizontal\") {
                    state SomeState = 3;
                }"
            )
            .is_ok_and(|result| result
                == ast::AliasReplacement::Widget(ast::ScopeDeclaration {
                    kind: ast::ScopeKind::Widget(ast::WidgetDeclaration {
                        type_path: ast::TypePath(vec!["widget".to_owned(), "FlexBox".to_owned()]),
                        fields: vec![ast::Field {
                            name: "direction".to_owned(),
                            value: ast::FieldValue::Literal(ast::Literal::QuotedString(
                                "\"horizontal\"".to_owned()
                            ))
                        }],
                    }),
                    children: vec![ast::InScopeDeclaration::State(ast::StateDeclaration {
                        name: "SomeState".to_owned(),
                        value: ast::Literal::UInt(3)
                    })]
                })));

        assert!(parser.parse("widget::FlexBox(").is_err());
        assert!(parser.parse("widget::FlexBox)").is_err());

        assert!(parser
            .parse("widget::FlexBox(direction = \"horizontal\", other_field) {")
            .is_err());
        assert!(parser
            .parse("widget::FlexBox(direction = \"horizontal\", other_field = 3, {")
            .is_err());
        assert!(parser
            .parse("widget::FlexBox(direction = \"horizontal\")) {")
            .is_err());

        assert!(parser
            .parse(
                "widget::FlexBox(direction = \"horizontal\")) {
                    state SomeState = 3
                }"
            )
            .is_err());
    }

    #[test]
    fn parse_state_declaration() {
        let parser = grammar::StateDeclarationParser::new();

        assert!(parser
            .parse("state SomeState = 1;")
            .is_ok_and(|result| result
                == ast::StateDeclaration {
                    name: "SomeState".to_owned(),
                    value: ast::Literal::UInt(1),
                }));

        assert!(parser
            .parse("state StringState = \"hello\";")
            .is_ok_and(|result| result
                == ast::StateDeclaration {
                    name: "StringState".to_owned(),
                    value: ast::Literal::QuotedString("\"hello\"".to_owned()),
                }));

        assert!(parser.parse("state _ = \"hello\";").is_err());
        assert!(parser
            .parse("state not_correct_declaration = \"hello\"")
            .is_err());
        assert!(parser.parse("state not_correct_declaration = ;").is_err());
        assert!(parser.parse("state 123 = 4;").is_err());
        assert!(parser.parse("123 = 4;").is_err());
        assert!(parser.parse("state = 4;").is_err());
    }

    #[test]
    fn parse_in_scope_declaration() {
        let parser = grammar::InScopeDeclarationParser::new();

        assert!(parser
            .parse("state some_state = 3;")
            .is_ok_and(|result| result
                == ast::InScopeDeclaration::State(ast::StateDeclaration {
                    name: "some_state".to_owned(),
                    value: ast::Literal::UInt(3),
                })));

        assert!(parser
            .parse("alias Flex = widget::FlexBox;")
            .is_ok_and(|result| result
                == ast::InScopeDeclaration::Alias(ast::AliasDeclaration {
                    name: "Flex".to_owned(),
                    replacement: ast::AliasReplacement::Path(ast::TypePath(vec![
                        "widget".to_owned(),
                        "FlexBox".to_owned()
                    ]))
                })));

        assert!(parser.parse("Row() {}").is_ok_and(|result| result
            == ast::InScopeDeclaration::Widget(ast::ScopeDeclaration {
                kind: ast::ScopeKind::Widget(ast::WidgetDeclaration {
                    type_path: ast::TypePath(vec!["Row".to_owned()]),
                    fields: vec![]
                }),
                children: vec![],
            })));

        assert!(parser.parse("Row() {").is_err());
        assert!(parser.parse("Row( {}").is_err());

        // INFO: TypeDeclaration is not in-scope declaration!
        assert!(parser.parse("Row()").is_err());

        assert!(parser.parse("some_ident").is_err());
        assert!(parser.parse("some_ident = 3;").is_err());
    }

    #[test]
    fn parse_widget_declaration() {
        let parser = grammar::WidgetDeclarationParser::new();

        // INFO: it's valid from syntax perspective, even if it's possibly not valid from semantic.
        assert!(parser
            .parse(
                "FlexBox(
                    direction = \"vertical\"
                ) {
                    state inner_state = 3;

                    Text(value = body_state) {}
                }"
            )
            .is_ok_and(|result| result
                == ast::ScopeDeclaration {
                    kind: ast::ScopeKind::Widget(ast::WidgetDeclaration {
                        type_path: ast::TypePath(vec!["FlexBox".to_owned()]),
                        fields: vec![ast::Field {
                            name: "direction".to_owned(),
                            value: ast::FieldValue::Literal(ast::Literal::QuotedString(
                                "\"vertical\"".to_owned(),
                            )),
                        }],
                    }),
                    children: vec![
                        ast::InScopeDeclaration::State(ast::StateDeclaration {
                            name: "inner_state".to_owned(),
                            value: ast::Literal::UInt(3),
                        }),
                        ast::InScopeDeclaration::Widget(ast::ScopeDeclaration {
                            kind: ast::ScopeKind::Widget(ast::WidgetDeclaration {
                                type_path: ast::TypePath(vec!["Text".to_owned()]),
                                fields: vec![ast::Field {
                                    name: "value".to_owned(),
                                    value: ast::FieldValue::State("body_state".to_owned()),
                                }],
                            }),
                            children: vec![],
                        }),
                    ],
                }));

        assert!(parser
            .parse("path::SomeWidget() {}")
            .is_ok_and(|result| result
                == ast::ScopeDeclaration {
                    kind: ast::ScopeKind::Widget(ast::WidgetDeclaration {
                        type_path: ast::TypePath(vec!["path".to_owned(), "SomeWidget".to_owned()]),
                        fields: vec![]
                    }),
                    children: vec![],
                }));

        assert!(parser.parse("path::SomeWidget() {").is_err());
        assert!(parser.parse("path::SomeWidget() }").is_err());
        assert!(parser.parse("path::SomeWidget( {}").is_err());
        assert!(parser.parse("path::SomeWidget) {}").is_err());
        assert!(parser.parse("_() {}").is_err());
        assert!(parser.parse("123() {}").is_err());
        assert!(parser.parse("ВиджетНаКириллице() {}").is_err());

        // INFO: WidgetDeclaration must have a scope. That's the difference from TypeDeclaration.
        assert!(parser.parse("path::SomeWidget()").is_err());
    }

    #[test]
    fn parse_type_declaration() {
        let parser = grammar::TypeDeclarationParser::new();

        assert!(parser.parse("Border()").is_ok_and(|result| result
            == ast::TypeDeclaration {
                type_path: ast::TypePath(vec!["Border".to_owned()]),
                fields: vec![]
            }));

        assert!(parser
            .parse("crate::types::Border()")
            .is_ok_and(|result| result
                == ast::TypeDeclaration {
                    type_path: ast::TypePath(vec![
                        "crate".to_owned(),
                        "types".to_owned(),
                        "Border".to_owned()
                    ]),
                    fields: vec![]
                }));

        assert!(parser.parse("Border(size = 3)").is_ok_and(|result| result
            == ast::TypeDeclaration {
                type_path: ast::TypePath(vec!["Border".to_owned()]),
                fields: vec![ast::Field {
                    name: "size".to_owned(),
                    value: ast::FieldValue::Literal(ast::Literal::UInt(3))
                }]
            }));
        assert!(parser.parse("Border(size = 3,)").is_ok_and(|result| result
            == ast::TypeDeclaration {
                type_path: ast::TypePath(vec!["Border".to_owned()]),
                fields: vec![ast::Field {
                    name: "size".to_owned(),
                    value: ast::FieldValue::Literal(ast::Literal::UInt(3))
                }]
            }));
        assert!(parser
            .parse("Border(size = 3, radius = 4)")
            .is_ok_and(|result| result
                == ast::TypeDeclaration {
                    type_path: ast::TypePath(vec!["Border".to_owned()]),
                    fields: vec![
                        ast::Field {
                            name: "size".to_owned(),
                            value: ast::FieldValue::Literal(ast::Literal::UInt(3))
                        },
                        ast::Field {
                            name: "radius".to_owned(),
                            value: ast::FieldValue::Literal(ast::Literal::UInt(4))
                        },
                    ]
                }));
        assert!(parser
            .parse("Border(size = 3, radius = 4,)")
            .is_ok_and(|result| result
                == ast::TypeDeclaration {
                    type_path: ast::TypePath(vec!["Border".to_owned()]),
                    fields: vec![
                        ast::Field {
                            name: "size".to_owned(),
                            value: ast::FieldValue::Literal(ast::Literal::UInt(3))
                        },
                        ast::Field {
                            name: "radius".to_owned(),
                            value: ast::FieldValue::Literal(ast::Literal::UInt(4))
                        },
                    ]
                }));

        assert!(parser.parse("Border(size = 3, radius = 4,").is_err());
        assert!(parser.parse("Border(size = 3, radius = 4").is_err());
        assert!(parser.parse("Border").is_err());
        assert!(parser.parse("Border(").is_err());
        assert!(parser.parse("Border)").is_err());
        assert!(parser.parse("Border(field)").is_err());
        assert!(parser.parse("Border(field = )").is_err());
        assert!(parser.parse("Border(field = ,)").is_err());
        assert!(parser.parse("border:Border()").is_err());
        assert!(parser.parse("border.Border()").is_err());
        assert!(parser.parse("border-Border()").is_err());
    }

    #[test]
    fn parse_field() {
        let parser = grammar::FieldParser::new();

        assert!(parser
            .parse("field1 = some_state")
            .is_ok_and(|result| result
                == ast::Field {
                    name: "field1".to_owned(),
                    value: ast::FieldValue::State("some_state".to_owned())
                }));
        assert!(parser
            .parse("other_field = \"string literal\"")
            .is_ok_and(|result| result
                == ast::Field {
                    name: "other_field".to_owned(),
                    value: ast::FieldValue::Literal(ast::Literal::QuotedString(
                        "\"string literal\"".to_owned()
                    ))
                }));
        assert!(parser.parse("font_size = 14").is_ok_and(|result| result
            == ast::Field {
                name: "font_size".to_owned(),
                value: ast::FieldValue::Literal(ast::Literal::UInt(14))
            }));

        assert!(parser.parse("font_size =").is_err());
        assert!(parser.parse("font_size").is_err());
        assert!(parser.parse("1font_size = \"\"").is_err());
        assert!(parser.parse("_font_size = \"\"").is_err());
        assert!(parser.parse("_font size = \"\"").is_err());
        assert!(parser.parse("_font size =").is_err());
        assert!(parser.parse("= other_state").is_err());
        assert!(parser
            .parse("valid_name = \"other string literal\",")
            .is_err());
    }

    #[test]
    fn parse_field_value() {
        let parser = grammar::FieldValueParser::new();

        assert!(parser
            .parse("1234")
            .is_ok_and(|result| result == ast::FieldValue::Literal(ast::Literal::UInt(1234))));
        assert!(parser
            .parse("0003")
            .is_ok_and(|result| result == ast::FieldValue::Literal(ast::Literal::UInt(3))));

        assert!(parser.parse("\"test\"").is_ok_and(|result| result
            == ast::FieldValue::Literal(ast::Literal::QuotedString("\"test\"".to_owned()))));
        assert!(parser
            .parse("\"test - _ 1234 TEST\"")
            .is_ok_and(|result| result
                == ast::FieldValue::Literal(ast::Literal::QuotedString(
                    "\"test - _ 1234 TEST\"".to_owned()
                ))));

        assert!(parser
            .parse("test")
            .is_ok_and(|result| result == ast::FieldValue::State("test".to_owned())));
        assert!(parser
            .parse("test123")
            .is_ok_and(|result| result == ast::FieldValue::State("test123".to_owned())));
        assert!(parser
            .parse("test_123")
            .is_ok_and(|result| result == ast::FieldValue::State("test_123".to_owned())));

        assert!(parser
            .parse("Border(field1 = \"val\", field2 = 1234, field3 = some_state)")
            .is_ok_and(|result| result
                == ast::FieldValue::Type(ast::TypeDeclaration {
                    type_path: ast::TypePath(vec!["Border".to_owned()]),
                    fields: vec![
                        ast::Field {
                            name: "field1".to_owned(),
                            value: ast::FieldValue::Literal(ast::Literal::QuotedString(
                                "\"val\"".to_owned()
                            ))
                        },
                        ast::Field {
                            name: "field2".to_owned(),
                            value: ast::FieldValue::Literal(ast::Literal::UInt(1234))
                        },
                        ast::Field {
                            name: "field3".to_owned(),
                            value: ast::FieldValue::State("some_state".to_owned())
                        },
                    ]
                })));
    }

    #[test]
    fn parse_type_path() {
        let parser = grammar::TypePathParser::new();

        assert!(parser
            .parse("crate::module::Name")
            .is_ok_and(|result| result
                == ast::TypePath(vec![
                    "crate".to_owned(),
                    "module".to_owned(),
                    "Name".to_owned()
                ])));
        assert!(parser
            .parse("crate::snake_123_name")
            .is_ok_and(|result| result
                == ast::TypePath(vec!["crate".to_owned(), "snake_123_name".to_owned()])));

        assert!(parser
            .parse("B1g::Black_::Box123")
            .is_ok_and(|result| result
                == ast::TypePath(vec![
                    "B1g".to_owned(),
                    "Black_".to_owned(),
                    "Box123".to_owned()
                ])));
        assert!(parser
            .parse("Name")
            .is_ok_and(|result| result == ast::TypePath(vec!["Name".to_owned()])));

        assert!(parser.parse("crate::snake-123_name").is_err());
        assert!(parser.parse("::").is_err());
        assert!(parser.parse("::name").is_err());
        assert!(parser.parse("123::name").is_err());
        assert!(parser.parse("_::name").is_err());
        assert!(parser.parse("name::_").is_err());
        assert!(parser.parse("name::1").is_err());
        assert!(parser.parse("name::123").is_err());
        assert!(parser.parse("name::123::name").is_err());
        assert!(parser.parse("name::_::name").is_err());
        assert!(parser.parse("name::-::name").is_err());
        assert!(parser.parse("name::name::name::").is_err());
    }

    #[test]
    fn parse_ident() {
        let parser = grammar::IdentParser::new();

        assert!(parser.parse("test").is_ok_and(|result| result == "test"));
        assert!(parser.parse("t").is_ok_and(|result| result == "t"));
        // INFO: Whitespaces are trimmed by lalrpop
        assert!(parser.parse(" t").is_ok_and(|result| result == "t"));
        assert!(parser.parse("Test").is_ok_and(|result| result == "Test"));
        assert!(parser
            .parse("test132")
            .is_ok_and(|result| result == "test132"));
        assert!(parser
            .parse("Test123")
            .is_ok_and(|result| result == "Test123"));
        assert!(parser
            .parse("test_132")
            .is_ok_and(|result| result == "test_132"));
        assert!(parser
            .parse("Test123_")
            .is_ok_and(|result| result == "Test123_"));
        assert!(parser
            .parse("te_123st_132")
            .is_ok_and(|result| result == "te_123st_132"));
        assert!(parser
            .parse("Te_st1_23_")
            .is_ok_and(|result| result == "Te_st1_23_"));

        assert!(parser.parse("1").is_err());
        assert!(parser.parse("10").is_err());
        assert!(parser.parse("01").is_err());
        assert!(parser.parse("01_").is_err());
        assert!(parser.parse("_").is_err());
        assert!(parser.parse("_test").is_err());
        assert!(parser.parse("_Test").is_err());
        assert!(parser.parse("123_test").is_err());
        assert!(parser.parse("321_Test").is_err());
        assert!(parser.parse("").is_err());

        // Non-ASCII symbols are not permitted.
        assert!(parser.parse("тест").is_err());
        assert!(parser.parse("тест123").is_err());
        assert!(parser.parse("testтест123").is_err());
        assert!(parser.parse("test_тест123").is_err());

        // Spaces between symbols are not permitted.
        assert!(parser.parse("te st").is_err());
        assert!(parser.parse("te 123").is_err());
        assert!(parser.parse("te _ 123").is_err());
        assert!(parser.parse("te 23 _").is_err());
    }

    #[test]
    fn parse_literal() {
        let parser = grammar::LiteralParser::new();

        assert!(parser
            .parse("1234")
            .is_ok_and(|result| result == ast::Literal::UInt(1234)));
        assert!(parser
            .parse("01234")
            .is_ok_and(|result| result == ast::Literal::UInt(1234)));
        assert!(parser
            .parse("0000123")
            .is_ok_and(|result| result == ast::Literal::UInt(123)));
        assert!(parser
            .parse("0")
            .is_ok_and(|result| result == ast::Literal::UInt(0)));
        assert!(parser
            .parse("0000000000000000000")
            .is_ok_and(|result| result == ast::Literal::UInt(0)));
        assert!(parser
            .parse("18446744073709551615")
            .is_ok_and(|result| result == ast::Literal::UInt(u64::MAX)));
        assert!(parser.parse("18446744073709551616").is_err());
        assert!(parser.parse("-1").is_err());
        assert!(parser.parse("-0").is_err());

        assert!(parser
            .parse("\"test\"")
            .is_ok_and(|result| result == ast::Literal::QuotedString("\"test\"".to_owned())));
        assert!(parser.parse("\"test - 12_34\"").is_ok_and(
            |result| result == ast::Literal::QuotedString("\"test - 12_34\"".to_owned())
        ));
        assert!(parser
            .parse("\"1234\"")
            .is_ok_and(|result| result == ast::Literal::QuotedString("\"1234\"".to_owned())));
        assert!(parser
            .parse("\"18446744073709551615\"")
            .is_ok_and(|result| result
                == ast::Literal::QuotedString("\"18446744073709551615\"".to_owned())));
        assert!(parser
            .parse("\"18446744073709551616\"")
            .is_ok_and(|result| result
                == ast::Literal::QuotedString("\"18446744073709551616\"".to_owned())));
        assert!(parser
            .parse("\"-18446744073709551616\"")
            .is_ok_and(|result| result
                == ast::Literal::QuotedString("\"-18446744073709551616\"".to_owned())));

        assert!(parser.parse("\"test").is_err());
        assert!(parser.parse("test\"").is_err());
        assert!(parser.parse("test").is_err());
        assert!(parser.parse("123test").is_err());
        assert!(parser.parse("123_test").is_err());
        assert!(parser.parse("123_").is_err());
        assert!(parser.parse("123-").is_err());
        assert!(parser.parse("\"").is_err());
        assert!(parser
            .parse("\"\"")
            .is_ok_and(|result| result == ast::Literal::QuotedString("\"\"".to_owned())));
        assert!(parser.parse("\"\"\"").is_err());

        // Non-ASCII symbols are not permitted.
        assert!(parser.parse("\"тест\"").is_err());
        assert!(parser.parse("\"тест 1234\"").is_err());
        assert!(parser.parse("\"-тест _1234\"").is_err());
        assert!(parser.parse("\"_1234 те-ст\"").is_err());
    }

    #[test]
    fn parse_uint_literal() {
        let parser = grammar::UIntLiteralParser::new();

        assert!(parser.parse("1234").is_ok_and(|result| result == 1234));
        assert!(parser.parse("0123").is_ok_and(|result| result == 123));
        assert!(parser
            .parse("0000000000000000000")
            .is_ok_and(|result| result == 0));
        assert!(parser.parse("0").is_ok_and(|result| result == 0));
        assert!(parser
            .parse("00000000000000000000000000000123")
            .is_ok_and(|result| result == 123));
        assert!(parser.parse("12034").is_ok_and(|result| result == 12034));
        assert!(parser
            .parse("18446744073709551615")
            .is_ok_and(|result| result == u64::MAX));
        assert!(parser
            .parse("18446744073709551614")
            .is_ok_and(|result| result == u64::MAX - 1));
        assert!(parser.parse("18446744073709551616").is_err_and(|err| err
            == lalrpop_util::ParseError::User {
                error: "Integer literal is outside the range of u64."
            }));

        assert!(parser.parse("").is_err());
        assert!(parser.parse("test").is_err());
        assert!(parser.parse("_test").is_err());
        assert!(parser.parse("123_test").is_err());
        assert!(parser.parse("123test").is_err());
        assert!(parser.parse("0test").is_err());
        assert!(parser.parse("0_test").is_err());
        assert!(parser.parse("тест").is_err());
        assert!(parser.parse("123тест").is_err());
        assert!(parser.parse("123_тест").is_err());
        assert!(parser.parse("0_тест").is_err());
    }

    #[test]
    fn parse_string_literal() {
        let parser = grammar::StringLiteralParser::new();

        assert!(parser.parse("\"\"").is_ok_and(|result| result == "\"\""));
        assert!(parser
            .parse("\"test\"")
            .is_ok_and(|result| result == "\"test\""));
        assert!(parser.parse("\" \"").is_ok_and(|result| result == "\" \""));
        assert!(parser
            .parse("\"test 1234\"")
            .is_ok_and(|result| result == "\"test 1234\""));
        assert!(parser
            .parse("\"test 1234 -\"")
            .is_ok_and(|result| result == "\"test 1234 -\""));
        assert!(parser
            .parse("\"test_1234 -\"")
            .is_ok_and(|result| result == "\"test_1234 -\""));
        assert!(parser
            .parse("\"1234 - test_\"")
            .is_ok_and(|result| result == "\"1234 - test_\""));
        assert!(parser
            .parse("\"-_-_\"")
            .is_ok_and(|result| result == "\"-_-_\""));
        assert!(parser
            .parse("\"234\"")
            .is_ok_and(|result| result == "\"234\""));

        assert!(parser.parse("\"test").is_err());
        assert!(parser.parse("test\"").is_err());
        assert!(parser.parse("test").is_err());
        assert!(parser.parse("").is_err());
        assert!(parser.parse("\"").is_err());
        assert!(parser.parse("\"\"test\"").is_err());
        assert!(parser.parse("\"\"\"").is_err());
        assert!(parser.parse("\"\"\"\"").is_err());

        // Non-ASCII symbols are not permitted in string literal.
        assert!(parser.parse("\"тест\"").is_err());
        assert!(parser.parse("\"123 тест\"").is_err());
        assert!(parser.parse("\"_тест\"").is_err());
        assert!(parser.parse("\"-тест 1234\"").is_err());
        assert!(parser.parse("\"test - тест 1234\"").is_err());
    }
}
