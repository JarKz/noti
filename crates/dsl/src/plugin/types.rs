use crate::{
    analysis::{CompiledTypeDeclaration, DiagnosticLabel},
    plugin::{
        StringVerifier, TreeBuilder, TreeNodeKind, UIntVerifier, ValidationError,
        ValidationErrorKind, VerificationContext, expect_color_literal, expect_string_literal,
        expect_uint_literal,
    },
};

pub struct BorderBuilder;

impl TreeBuilder for BorderBuilder {
    fn required_fields(&self) -> &'static [&'static str] {
        &[]
    }

    fn available_fields(&self) -> &'static [&'static str] {
        &["size", "radius", "color"]
    }

    fn kind(&self) -> TreeNodeKind {
        TreeNodeKind::Type
    }

    fn on_validate(
        &self,
        type_declaration: &CompiledTypeDeclaration,
    ) -> Result<(), ValidationError> {
        if let Some(size_value) = type_declaration.fields.get("size") {
            expect_uint_literal("size", size_value, UIntVerifier::AcceptAny)?;
        }

        if let Some(radius_value) = type_declaration.fields.get("radius") {
            expect_uint_literal("radius", radius_value, UIntVerifier::AcceptAny)?;
        }

        if let Some(color_value) = type_declaration.fields.get("color") {
            expect_color_literal("color", color_value)?;
        }

        Ok(())
    }

    fn build(&self) {
        todo!()
    }
}

pub struct FontBuilder;

impl TreeBuilder for FontBuilder {
    fn required_fields(&self) -> &'static [&'static str] {
        &[]
    }

    fn available_fields(&self) -> &'static [&'static str] {
        &["name", "size", "style"]
    }

    fn kind(&self) -> TreeNodeKind {
        TreeNodeKind::Type
    }

    fn on_validate(
        &self,
        type_declaration: &CompiledTypeDeclaration,
    ) -> Result<(), ValidationError> {
        if let Some(name_field) = type_declaration.fields.get("name") {
            expect_string_literal("name", name_field, StringVerifier::AcceptAny)?;
        }

        if let Some(size_field) = type_declaration.fields.get("size") {
            expect_uint_literal("size", size_field, UIntVerifier::AcceptAny)?;
        }

        if let Some(style_field) = type_declaration.fields.get("style") {
            expect_string_literal(
                "style",
                style_field,
                &["regular", "bold", "italic", "bold-italic"][..],
            )?;
        }

        Ok(())
    }

    fn build(&self) {
        todo!()
    }
}

pub struct AnimationDefinitionBuilder;

impl TreeBuilder for AnimationDefinitionBuilder {
    fn required_fields(&self) -> &'static [&'static str] {
        &["duration", "kind"]
    }

    fn available_fields(&self) -> &'static [&'static str] {
        &["easing", "duration", "kind"]
    }

    fn kind(&self) -> TreeNodeKind {
        TreeNodeKind::Type
    }

    fn on_validate(
        &self,
        type_declaration: &CompiledTypeDeclaration,
    ) -> Result<(), ValidationError> {
        if let Some(easing_value) = type_declaration.fields.get("easing") {
            expect_string_literal("easing", easing_value, possible_easing_values())?;
        }

        if let Some(duration_value) = type_declaration.fields.get("duration") {
            expect_string_literal("duration", duration_value, &verify_dufration)?;
        }

        if let Some(kind_value) = type_declaration.fields.get("kind") {
            expect_string_literal("kind", kind_value, possible_animation_kind_values())?;
        }

        Ok(())
    }

    fn build(&self) {
        todo!()
    }
}

pub struct SpatialChangeDefinitionBuilder;

impl TreeBuilder for SpatialChangeDefinitionBuilder {
    fn required_fields(&self) -> &'static [&'static str] {
        &["duration"]
    }

    fn available_fields(&self) -> &'static [&'static str] {
        &["easing", "duration"]
    }

    fn kind(&self) -> TreeNodeKind {
        TreeNodeKind::Type
    }

    fn on_validate(
        &self,
        type_declaration: &CompiledTypeDeclaration,
    ) -> Result<(), ValidationError> {
        if let Some(easing_value) = type_declaration.fields.get("easing") {
            expect_string_literal("easing", easing_value, possible_easing_values())?;
        }

        if let Some(duration_value) = type_declaration.fields.get("duration") {
            expect_string_literal("duration", duration_value, &verify_dufration)?;
        }

        Ok(())
    }

    fn build(&self) {
        todo!()
    }
}

fn possible_easing_values() -> &'static [&'static str] {
    &["linear", "ease-out", "ease-in-out"]
}

fn possible_animation_kind_values() -> &'static [&'static str] {
    &["fade", "pop", "translate"]
}

fn verify_dufration(
    duration: &str,
    VerificationContext {
        field_name,
        field_value,
    }: VerificationContext,
) -> Result<(), ValidationError> {
    humantime::parse_duration(duration)
        .map(|_| ())
        .map_err(|_| ValidationError {
            kind: super::ValidationErrorKind::InvalidFieldValue,
            message: format!("Invalid value for {field_name}"),
            labels: vec![
                DiagnosticLabel {
                    message: format!("Invalid value for {field_name}: expected a human time like \"5s\" or \"4s 30ms\""),
                    span: *field_value.span(),
                }
            ]

        })
}

pub struct AlignmentBuilder;

impl TreeBuilder for AlignmentBuilder {
    fn required_fields(&self) -> &'static [&'static str] {
        &[]
    }

    fn available_fields(&self) -> &'static [&'static str] {
        &["horizontal", "vertical"]
    }

    fn kind(&self) -> TreeNodeKind {
        TreeNodeKind::Type
    }

    fn on_validate(
        &self,
        type_declaration: &CompiledTypeDeclaration,
    ) -> Result<(), ValidationError> {
        for field_name in ["horizontal", "vertical"] {
            if let Some(position_value) = type_declaration.fields.get(field_name) {
                expect_string_literal(field_name, position_value, possible_position_values())?;
            }
        }

        Ok(())
    }

    fn build(&self) {
        todo!()
    }
}

fn possible_position_values() -> &'static [&'static str] {
    &["start", "end", "center", "space-between"]
}

pub struct SpacingBuilder;

impl TreeBuilder for SpacingBuilder {
    fn required_fields(&self) -> &'static [&'static str] {
        &[]
    }

    fn available_fields(&self) -> &'static [&'static str] {
        &[
            "top",
            "right",
            "bottom",
            "left",
            "horizontal",
            "vertical",
            "all",
        ]
    }

    fn kind(&self) -> TreeNodeKind {
        TreeNodeKind::Type
    }

    fn on_validate(
        &self,
        type_declaration: &CompiledTypeDeclaration,
    ) -> Result<(), ValidationError> {
        #[derive(Default)]
        struct ClashNameVerifier {
            top: bool,
            bottom: bool,
            right: bool,
            left: bool,
        }

        let mut clash_name_verifier = ClashNameVerifier::default();

        let mut horizontal_fields_diagnostic_labels = vec![];
        let mut vertical_fields_diagnostic_labels = vec![];

        if let Some(top_value) = type_declaration.fields.get("top") {
            expect_uint_literal("top", top_value, UIntVerifier::AcceptAny)?;
            clash_name_verifier.top = true;

            vertical_fields_diagnostic_labels.push(DiagnosticLabel {
                message: "The top field is declared here".to_owned(),
                span: *type_declaration.get_field_span("top").expect(
                    "The field span usually persist on creation of CompiledTypeDeclaration",
                ),
            })
        }

        if let Some(bottom_value) = type_declaration.fields.get("bottom") {
            expect_uint_literal("bottom", bottom_value, UIntVerifier::AcceptAny)?;
            clash_name_verifier.bottom = true;

            vertical_fields_diagnostic_labels.push(DiagnosticLabel {
                message: "The bottom field is declared here".to_owned(),
                span: *type_declaration.get_field_span("bottom").expect(
                    "The field span usually persist on creation of CompiledTypeDeclaration",
                ),
            })
        }

        if let Some(right_value) = type_declaration.fields.get("right") {
            expect_uint_literal("right", right_value, UIntVerifier::AcceptAny)?;
            clash_name_verifier.right = true;

            horizontal_fields_diagnostic_labels.push(DiagnosticLabel {
                message: "The right field is declared here".to_owned(),
                span: *type_declaration.get_field_span("right").expect(
                    "The field span usually persist on creation of CompiledTypeDeclaration",
                ),
            })
        }

        if let Some(left_value) = type_declaration.fields.get("left") {
            expect_uint_literal("left", left_value, UIntVerifier::AcceptAny)?;
            clash_name_verifier.left = true;

            horizontal_fields_diagnostic_labels.push(DiagnosticLabel {
                message: "The left field is declared here".to_owned(),
                span: *type_declaration.get_field_span("left").expect(
                    "The field span usually persist on creation of CompiledTypeDeclaration",
                ),
            })
        }

        if let Some(horizontal_value) = type_declaration.fields.get("horizontal") {
            if clash_name_verifier.left || clash_name_verifier.right {
                let mut labels = vec![DiagnosticLabel {
                    message: "The horizontal field is declared here".to_owned(),
                    span: *type_declaration.get_field_span("horizontal").expect(
                        "The field span usually persist on creation of CompiledTypeDeclaration",
                    ),
                }];
                labels.extend(horizontal_fields_diagnostic_labels);

                return Err(ValidationError {
                    kind: ValidationErrorKind::MisusedFieldName,
                    message: "Misusused `horizontal` field for Spacing: it cannot be used with `left` or `right` fields".to_owned(),
                    labels,
                });
            }

            clash_name_verifier.left = true;
            clash_name_verifier.right = true;
            expect_uint_literal("horizontal", horizontal_value, UIntVerifier::AcceptAny)?;
        }

        if let Some(vertical_value) = type_declaration.fields.get("vertical") {
            if clash_name_verifier.top || clash_name_verifier.bottom {
                let mut labels = vec![DiagnosticLabel {
                    message: "The vertical field is declared here".to_owned(),
                    span: *type_declaration.get_field_span("vertical").expect(
                        "The field span usually persist on creation of CompiledTypeDeclaration",
                    ),
                }];
                labels.extend(vertical_fields_diagnostic_labels);

                return Err(ValidationError {
                    kind: ValidationErrorKind::MisusedFieldName,
                    message: "Misusused `vertical` field for Spacing: it cannot be used with `top` or `bottom` fields".to_owned(),
                    labels,
                });
            }

            clash_name_verifier.top = true;
            clash_name_verifier.bottom = true;
            expect_uint_literal("vertical", vertical_value, UIntVerifier::AcceptAny)?;
        }

        if let Some(all_value) = type_declaration.fields.get("all") {
            if clash_name_verifier.top
                || clash_name_verifier.bottom
                || clash_name_verifier.right
                || clash_name_verifier.left
            {
                let mut labels = vec![DiagnosticLabel {
                    message: "The `all` field is declared here".to_owned(),
                    span: *type_declaration.get_field_span("all").expect(
                        "The field span usually persist on creation of CompiledTypeDeclaration",
                    ),
                }];
                labels.extend(horizontal_fields_diagnostic_labels);
                labels.extend(vertical_fields_diagnostic_labels);

                return Err(ValidationError {
                    kind: ValidationErrorKind::MisusedFieldName,
                    message: "Misusused `all` field for Spacing: it cannot be used with `top`, `right`, `bottom` or `left` fields".to_owned(),
                    labels,
                });
            }

            expect_uint_literal("all", all_value, UIntVerifier::AcceptAny)?;
        }

        Ok(())
    }

    fn build(&self) {
        todo!()
    }
}


#[cfg(test)]
mod test {
    use crate::plugin::*;
    use crate::analysis::*;
    use crate::parser;

    #[test]
    fn border_validation() {
        let parser = parser::grammar::InputParser::new();
        let build_context = BuildContext::default().with_default_plugins();
        let mut context = SemanticAnalyzerContext::new(&build_context);

        assert!(
            parser
                .parse(
                    &mut vec![], 
                    "FlexBox(direction = \"horizontal\", border = Border(size = 5, color = #FFF, radius = 4)) {}"
                )
                .unwrap()
                .analyze(&mut context, SemanticScope::default())
                .unwrap()
                .validate(&context)
                .is_ok()
        );

        assert!(
            parser
                .parse(
                    &mut vec![], 
                    "FlexBox(direction = \"horizontal\", border = Border(size = \"big\", color = #FFF, radius = 4)) {}"
                )
                .unwrap()
                .analyze(&mut context, SemanticScope::default())
                .unwrap()
                .validate(&context)
                .is_err_and(|err| err.kind == ValidationErrorKind::MismatchedFieldValueType)
        );

        assert!(
            parser
                .parse(
                    &mut vec![], 
                    "FlexBox(direction = \"horizontal\", border = Border(size = 3, thickenss = 4)) {}"
                )
                .unwrap()
                .analyze(&mut context, SemanticScope::default())
                .unwrap()
                .validate(&context)
                .is_err_and(|err| err.kind == ValidationErrorKind::UnknownField)
        );
    }

    #[test]
    fn font_validation() {
        let parser = parser::grammar::InputParser::new();
        let build_context = BuildContext::default().with_default_plugins();
        let mut context = SemanticAnalyzerContext::new(&build_context);

        assert!(
            parser
                .parse(
                    &mut vec![], 
                    "Text(font = Font(size = 14, name = \"Noto Sans\", style = \"bold\")) {}"
                )
                .unwrap()
                .analyze(&mut context, SemanticScope::default())
                .unwrap()
                .validate(&context)
                .is_ok()
        );

        assert!(
            parser
                .parse(
                    &mut vec![], 
                    "state fontName = \"Noto Sans\";
                    Text(font = Font(size = 14, name = fontName, style = \"bold\")) {}"
                )
                .unwrap()
                .analyze(&mut context, SemanticScope::default())
                .unwrap()
                .validate(&context)
                .is_err_and(|err| err.kind == ValidationErrorKind::MismatchedFieldValueType)
        );
    }

    #[test]
    fn animation_definition_validation() {
        let parser = parser::grammar::InputParser::new();
        let build_context = BuildContext::default().with_default_plugins();
        let mut context = SemanticAnalyzerContext::new(&build_context);

        assert!(
            parser
                .parse(
                    &mut vec![], 
                    "AnimatedVisibility(
                        primary_animation = AnimationDefinition(kind = \"fade\", duration = \"300ms\"),
                        secondary_animation = AnimationDefinition(kind = \"pop\", duration = \"500ms\"),
                        primary_spatial_change = SpatialChangeDefinition(easing = \"ease-out\", duration = \"200ms\"),
                        secondary_spatial_change = SpatialChangeDefinition(duration = \"100ms\"),
                    ) {}"
                )
                .unwrap()
                .analyze(&mut context, SemanticScope::default())
                .unwrap()
                .validate(&context)
                .is_ok()
        );
    }

    #[test]
    fn spacing_validation() {
        let parser = parser::grammar::InputParser::new();
        let build_context = BuildContext::default().with_default_plugins();
        let mut context = SemanticAnalyzerContext::new(&build_context);

        assert!(
            parser
                .parse(
                    &mut vec![], 
                    "Box(
                        padding = Spacing(all = 5),
                        margin = Spacing(horizontal = 5, top = 3)
                    ) {}"
                )
                .unwrap()
                .analyze(&mut context, SemanticScope::default())
                .unwrap()
                .validate(&context)
                .is_ok()
        );

        assert!(
            parser
                .parse(
                    &mut vec![], 
                    "Box(padding = Spacing(horizontal = 3, left = 4)) {}"
                )
                .unwrap()
                .analyze(&mut context, SemanticScope::default())
                .unwrap()
                .validate(&context)
                .is_err_and(|err| err.kind == ValidationErrorKind::MisusedFieldName)
        );

        assert!(
            parser
                .parse(
                    &mut vec![], 
                    "Box(padding = Spacing(vertical = 3, top = 4)) {}"
                )
                .unwrap()
                .analyze(&mut context, SemanticScope::default())
                .unwrap()
                .validate(&context)
                .is_err_and(|err| err.kind == ValidationErrorKind::MisusedFieldName)
        );

        assert!(
            parser
                .parse(
                    &mut vec![], 
                    "Box(padding = Spacing(horizontal = 3, vertical = 4)) {}"
                )
                .unwrap()
                .analyze(&mut context, SemanticScope::default())
                .unwrap()
                .validate(&context)
                .is_ok()
        );
    }
}
