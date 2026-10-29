use lalrpop_util::lalrpop_mod;

// TODO: allow more characters in string literal
lalrpop_mod!(pub grammar);

#[cfg(test)]
mod test {
    use super::grammar;
    use crate::ast::*;

    #[test]
    fn parse_input() {
        let parser = grammar::InputParser::new();

        assert!(
            parser
                .parse(&mut vec![], "state counter = 3;")
                .is_ok_and(|result| result
                    == Spanned::new(
                        ScopeDeclaration {
                            kind: ScopeKind::Global,
                            children: vec![Spanned::new(
                                InScopeDeclaration::State(StateDeclaration {
                                    name: Spanned::new("counter".to_owned(), 6, 13),
                                    value: Spanned::new(Literal::UInt(3), 16, 17)
                                },),
                                0,
                                18
                            )]
                        },
                        0,
                        18
                    )),
        );
        assert!(
            parser
                .parse(&mut vec![], "alias Flex = widget::FlexBox;")
                .is_ok_and(|result| result
                    == Spanned::new(
                        ScopeDeclaration {
                            kind: ScopeKind::Global,
                            children: vec![Spanned::new(
                                InScopeDeclaration::Alias(AliasDeclaration {
                                    name: Spanned::new("Flex".to_owned(), 6, 10),
                                    replacement: Spanned::new(
                                        AliasReplacement::Path(TypePath(vec![
                                            Spanned::new("widget".to_owned(), 13, 19),
                                            Spanned::new("FlexBox".to_owned(), 21, 28)
                                        ])),
                                        13,
                                        28
                                    )
                                }),
                                0,
                                29
                            )]
                        },
                        0,
                        29
                    ),)
        );

        assert!(
            parser
                .parse(
                    &mut vec![],
                    "state body_state = \"value\";
                    alias TickBorder = Border(size = 8);

                    FlexBox(
                        direction = \"horizontal\",
                        border = TickBorder(),
                    ) {
                        Text(value = body_state) {}
                    }"
                )
                .is_ok_and(|result| result
                    == Spanned::new(
                        ScopeDeclaration {
                            kind: ScopeKind::Global,
                            children: vec![
                                Spanned::new(
                                    InScopeDeclaration::State(StateDeclaration {
                                        name: Spanned::new("body_state".to_owned(), 6, 16),
                                        value: Spanned::new(
                                            Literal::String("value".to_owned()),
                                            19,
                                            26
                                        ),
                                    }),
                                    0,
                                    27
                                ),
                                Spanned::new(
                                    InScopeDeclaration::Alias(AliasDeclaration {
                                        name: Spanned::new("TickBorder".to_owned(), 54, 64),
                                        replacement: Spanned::new(
                                            AliasReplacement::Type(TypeDeclaration {
                                                type_path: Spanned::new(
                                                    TypePath(vec![Spanned::new(
                                                        "Border".to_owned(),
                                                        67,
                                                        73
                                                    )]),
                                                    67,
                                                    73
                                                ),
                                                fields: vec![Spanned::new(
                                                    Field {
                                                        name: Spanned::new(
                                                            "size".to_owned(),
                                                            74,
                                                            78
                                                        ),
                                                        value: Spanned::new(
                                                            FieldValue::Literal(Literal::UInt(8)),
                                                            81,
                                                            82
                                                        ),
                                                    },
                                                    74,
                                                    82
                                                )]
                                            }),
                                            67,
                                            83
                                        )
                                    }),
                                    48,
                                    84
                                ),
                                Spanned::new(
                                    InScopeDeclaration::Widget(ScopeDeclaration {
                                        kind: ScopeKind::Widget(Spanned::new(
                                            WidgetDeclaration {
                                                type_path: Spanned::new(
                                                    TypePath(vec![Spanned::new(
                                                        "FlexBox".to_owned(),
                                                        106,
                                                        113,
                                                    )]),
                                                    106,
                                                    113
                                                ),
                                                fields: vec![
                                                    Spanned::new(
                                                        Field {
                                                            name: Spanned::new(
                                                                "direction".to_owned(),
                                                                139,
                                                                148
                                                            ),
                                                            value: Spanned::new(
                                                                FieldValue::Literal(
                                                                    Literal::String(
                                                                        "horizontal".to_owned()
                                                                    )
                                                                ),
                                                                151,
                                                                163
                                                            )
                                                        },
                                                        139,
                                                        163
                                                    ),
                                                    Spanned::new(
                                                        Field {
                                                            name: Spanned::new(
                                                                "border".to_owned(),
                                                                189,
                                                                195
                                                            ),
                                                            value: Spanned::new(
                                                                FieldValue::Type(TypeDeclaration {
                                                                    type_path: Spanned::new(
                                                                        TypePath(vec![
                                                                            Spanned::new(
                                                                                "TickBorder"
                                                                                    .to_owned(),
                                                                                198,
                                                                                208
                                                                            )
                                                                        ]),
                                                                        198,
                                                                        208
                                                                    ),
                                                                    fields: vec![]
                                                                }),
                                                                198,
                                                                210
                                                            )
                                                        },
                                                        189,
                                                        210
                                                    )
                                                ]
                                            },
                                            106,
                                            233
                                        )),
                                        children: vec![Spanned::new(
                                            InScopeDeclaration::Widget(ScopeDeclaration {
                                                kind: ScopeKind::Widget(Spanned::new(
                                                    WidgetDeclaration {
                                                        type_path: Spanned::new(
                                                            TypePath(vec![Spanned::new(
                                                                "Text".to_owned(),
                                                                260,
                                                                264
                                                            ),]),
                                                            260,
                                                            264
                                                        ),
                                                        fields: vec![Spanned::new(
                                                            Field {
                                                                name: Spanned::new(
                                                                    "value".to_owned(),
                                                                    265,
                                                                    270
                                                                ),
                                                                value: Spanned::new(
                                                                    FieldValue::State(
                                                                        "body_state".to_owned()
                                                                    ),
                                                                    273,
                                                                    283
                                                                )
                                                            },
                                                            265,
                                                            283
                                                        )]
                                                    },
                                                    260,
                                                    284,
                                                )),
                                                children: vec![]
                                            }),
                                            260,
                                            287
                                        )]
                                    }),
                                    106,
                                    309
                                )
                            ]
                        },
                        0,
                        309
                    ))
        );

        let mut errors = vec![];
        // INFO: Empty input is not acceptable.
        assert!(parser.parse(&mut errors, "").is_err() || !errors.is_empty());

        let mut errors = vec![];
        assert!(
            parser
                .parse(
                    &mut errors,
                    "state body_state = \"value\";
                    alias TickBorder = Border(size = 8)

                    FlexBox(
                        direction = \"horizontal\",
                        border = TickBorder(),
                    ) {
                        Text(value = body_state) {}
                    }"
                )
                .is_err()
                || !errors.is_empty()
        );

        let mut errors = vec![];
        assert!(
            parser
                .parse(
                    &mut errors,
                    "state body_state = \"value\"
                    alias TickBorder = Border(size = 8);

                    FlexBox(
                        direction = \"horizontal\",
                        border = TickBorder(),
                    ) {
                        Text(value = body_state) {}
                    }"
                )
                .is_err()
                || !errors.is_empty()
        );

        // INFO: Widgets are declared with "{}" introducing a scope, even if a widget cannot have children.
        let mut errors = vec![];
        assert!(
            parser
                .parse(
                    &mut errors,
                    "state body_state = \"value\"
                    alias TickBorder = Border(size = 8);

                    FlexBox(
                        direction = \"horizontal\",
                        border = TickBorder(),
                    ) {
                        Text(value = body_state)
                    }"
                )
                .is_err()
                || !errors.is_empty()
        );
    }

    #[test]
    fn parse_alias_declaration() {
        let parser = grammar::AliasDeclarationParser::new();

        assert!(
            parser
                .parse(
                    &mut vec![],
                    "alias Row = widget::FlexBox(direction = \"vertical\") {};"
                )
                .is_ok_and(|result| result
                    == AliasDeclaration {
                        name: Spanned::new("Row".to_owned(), 6, 9),
                        replacement: Spanned::new(
                            AliasReplacement::Widget(ScopeDeclaration {
                                kind: ScopeKind::Widget(Spanned::new(
                                    WidgetDeclaration {
                                        type_path: Spanned::new(
                                            TypePath(vec![
                                                Spanned::new("widget".to_owned(), 12, 18),
                                                Spanned::new("FlexBox".to_owned(), 20, 27)
                                            ]),
                                            12,
                                            27
                                        ),
                                        fields: vec![Spanned::new(
                                            Field {
                                                name: Spanned::new("direction".to_owned(), 28, 37),
                                                value: Spanned::new(
                                                    FieldValue::Literal(Literal::String(
                                                        "vertical".to_owned()
                                                    )),
                                                    40,
                                                    50
                                                )
                                            },
                                            28,
                                            50
                                        )],
                                    },
                                    12,
                                    51
                                )),
                                children: vec![],
                            }),
                            12,
                            54
                        )
                    },)
        );

        assert!(
            parser
                .parse(&mut vec![], "alias TickBorder = Border(size = 8);")
                .is_ok_and(|result| result
                    == AliasDeclaration {
                        name: Spanned::new("TickBorder".to_owned(), 6, 16),
                        replacement: Spanned::new(
                            AliasReplacement::Type(TypeDeclaration {
                                type_path: Spanned::new(
                                    TypePath(vec![Spanned::new("Border".to_owned(), 19, 25)]),
                                    19,
                                    25
                                ),
                                fields: vec![Spanned::new(
                                    Field {
                                        name: Spanned::new("size".to_owned(), 26, 30),
                                        value: Spanned::new(
                                            FieldValue::Literal(Literal::UInt(8)),
                                            33,
                                            34
                                        ),
                                    },
                                    26,
                                    34
                                )]
                            }),
                            19,
                            35
                        )
                    })
        );

        assert!(
            parser
                .parse(&mut vec![], "alias Flex = widget::FlexBox;")
                .is_ok_and(|result| result
                    == AliasDeclaration {
                        name: Spanned::new("Flex".to_owned(), 6, 10),
                        replacement: Spanned::new(
                            AliasReplacement::Path(TypePath(vec![
                                Spanned::new("widget".to_owned(), 13, 19),
                                Spanned::new("FlexBox".to_owned(), 21, 28)
                            ])),
                            13,
                            28
                        )
                    })
        );

        assert!(
            parser
                .parse(
                    &mut vec![],
                    "alias _ = widget::FlexBox(direction = \"vertical\") {};"
                )
                .is_err()
        );
        assert!(parser.parse(&mut vec![], "alias _ = SomeShit;").is_err());
        assert!(
            parser
                .parse(&mut vec![], "alias 123 = Property();")
                .is_err()
        );
        assert!(
            parser
                .parse(&mut vec![], "some_alias = Property();")
                .is_err()
        );
        assert!(
            parser
                .parse(&mut vec![], "some_alias = Property()")
                .is_err()
        );
        assert!(
            parser
                .parse(
                    &mut vec![],
                    "alias Row = widget::FlexBox(direction = \"vertical\") {}"
                )
                .is_err()
        );
    }

    #[test]
    fn parse_alias_replacement() {
        let parser = grammar::AliasReplacementParser::new();

        assert!(
            parser
                .parse(&mut vec![], "widget::FlexBox")
                .is_ok_and(|result| result
                    == AliasReplacement::Path(TypePath(vec![
                        Spanned::new("widget".to_owned(), 0, 6),
                        Spanned::new("FlexBox".to_owned(), 8, 15)
                    ])))
        );

        assert!(
            parser
                .parse(&mut vec![], "widget::FlexBox()")
                .is_ok_and(|result| result
                    == AliasReplacement::Type(TypeDeclaration {
                        type_path: Spanned::new(
                            TypePath(vec![
                                Spanned::new("widget".to_owned(), 0, 6),
                                Spanned::new("FlexBox".to_owned(), 8, 15)
                            ]),
                            0,
                            15
                        ),
                        fields: vec![],
                    }))
        );

        assert!(
            parser
                .parse(&mut vec![], "widget::FlexBox(direction = \"horizontal\")")
                .is_ok_and(|result| result
                    == AliasReplacement::Type(TypeDeclaration {
                        type_path: Spanned::new(
                            TypePath(vec![
                                Spanned::new("widget".to_owned(), 0, 6),
                                Spanned::new("FlexBox".to_owned(), 8, 15)
                            ]),
                            0,
                            15
                        ),
                        fields: vec![Spanned::new(
                            Field {
                                name: Spanned::new("direction".to_owned(), 16, 25),
                                value: Spanned::new(
                                    FieldValue::Literal(Literal::String("horizontal".to_owned())),
                                    28,
                                    40
                                )
                            },
                            16,
                            40
                        )],
                    }))
        );

        assert!(
            parser
                .parse(
                    &mut vec![],
                    "widget::FlexBox(direction = \"horizontal\") {
                    state SomeState = 3;
                }"
                )
                .is_ok_and(|result| result
                    == AliasReplacement::Widget(ScopeDeclaration {
                        kind: ScopeKind::Widget(Spanned::new(
                            WidgetDeclaration {
                                type_path: Spanned::new(
                                    TypePath(vec![
                                        Spanned::new("widget".to_owned(), 0, 6),
                                        Spanned::new("FlexBox".to_owned(), 8, 15)
                                    ]),
                                    0,
                                    15
                                ),
                                fields: vec![Spanned::new(
                                    Field {
                                        name: Spanned::new("direction".to_owned(), 16, 25),
                                        value: Spanned::new(
                                            FieldValue::Literal(Literal::String(
                                                "horizontal".to_owned()
                                            )),
                                            28,
                                            40
                                        )
                                    },
                                    16,
                                    40
                                )],
                            },
                            0,
                            41
                        ),),
                        children: vec![Spanned::new(
                            InScopeDeclaration::State(StateDeclaration {
                                name: Spanned::new("SomeState".to_owned(), 70, 79),
                                value: Spanned::new(Literal::UInt(3), 82, 83),
                            }),
                            64,
                            84
                        )]
                    }))
        );

        assert!(parser.parse(&mut vec![], "widget::FlexBox(").is_err());
        assert!(parser.parse(&mut vec![], "widget::FlexBox)").is_err());

        assert!(
            parser
                .parse(
                    &mut vec![],
                    "widget::FlexBox(direction = \"horizontal\", other_field) {"
                )
                .is_err()
        );
        assert!(
            parser
                .parse(
                    &mut vec![],
                    "widget::FlexBox(direction = \"horizontal\", other_field = 3, {"
                )
                .is_err()
        );
        assert!(
            parser
                .parse(
                    &mut vec![],
                    "widget::FlexBox(direction = \"horizontal\")) {"
                )
                .is_err()
        );

        assert!(
            parser
                .parse(
                    &mut vec![],
                    "widget::FlexBox(direction = \"horizontal\")) {
                    state SomeState = 3
                }"
                )
                .is_err()
        );
    }

    #[test]
    fn parse_state_declaration() {
        let parser = grammar::StateDeclarationParser::new();

        assert!(
            parser
                .parse(&mut vec![], "state SomeState = 1;")
                .is_ok_and(|result| result
                    == StateDeclaration {
                        name: Spanned::new("SomeState".to_owned(), 6, 15),
                        value: Spanned::new(Literal::UInt(1), 18, 19),
                    })
        );

        assert!(
            parser
                .parse(&mut vec![], "state StringState = \"hello\";")
                .is_ok_and(|result| result
                    == StateDeclaration {
                        name: Spanned::new("StringState".to_owned(), 6, 17),
                        value: Spanned::new(Literal::String("hello".to_owned()), 20, 27),
                    })
        );

        assert!(parser.parse(&mut vec![], "state _ = \"hello\";").is_err());
        assert!(
            parser
                .parse(&mut vec![], "state not_correct_declaration = \"hello\"")
                .is_err()
        );
        assert!(
            parser
                .parse(&mut vec![], "state not_correct_declaration = ;")
                .is_err()
        );
        assert!(parser.parse(&mut vec![], "state 123 = 4;").is_err());
        assert!(parser.parse(&mut vec![], "123 = 4;").is_err());
        assert!(parser.parse(&mut vec![], "state = 4;").is_err());
    }

    #[test]
    fn parse_in_scope_declaration() {
        let parser = grammar::InScopeDeclarationParser::new();

        assert!(
            parser
                .parse(&mut vec![], "state some_state = 3;")
                .is_ok_and(|result| result
                    == Some(InScopeDeclaration::State(StateDeclaration {
                        name: Spanned::new("some_state".to_owned(), 6, 16),
                        value: Spanned::new(Literal::UInt(3), 19, 20),
                    })))
        );

        assert!(
            parser
                .parse(&mut vec![], "alias Flex = widget::FlexBox;")
                .is_ok_and(|result| result
                    == Some(InScopeDeclaration::Alias(AliasDeclaration {
                        name: Spanned::new("Flex".to_owned(), 6, 10),
                        replacement: Spanned::new(
                            AliasReplacement::Path(TypePath(vec![
                                Spanned::new("widget".to_owned(), 13, 19),
                                Spanned::new("FlexBox".to_owned(), 21, 28),
                            ])),
                            13,
                            28
                        )
                    })))
        );

        assert!(
            parser
                .parse(&mut vec![], "Row() {}")
                .is_ok_and(|result| result
                    == Some(InScopeDeclaration::Widget(ScopeDeclaration {
                        kind: ScopeKind::Widget(Spanned::new(
                            WidgetDeclaration {
                                type_path: Spanned::new(
                                    TypePath(vec![Spanned::new("Row".to_owned(), 0, 3)]),
                                    0,
                                    3
                                ),
                                fields: vec![]
                            },
                            0,
                            5
                        )),
                        children: vec![],
                    })))
        );

        let mut errors = vec![];
        assert!(parser.parse(&mut errors, "Row() {").is_err() || !errors.is_empty());

        let mut errors = vec![];
        assert!(parser.parse(&mut errors, "Row( {}").is_err() || !errors.is_empty());

        // INFO: TypeDeclaration is not in-scope declaration!
        let mut errors = vec![];
        assert!(parser.parse(&mut errors, "Row()").is_err() || !errors.is_empty());

        let mut errors = vec![];
        assert!(parser.parse(&mut errors, "some_ident").is_err() || !errors.is_empty());

        let mut errors = vec![];
        assert!(parser.parse(&mut errors, "some_ident = 3;").is_err() || !errors.is_empty());
    }

    #[test]
    fn parse_widget_declaration() {
        let parser = grammar::WidgetDeclarationParser::new();

        // INFO: it's valid from syntax perspective, even if it's possibly not valid from semantic.
        assert!(
            parser
                .parse(
                    &mut vec![],
                    "FlexBox(
                        direction = \"vertical\"
                    ) {
                        state inner_state = 3;

                        Text(value = body_state) {}
                    }"
                )
                .is_ok_and(|result| result
                    == ScopeDeclaration {
                        kind: ScopeKind::Widget(Spanned::new(
                            WidgetDeclaration {
                                type_path: Spanned::new(
                                    TypePath(vec![Spanned::new("FlexBox".to_owned(), 0, 7)]),
                                    0,
                                    7
                                ),
                                fields: vec![Spanned::new(
                                    Field {
                                        name: Spanned::new("direction".to_owned(), 33, 42),
                                        value: Spanned::new(
                                            FieldValue::Literal(Literal::String(
                                                "vertical".to_owned(),
                                            )),
                                            45,
                                            55
                                        ),
                                    },
                                    33,
                                    55
                                )],
                            },
                            0,
                            77
                        )),
                        children: vec![
                            Spanned::new(
                                InScopeDeclaration::State(StateDeclaration {
                                    name: Spanned::new("inner_state".to_owned(), 110, 121),
                                    value: Spanned::new(Literal::UInt(3), 124, 125)
                                }),
                                104,
                                126
                            ),
                            Spanned::new(
                                InScopeDeclaration::Widget(ScopeDeclaration {
                                    kind: ScopeKind::Widget(Spanned::new(
                                        WidgetDeclaration {
                                            type_path: Spanned::new(
                                                TypePath(vec![Spanned::new(
                                                    "Text".to_owned(),
                                                    152,
                                                    156
                                                )]),
                                                152,
                                                156
                                            ),
                                            fields: vec![Spanned::new(
                                                Field {
                                                    name: Spanned::new(
                                                        "value".to_owned(),
                                                        157,
                                                        162
                                                    ),
                                                    value: Spanned::new(
                                                        FieldValue::State("body_state".to_owned()),
                                                        165,
                                                        175
                                                    ),
                                                },
                                                157,
                                                175
                                            )],
                                        },
                                        152,
                                        176
                                    )),
                                    children: vec![],
                                }),
                                152,
                                179
                            ),
                        ],
                    })
        );

        assert!(
            parser
                .parse(&mut vec![], "path::SomeWidget() {}")
                .is_ok_and(|result| result
                    == ScopeDeclaration {
                        kind: ScopeKind::Widget(Spanned::new(
                            WidgetDeclaration {
                                type_path: Spanned::new(
                                    TypePath(vec![
                                        Spanned::new("path".to_owned(), 0, 4),
                                        Spanned::new("SomeWidget".to_owned(), 6, 16)
                                    ]),
                                    0,
                                    16
                                ),
                                fields: vec![]
                            },
                            0,
                            18
                        )),
                        children: vec![],
                    })
        );

        assert!(parser.parse(&mut vec![], "path::SomeWidget() {").is_err());
        assert!(parser.parse(&mut vec![], "path::SomeWidget() }").is_err());
        assert!(parser.parse(&mut vec![], "path::SomeWidget( {}").is_err());
        assert!(parser.parse(&mut vec![], "path::SomeWidget) {}").is_err());
        assert!(parser.parse(&mut vec![], "_() {}").is_err());
        assert!(parser.parse(&mut vec![], "123() {}").is_err());
        assert!(parser.parse(&mut vec![], "ВиджетНаКириллице() {}").is_err());

        // INFO: WidgetDeclaration must have a scope. That's the difference from TypeDeclaration.
        assert!(parser.parse(&mut vec![], "path::SomeWidget()").is_err());
    }

    #[test]
    fn parse_type_declaration() {
        let parser = grammar::TypeDeclarationParser::new();

        assert!(
            parser
                .parse(&mut vec![], "Border()")
                .is_ok_and(|result| result
                    == TypeDeclaration {
                        type_path: Spanned::new(
                            TypePath(vec![Spanned::new("Border".to_owned(), 0, 6)]),
                            0,
                            6
                        ),
                        fields: vec![]
                    })
        );

        assert!(
            parser
                .parse(&mut vec![], "crate::types::Border()")
                .is_ok_and(|result| result
                    == TypeDeclaration {
                        type_path: Spanned::new(
                            TypePath(vec![
                                Spanned::new("crate".to_owned(), 0, 5),
                                Spanned::new("types".to_owned(), 7, 12),
                                Spanned::new("Border".to_owned(), 14, 20)
                            ]),
                            0,
                            20
                        ),
                        fields: vec![]
                    })
        );

        assert!(
            parser
                .parse(&mut vec![], "Border(size = 3)")
                .is_ok_and(|result| result
                    == TypeDeclaration {
                        type_path: Spanned::new(
                            TypePath(vec![Spanned::new("Border".to_owned(), 0, 6)]),
                            0,
                            6
                        ),
                        fields: vec![Spanned::new(
                            Field {
                                name: Spanned::new("size".to_owned(), 7, 11),
                                value: Spanned::new(FieldValue::Literal(Literal::UInt(3)), 14, 15),
                            },
                            7,
                            15
                        )]
                    })
        );

        assert!(
            parser
                .parse(&mut vec![], "Border(size = 3,)")
                .is_ok_and(|result| result
                    == TypeDeclaration {
                        type_path: Spanned::new(
                            TypePath(vec![Spanned::new("Border".to_owned(), 0, 6)]),
                            0,
                            6
                        ),
                        fields: vec![Spanned::new(
                            Field {
                                name: Spanned::new("size".to_owned(), 7, 11),
                                value: Spanned::new(FieldValue::Literal(Literal::UInt(3)), 14, 15),
                            },
                            7,
                            15
                        )]
                    })
        );

        assert!(
            parser
                .parse(&mut vec![], "Border(size = 3, radius = 4)")
                .is_ok_and(|result| result
                    == TypeDeclaration {
                        type_path: Spanned::new(
                            TypePath(vec![Spanned::new("Border".to_owned(), 0, 6)]),
                            0,
                            6
                        ),
                        fields: vec![
                            Spanned::new(
                                Field {
                                    name: Spanned::new("size".to_owned(), 7, 11),
                                    value: Spanned::new(
                                        FieldValue::Literal(Literal::UInt(3)),
                                        14,
                                        15
                                    ),
                                },
                                7,
                                15
                            ),
                            Spanned::new(
                                Field {
                                    name: Spanned::new("radius".to_owned(), 17, 23),
                                    value: Spanned::new(
                                        FieldValue::Literal(Literal::UInt(4)),
                                        26,
                                        27
                                    )
                                },
                                17,
                                27
                            )
                        ]
                    })
        );

        assert!(
            parser
                .parse(&mut vec![], "Border(size = 3, radius = 4,)")
                .is_ok_and(|result| result
                    == TypeDeclaration {
                        type_path: Spanned::new(
                            TypePath(vec![Spanned::new("Border".to_owned(), 0, 6)]),
                            0,
                            6
                        ),
                        fields: vec![
                            Spanned::new(
                                Field {
                                    name: Spanned::new("size".to_owned(), 7, 11),
                                    value: Spanned::new(
                                        FieldValue::Literal(Literal::UInt(3)),
                                        14,
                                        15
                                    ),
                                },
                                7,
                                15
                            ),
                            Spanned::new(
                                Field {
                                    name: Spanned::new("radius".to_owned(), 17, 23),
                                    value: Spanned::new(
                                        FieldValue::Literal(Literal::UInt(4)),
                                        26,
                                        27
                                    )
                                },
                                17,
                                27
                            )
                        ]
                    })
        );

        assert!(
            parser
                .parse(&mut vec![], "Border(size = 3, radius = 4,")
                .is_err()
        );
        assert!(
            parser
                .parse(&mut vec![], "Border(size = 3, radius = 4")
                .is_err()
        );
        assert!(parser.parse(&mut vec![], "Border").is_err());
        assert!(parser.parse(&mut vec![], "Border(").is_err());
        assert!(parser.parse(&mut vec![], "Border)").is_err());
        assert!(parser.parse(&mut vec![], "Border(field)").is_err());
        assert!(parser.parse(&mut vec![], "Border(field = )").is_err());
        assert!(parser.parse(&mut vec![], "Border(field = ,)").is_err());
        assert!(parser.parse(&mut vec![], "border:Border()").is_err());
        assert!(parser.parse(&mut vec![], "border.Border()").is_err());
        assert!(parser.parse(&mut vec![], "border-Border()").is_err());
    }

    #[test]
    fn parse_field() {
        let parser = grammar::FieldParser::new();

        assert!(
            parser
                .parse(&mut vec![], "field1 = some_state")
                .is_ok_and(|result| result
                    == Field {
                        name: Spanned::new("field1".to_owned(), 0, 6),
                        value: Spanned::new(FieldValue::State("some_state".to_owned()), 9, 19)
                    })
        );
        assert!(
            parser
                .parse(&mut vec![], "other_field = \"string literal\"")
                .is_ok_and(|result| result
                    == Field {
                        name: Spanned::new("other_field".to_owned(), 0, 11),
                        value: Spanned::new(
                            FieldValue::Literal(Literal::String("string literal".to_owned())),
                            14,
                            30
                        )
                    })
        );
        assert!(
            parser
                .parse(&mut vec![], "font_size = 14")
                .is_ok_and(|result| result
                    == Field {
                        name: Spanned::new("font_size".to_owned(), 0, 9),
                        value: Spanned::new(FieldValue::Literal(Literal::UInt(14)), 12, 14),
                    })
        );

        assert!(parser.parse(&mut vec![], "font_size =").is_err());
        assert!(parser.parse(&mut vec![], "font_size").is_err());
        assert!(parser.parse(&mut vec![], "1font_size = \"\"").is_err());
        assert!(parser.parse(&mut vec![], "_font_size = \"\"").is_err());
        assert!(parser.parse(&mut vec![], "_font size = \"\"").is_err());
        assert!(parser.parse(&mut vec![], "_font size =").is_err());
        assert!(parser.parse(&mut vec![], "= other_state").is_err());
        assert!(
            parser
                .parse(&mut vec![], "valid_name = \"other string literal\",")
                .is_err()
        );
    }

    #[test]
    fn parse_field_value() {
        let parser = grammar::FieldValueParser::new();

        assert!(
            parser
                .parse(&mut vec![], "1234")
                .is_ok_and(|result| result == FieldValue::Literal(Literal::UInt(1234)))
        );
        assert!(
            parser
                .parse(&mut vec![], "0003")
                .is_ok_and(|result| result == FieldValue::Literal(Literal::UInt(3)))
        );

        assert!(
            parser.parse(&mut vec![], "\"test\"").is_ok_and(
                |result| result == FieldValue::Literal(Literal::String("test".to_owned()))
            )
        );
        assert!(
            parser
                .parse(&mut vec![], "\"test - _ 1234 TEST\"")
                .is_ok_and(|result| result
                    == FieldValue::Literal(Literal::String("test - _ 1234 TEST".to_owned())))
        );

        assert!(
            parser
                .parse(&mut vec![], "test")
                .is_ok_and(|result| result == FieldValue::State("test".to_owned()))
        );
        assert!(
            parser
                .parse(&mut vec![], "test123")
                .is_ok_and(|result| result == FieldValue::State("test123".to_owned()))
        );
        assert!(
            parser
                .parse(&mut vec![], "test_123")
                .is_ok_and(|result| result == FieldValue::State("test_123".to_owned()))
        );

        assert!(
            parser
                .parse(
                    &mut vec![],
                    "Border(field1 = \"val\", field2 = 1234, field3 = some_state)"
                )
                .is_ok_and(|result| result
                    == FieldValue::Type(TypeDeclaration {
                        type_path: Spanned::new(
                            TypePath(vec![Spanned::new("Border".to_owned(), 0, 6)]),
                            0,
                            6
                        ),
                        fields: vec![
                            Spanned::new(
                                Field {
                                    name: Spanned::new("field1".to_owned(), 7, 13),
                                    value: Spanned::new(
                                        FieldValue::Literal(Literal::String("val".to_owned())),
                                        16,
                                        21
                                    )
                                },
                                7,
                                21
                            ),
                            Spanned::new(
                                Field {
                                    name: Spanned::new("field2".to_owned(), 23, 29),
                                    value: Spanned::new(
                                        FieldValue::Literal(Literal::UInt(1234)),
                                        32,
                                        36
                                    )
                                },
                                23,
                                36
                            ),
                            Spanned::new(
                                Field {
                                    name: Spanned::new("field3".to_owned(), 38, 44),
                                    value: Spanned::new(
                                        FieldValue::State("some_state".to_owned()),
                                        47,
                                        57
                                    )
                                },
                                38,
                                57
                            ),
                        ]
                    }))
        );
    }

    #[test]
    fn parse_type_path() {
        let parser = grammar::TypePathParser::new();

        assert!(
            parser
                .parse(&mut vec![], "crate::module::Name")
                .is_ok_and(|result| result
                    == TypePath(vec![
                        Spanned::new("crate".to_owned(), 0, 5),
                        Spanned::new("module".to_owned(), 7, 13),
                        Spanned::new("Name".to_owned(), 15, 19)
                    ]))
        );
        assert!(
            parser
                .parse(&mut vec![], "crate::snake_123_name")
                .is_ok_and(|result| result
                    == TypePath(vec![
                        Spanned::new("crate".to_owned(), 0, 5),
                        Spanned::new("snake_123_name".to_owned(), 7, 21)
                    ]))
        );

        assert!(
            parser
                .parse(&mut vec![], "B1g::Black_::Box123")
                .is_ok_and(|result| result
                    == TypePath(vec![
                        Spanned::new("B1g".to_owned(), 0, 3),
                        Spanned::new("Black_".to_owned(), 5, 11),
                        Spanned::new("Box123".to_owned(), 13, 19),
                    ]))
        );
        assert!(
            parser
                .parse(&mut vec![], "Name")
                .is_ok_and(|result| result == TypePath(vec![Spanned::new("Name".to_owned(), 0, 4)]))
        );

        assert!(parser.parse(&mut vec![], "crate::snake-123_name").is_err());
        assert!(parser.parse(&mut vec![], "::").is_err());
        assert!(parser.parse(&mut vec![], "::name").is_err());
        assert!(parser.parse(&mut vec![], "123::name").is_err());
        assert!(parser.parse(&mut vec![], "_::name").is_err());
        assert!(parser.parse(&mut vec![], "name::_").is_err());
        assert!(parser.parse(&mut vec![], "name::1").is_err());
        assert!(parser.parse(&mut vec![], "name::123").is_err());
        assert!(parser.parse(&mut vec![], "name::123::name").is_err());
        assert!(parser.parse(&mut vec![], "name::_::name").is_err());
        assert!(parser.parse(&mut vec![], "name::-::name").is_err());
        assert!(parser.parse(&mut vec![], "name::name::name::").is_err());
    }

    #[test]
    fn parse_ident() {
        let parser = grammar::IdentParser::new();

        assert!(
            parser
                .parse(&mut vec![], "test")
                .is_ok_and(|result| result == "test")
        );
        assert!(
            parser
                .parse(&mut vec![], "t")
                .is_ok_and(|result| result == "t")
        );
        // INFO: Whitespaces are trimmed by lalrpop
        assert!(
            parser
                .parse(&mut vec![], " t")
                .is_ok_and(|result| result == "t")
        );
        assert!(
            parser
                .parse(&mut vec![], "Test")
                .is_ok_and(|result| result == "Test")
        );
        assert!(
            parser
                .parse(&mut vec![], "test132")
                .is_ok_and(|result| result == "test132")
        );
        assert!(
            parser
                .parse(&mut vec![], "Test123")
                .is_ok_and(|result| result == "Test123")
        );
        assert!(
            parser
                .parse(&mut vec![], "test_132")
                .is_ok_and(|result| result == "test_132")
        );
        assert!(
            parser
                .parse(&mut vec![], "Test123_")
                .is_ok_and(|result| result == "Test123_")
        );
        assert!(
            parser
                .parse(&mut vec![], "te_123st_132")
                .is_ok_and(|result| result == "te_123st_132")
        );
        assert!(
            parser
                .parse(&mut vec![], "Te_st1_23_")
                .is_ok_and(|result| result == "Te_st1_23_")
        );

        assert!(parser.parse(&mut vec![], "1").is_err());
        assert!(parser.parse(&mut vec![], "10").is_err());
        assert!(parser.parse(&mut vec![], "01").is_err());
        assert!(parser.parse(&mut vec![], "01_").is_err());
        assert!(parser.parse(&mut vec![], "_").is_err());
        assert!(parser.parse(&mut vec![], "_test").is_err());
        assert!(parser.parse(&mut vec![], "_Test").is_err());
        assert!(parser.parse(&mut vec![], "123_test").is_err());
        assert!(parser.parse(&mut vec![], "321_Test").is_err());
        assert!(parser.parse(&mut vec![], "").is_err());

        // Non-ASCII symbols are not permitted.
        assert!(parser.parse(&mut vec![], "тест").is_err());
        assert!(parser.parse(&mut vec![], "тест123").is_err());
        assert!(parser.parse(&mut vec![], "testтест123").is_err());
        assert!(parser.parse(&mut vec![], "test_тест123").is_err());

        // Spaces between symbols are not permitted.
        assert!(parser.parse(&mut vec![], "te st").is_err());
        assert!(parser.parse(&mut vec![], "te 123").is_err());
        assert!(parser.parse(&mut vec![], "te _ 123").is_err());
        assert!(parser.parse(&mut vec![], "te 23 _").is_err());
    }

    #[test]
    fn parse_literal() {
        let parser = grammar::LiteralParser::new();

        assert!(
            parser
                .parse(&mut vec![], "1234")
                .is_ok_and(|result| result == Literal::UInt(1234))
        );
        assert!(
            parser
                .parse(&mut vec![], "01234")
                .is_ok_and(|result| result == Literal::UInt(1234))
        );
        assert!(
            parser
                .parse(&mut vec![], "0000123")
                .is_ok_and(|result| result == Literal::UInt(123))
        );
        assert!(
            parser
                .parse(&mut vec![], "0")
                .is_ok_and(|result| result == Literal::UInt(0))
        );
        assert!(
            parser
                .parse(&mut vec![], "0000000000000000000")
                .is_ok_and(|result| result == Literal::UInt(0))
        );
        assert!(
            parser
                .parse(&mut vec![], "18446744073709551615")
                .is_ok_and(|result| result == Literal::UInt(u64::MAX))
        );
        assert!(parser.parse(&mut vec![], "18446744073709551616").is_err());
        assert!(parser.parse(&mut vec![], "-1").is_err());
        assert!(parser.parse(&mut vec![], "-0").is_err());

        assert!(
            parser
                .parse(&mut vec![], "\"test\"")
                .is_ok_and(|result| result == Literal::String("test".to_owned()))
        );
        assert!(
            parser
                .parse(&mut vec![], "\"test - 12_34\"")
                .is_ok_and(|result| result == Literal::String("test - 12_34".to_owned()))
        );
        assert!(
            parser
                .parse(&mut vec![], "\"1234\"")
                .is_ok_and(|result| result == Literal::String("1234".to_owned()))
        );
        assert!(
            parser
                .parse(&mut vec![], "\"18446744073709551615\"")
                .is_ok_and(|result| result == Literal::String("18446744073709551615".to_owned()))
        );
        assert!(
            parser
                .parse(&mut vec![], "\"18446744073709551616\"")
                .is_ok_and(|result| result == Literal::String("18446744073709551616".to_owned()))
        );
        assert!(
            parser
                .parse(&mut vec![], "\"-18446744073709551616\"")
                .is_ok_and(|result| result == Literal::String("-18446744073709551616".to_owned()))
        );

        assert!(parser.parse(&mut vec![], "\"test").is_err());
        assert!(parser.parse(&mut vec![], "test\"").is_err());
        assert!(parser.parse(&mut vec![], "test").is_err());
        assert!(parser.parse(&mut vec![], "123test").is_err());
        assert!(parser.parse(&mut vec![], "123_test").is_err());
        assert!(parser.parse(&mut vec![], "123_").is_err());
        assert!(parser.parse(&mut vec![], "123-").is_err());
        assert!(parser.parse(&mut vec![], "\"").is_err());
        assert!(
            parser
                .parse(&mut vec![], "\"\"")
                .is_ok_and(|result| result == Literal::String("".to_owned()))
        );
        assert!(parser.parse(&mut vec![], "\"\"\"").is_err());

        // Non-ASCII symbols are not permitted.
        assert!(parser.parse(&mut vec![], "\"тест\"").is_err());
        assert!(parser.parse(&mut vec![], "\"тест 1234\"").is_err());
        assert!(parser.parse(&mut vec![], "\"-тест _1234\"").is_err());
        assert!(parser.parse(&mut vec![], "\"_1234 те-ст\"").is_err());

        assert!(
            parser
                .parse(&mut vec![], "true")
                .is_ok_and(|result| result == Literal::Bool(true))
        );
        assert!(
            parser
                .parse(&mut vec![], "false")
                .is_ok_and(|result| result == Literal::Bool(false))
        );
        assert!(parser.parse(&mut vec![], "yes").is_err());
        assert!(parser.parse(&mut vec![], "no").is_err());
        assert!(parser.parse(&mut vec![], "on").is_err());
        assert!(parser.parse(&mut vec![], "off").is_err());
    }

    #[test]
    fn parse_uint_literal() {
        let parser = grammar::UIntLiteralParser::new();

        assert!(
            parser
                .parse(&mut vec![], "1234")
                .is_ok_and(|result| result == 1234)
        );
        assert!(
            parser
                .parse(&mut vec![], "0123")
                .is_ok_and(|result| result == 123)
        );
        assert!(
            parser
                .parse(&mut vec![], "0000000000000000000")
                .is_ok_and(|result| result == 0)
        );
        assert!(
            parser
                .parse(&mut vec![], "0")
                .is_ok_and(|result| result == 0)
        );
        assert!(
            parser
                .parse(&mut vec![], "00000000000000000000000000000123")
                .is_ok_and(|result| result == 123)
        );
        assert!(
            parser
                .parse(&mut vec![], "12034")
                .is_ok_and(|result| result == 12034)
        );
        assert!(
            parser
                .parse(&mut vec![], "18446744073709551615")
                .is_ok_and(|result| result == u64::MAX)
        );
        assert!(
            parser
                .parse(&mut vec![], "18446744073709551614")
                .is_ok_and(|result| result == u64::MAX - 1)
        );
        assert!(
            parser
                .parse(&mut vec![], "18446744073709551616")
                .is_err_and(|err| err
                    == lalrpop_util::ParseError::User {
                        error: "Integer literal is outside the range of u64."
                    })
        );

        assert!(parser.parse(&mut vec![], "").is_err());
        assert!(parser.parse(&mut vec![], "test").is_err());
        assert!(parser.parse(&mut vec![], "_test").is_err());
        assert!(parser.parse(&mut vec![], "123_test").is_err());
        assert!(parser.parse(&mut vec![], "123test").is_err());
        assert!(parser.parse(&mut vec![], "0test").is_err());
        assert!(parser.parse(&mut vec![], "0_test").is_err());
        assert!(parser.parse(&mut vec![], "тест").is_err());
        assert!(parser.parse(&mut vec![], "123тест").is_err());
        assert!(parser.parse(&mut vec![], "123_тест").is_err());
        assert!(parser.parse(&mut vec![], "0_тест").is_err());
    }

    #[test]
    fn parse_string_literal() {
        let parser = grammar::StringLiteralParser::new();

        assert!(
            parser
                .parse(&mut vec![], "\"\"")
                .is_ok_and(|result| result.is_empty())
        );
        assert!(
            parser
                .parse(&mut vec![], "\"test\"")
                .is_ok_and(|result| result == "test")
        );
        assert!(
            parser
                .parse(&mut vec![], "\" \"")
                .is_ok_and(|result| result == " ")
        );
        assert!(
            parser
                .parse(&mut vec![], "\"test 1234\"")
                .is_ok_and(|result| result == "test 1234")
        );
        assert!(
            parser
                .parse(&mut vec![], "\"test 1234 -\"")
                .is_ok_and(|result| result == "test 1234 -")
        );
        assert!(
            parser
                .parse(&mut vec![], "\"test_1234 -\"")
                .is_ok_and(|result| result == "test_1234 -")
        );
        assert!(
            parser
                .parse(&mut vec![], "\"1234 - test_\"")
                .is_ok_and(|result| result == "1234 - test_")
        );
        assert!(
            parser
                .parse(&mut vec![], "\"-_-_\"")
                .is_ok_and(|result| result == "-_-_")
        );
        assert!(
            parser
                .parse(&mut vec![], "\"234\"")
                .is_ok_and(|result| result == "234")
        );

        assert!(parser.parse(&mut vec![], "\"test").is_err());
        assert!(parser.parse(&mut vec![], "test\"").is_err());
        assert!(parser.parse(&mut vec![], "test").is_err());
        assert!(parser.parse(&mut vec![], "").is_err());
        assert!(parser.parse(&mut vec![], "\"").is_err());
        assert!(parser.parse(&mut vec![], "\"\"test\"").is_err());
        assert!(parser.parse(&mut vec![], "\"\"\"").is_err());
        assert!(parser.parse(&mut vec![], "\"\"\"\"").is_err());

        // Non-ASCII symbols are not permitted in string literal.
        assert!(parser.parse(&mut vec![], "\"тест\"").is_err());
        assert!(parser.parse(&mut vec![], "\"123 тест\"").is_err());
        assert!(parser.parse(&mut vec![], "\"_тест\"").is_err());
        assert!(parser.parse(&mut vec![], "\"-тест 1234\"").is_err());
        assert!(parser.parse(&mut vec![], "\"test - тест 1234\"").is_err());
    }

    #[test]
    fn parse_bool_literal() {
        let parser = grammar::BoolLiteralParser::new();

        assert!(parser.parse(&mut vec![], "true").is_ok_and(|result| result));
        assert!(
            parser
                .parse(&mut vec![], "false")
                .is_ok_and(|result| !result)
        );

        assert!(parser.parse(&mut vec![], "on").is_err());
        assert!(parser.parse(&mut vec![], "off").is_err());

        assert!(parser.parse(&mut vec![], "yes").is_err());
        assert!(parser.parse(&mut vec![], "no").is_err());
        assert!(parser.parse(&mut vec![], "1").is_err());
        assert!(parser.parse(&mut vec![], "0").is_err());
    }

    #[test]
    fn parse_color_literal() {
        use widgets::types::Bgra;

        let parser = grammar::ColorLiteralParser::new();

        assert!(parser.parse(&mut vec![], "#FFF").is_ok_and(|result| result
            == Color::Rgba(Bgra {
                red: 255,
                blue: 255,
                green: 255,
                alpha: 255,
            })));

        assert!(
            parser
                .parse(&mut vec![], "#FF0080")
                .is_ok_and(|result| result
                    == Color::Rgba(Bgra {
                        red: 255,
                        green: 0,
                        blue: 128,
                        alpha: 255,
                    }))
        );
        assert!(parser.parse(&mut vec![], "#FF008000").is_ok_and(|result| {
            result
                == Color::Rgba(
                    Bgra {
                        red: 255,
                        green: 0,
                        blue: 128,
                        alpha: 0,
                    }
                    .pre_mul_alpha(),
                )
        }));

        assert!(
            parser
                .parse(&mut vec![], "linear-gradient(90deg, #FFF )")
                .is_ok_and(|result| result
                    == Color::LinearGradient {
                        angle: 90,
                        colors: vec![Bgra {
                            red: 255,
                            green: 255,
                            blue: 255,
                            alpha: 255,
                        },]
                    })
        );

        assert!(
            parser
                .parse(&mut vec![], "linear-gradient( -45deg, #FFF, #000 )")
                .is_ok_and(|result| result
                    == Color::LinearGradient {
                        angle: -45,
                        colors: vec![
                            Bgra {
                                red: 255,
                                green: 255,
                                blue: 255,
                                alpha: 255,
                            },
                            Bgra {
                                red: 0,
                                green: 0,
                                blue: 0,
                                alpha: 255,
                            },
                        ]
                    })
        );

        assert!(parser.parse(&mut vec![], "linear-gradient(80deg)").is_err());
        assert!(
            parser
                .parse(&mut vec![], "linear-gradient(80deg,)")
                .is_err()
        );
        assert!(parser.parse(&mut vec![], "gradient()").is_err());
        assert!(parser.parse(&mut vec![], "linear()").is_err());
        assert!(
            parser
                .parse(&mut vec![], "linear-gradient(70deg, #FFF")
                .is_err()
        );
        assert!(parser.parse(&mut vec![], "rgb(255, 255, 255)").is_err());
        assert!(
            parser
                .parse(&mut vec![], "rgba(255, 255, 255, 255)")
                .is_err()
        );
    }

    #[test]
    fn parse_rgba_literal() {
        use widgets::types::Bgra;

        let parser = grammar::RgbaLiteralParser::new();

        assert!(parser.parse(&mut vec![], "#FFF").is_ok_and(|result| result
            == Bgra {
                red: 255,
                blue: 255,
                green: 255,
                alpha: 255
            }));
        assert!(parser.parse(&mut vec![], "#FfF").is_ok_and(|result| result
            == Bgra {
                red: 255,
                blue: 255,
                green: 255,
                alpha: 255
            }));
        assert!(
            parser
                .parse(&mut vec![], "#FFFFFF")
                .is_ok_and(|result| result
                    == Bgra {
                        red: 255,
                        blue: 255,
                        green: 255,
                        alpha: 255
                    })
        );
        assert!(
            parser
                .parse(&mut vec![], "#000000FF")
                .is_ok_and(|result| result
                    == Bgra {
                        red: 0,
                        blue: 0,
                        green: 0,
                        alpha: 255
                    })
        );
        assert!(
            parser
                .parse(&mut vec![], "#00000080")
                .is_ok_and(|result| result
                    == Bgra {
                        red: 0,
                        blue: 0,
                        green: 0,
                        alpha: 128,
                    })
        );
        assert!(parser.parse(&mut vec![], "#80808080").is_ok_and(|result| {
            result
                == Bgra {
                    red: 128,
                    blue: 128,
                    green: 128,
                    alpha: 128,
                }
                .pre_mul_alpha()
        }));

        assert!(parser.parse(&mut vec![], "#GGG").is_err());
        assert!(parser.parse(&mut vec![], "#zzz").is_err());
        assert!(parser.parse(&mut vec![], "#ZZZ").is_err());
        assert!(parser.parse(&mut vec![], "#FFFF").is_err());
        assert!(parser.parse(&mut vec![], "FFF").is_err());
        assert!(parser.parse(&mut vec![], "FFFFFF").is_err());
        assert!(parser.parse(&mut vec![], "#0Z0").is_err());
        assert!(parser.parse(&mut vec![], "#000FFFGG").is_err());
        assert!(parser.parse(&mut vec![], "#000FF").is_err());
        assert!(parser.parse(&mut vec![], "#000FFFF").is_err());
    }
}
