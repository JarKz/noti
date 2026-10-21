use crate::{
    analysis::{CompiledTypeDeclaration, DiagnosticLabel},
    ast,
    plugin::{
        TreeBuilder, TreeNodeKind, UIntVerifier, ValidationError, ValidationErrorKind, WidgetKind,
        expect_bool_literal, expect_color_literal, expect_state_with, expect_string_literal,
        expect_type, expect_uint_literal,
    },
};

pub struct AnimatedVisibilityBuilder;

impl TreeBuilder for AnimatedVisibilityBuilder {
    fn required_fields(&self) -> &'static [&'static str] {
        &["primary_animation", "primary_spatial_change"]
    }

    fn available_fields(&self) -> &'static [&'static str] {
        &[
            "visible",
            "primary_animation",
            "secondary_animation",
            "primary_spatial_change",
            "secondary_spatial_change",
        ]
    }

    fn kind(&self) -> TreeNodeKind {
        TreeNodeKind::Widget(WidgetKind::Minimal)
    }

    fn on_validate(
        &self,
        type_declaration: &CompiledTypeDeclaration,
    ) -> Result<(), ValidationError> {
        if let Some(visible_value) = type_declaration.fields.get("visible") {
            expect_state_with("visible", visible_value, |state_value| match state_value {
                ast::Literal::Bool(_) => Ok(()),
                ast::Literal::String(_) | ast::Literal::UInt(_) | ast::Literal::Color(_) => {
                    Err(ValidationError {
                        kind: ValidationErrorKind::IncompatibleStateType,
                        message: "Incompatible state type for visible: expected bool".to_owned(),
                        labels: vec![DiagnosticLabel {
                            message: "Incompatible state type for visible: expected bool"
                                .to_owned(),
                            span: *visible_value.span(),
                        }],
                    })
                }
            })?;
        }

        // INFO: the existence of required field is ensured by default `TreeBuilder::validate`
        // method.
        for field_name in ["primary_animation", "secondary_animation"] {
            if let Some(animation_value) = type_declaration.fields.get(field_name) {
                expect_type(
                    field_name,
                    animation_value,
                    ast::TypePath(vec!["AnimationDefinition".to_owned()]),
                )?;
            }
        }

        // INFO: the existence of required field is ensured by default `TreeBuilder::validate`
        // method.
        for field_name in ["primary_spatial_change", "secondary_spatial_change"] {
            if let Some(spatial_change_value) = type_declaration.fields.get(field_name) {
                expect_type(
                    field_name,
                    spatial_change_value,
                    ast::TypePath(vec!["SpatialChangeDefinition".to_owned()]),
                )?;
            }
        }

        Ok(())
    }

    fn build(&self) {
        todo!()
    }
}

pub struct ConstrainedBoxBuilder;

impl TreeBuilder for ConstrainedBoxBuilder {
    fn required_fields(&self) -> &'static [&'static str] {
        &[]
    }

    fn available_fields(&self) -> &'static [&'static str] {
        &["min_width", "max_width", "min_height", "max_height"]
    }

    fn kind(&self) -> TreeNodeKind {
        TreeNodeKind::Widget(WidgetKind::Minimal)
    }

    fn on_validate(
        &self,
        type_declaration: &CompiledTypeDeclaration,
    ) -> Result<(), ValidationError> {
        for &field_name in self.available_fields() {
            if let Some(field_value) = type_declaration.fields.get(field_name) {
                expect_uint_literal(field_name, field_value, UIntVerifier::AcceptAny)?;
            }
        }

        Ok(())
    }

    fn build(&self) {
        todo!()
    }
}

pub struct BoxBuilder;

impl TreeBuilder for BoxBuilder {
    fn required_fields(&self) -> &'static [&'static str] {
        &[]
    }

    fn available_fields(&self) -> &'static [&'static str] {
        &["width", "height"]
    }

    fn kind(&self) -> TreeNodeKind {
        TreeNodeKind::Widget(WidgetKind::Container)
    }

    fn on_validate(
        &self,
        type_declaration: &CompiledTypeDeclaration,
    ) -> Result<(), ValidationError> {
        for field_name in ["width", "height"] {
            if let Some(width_value) = type_declaration.fields.get(field_name) {
                expect_uint_literal(field_name, width_value, UIntVerifier::AcceptAny)?;
            }
        }

        Ok(())
    }

    fn build(&self) {
        todo!()
    }
}

pub struct FlexBoxBuilder;

impl TreeBuilder for FlexBoxBuilder {
    fn required_fields(&self) -> &'static [&'static str] {
        &["direction"]
    }

    fn available_fields(&self) -> &'static [&'static str] {
        &["direction", "expand", "spacing"]
    }

    fn kind(&self) -> TreeNodeKind {
        TreeNodeKind::Widget(WidgetKind::Container)
    }

    fn on_validate(
        &self,
        type_declaration: &CompiledTypeDeclaration,
    ) -> Result<(), ValidationError> {
        let direction_value = type_declaration
            .fields
            .get("direction")
            .expect("The persistence of the field is checked by trait's default method.");

        expect_string_literal("direction", direction_value, possible_direction_values())?;

        if let Some(spacing_value) = type_declaration.fields.get("spacing") {
            expect_uint_literal("spacing", spacing_value, UIntVerifier::AcceptAny)?;
        }

        if let Some(expand_value) = type_declaration.fields.get("expand") {
            expect_bool_literal("expand", expand_value)?;
        }

        Ok(())
    }

    fn build(&self) {
        todo!()
    }
}

fn possible_direction_values() -> &'static [&'static str] {
    &["horizontal", "vertical"]
}

pub struct TextBuilder;

impl TreeBuilder for TextBuilder {
    fn required_fields(&self) -> &'static [&'static str] {
        &[]
    }

    fn available_fields(&self) -> &'static [&'static str] {
        &[
            "font",
            "wrap",
            "alignment",
            "line_spacing",
            "color",
            "value",
        ]
    }

    fn kind(&self) -> TreeNodeKind {
        TreeNodeKind::Widget(WidgetKind::Standard)
    }

    fn on_validate(
        &self,
        type_declaration: &CompiledTypeDeclaration,
    ) -> Result<(), ValidationError> {
        if let Some(wrap_value) = type_declaration.fields.get("wrap") {
            expect_bool_literal("wrap", wrap_value)?;
        }

        if let Some(alignment_value) = type_declaration.fields.get("alignment") {
            expect_string_literal("alignment", alignment_value, possible_alignment_values())?;
        }

        if let Some(line_spacing_value) = type_declaration.fields.get("line_spacing") {
            expect_uint_literal("line_spacing", line_spacing_value, UIntVerifier::AcceptAny)?;
        }

        if let Some(color_value) = type_declaration.fields.get("color") {
            expect_color_literal("color", color_value)?;
        }

        if let Some(font_value) = type_declaration.fields.get("font") {
            expect_type("font", font_value, ast::TypePath(vec!["Font".to_owned()]))?;
        }

        if let Some(value) = type_declaration.fields.get("value") {
            expect_state_with("value", value, |state_literal| match state_literal {
                ast::Literal::String(_) => Ok(()),
                ast::Literal::UInt(_) | ast::Literal::Bool(_) | ast::Literal::Color(_) => {
                    Err(ValidationError {
                        kind: ValidationErrorKind::IncompatibleStateType,
                        message: "Incompatible state type for value: expected string".to_owned(),
                        labels: vec![DiagnosticLabel {
                            message: "Incompatible state type for value: expected string"
                                .to_owned(),
                            span: *value.span(),
                        }],
                    })
                }
            })?;
        }

        Ok(())
    }

    fn build(&self) {
        todo!()
    }
}

fn possible_alignment_values() -> &'static [&'static str] {
    &["center", "left", "right", "justify"]
}

pub struct ImageBuilder;

impl TreeBuilder for ImageBuilder {
    fn required_fields(&self) -> &'static [&'static str] {
        &[]
    }

    fn available_fields(&self) -> &'static [&'static str] {
        &[
            "source",
            "width",
            "height",
            "rounding",
            "fit_mode",
            "resizing_method",
            "mipmap_mode",
        ]
    }

    fn kind(&self) -> TreeNodeKind {
        TreeNodeKind::Widget(WidgetKind::Standard)
    }

    fn on_validate(
        &self,
        type_declaration: &CompiledTypeDeclaration,
    ) -> Result<(), ValidationError> {
        if let Some(source_value) = type_declaration.fields.get("source") {
            expect_state_with("source", source_value, |state_value| match state_value {
                ast::Literal::String(_) => Ok(()),
                ast::Literal::UInt(_) | ast::Literal::Bool(_) | ast::Literal::Color(_) => {
                    Err(ValidationError {
                        kind: ValidationErrorKind::IncompatibleStateType,
                        message: "Incompatible state type for source: expected string".to_owned(),
                        labels: vec![DiagnosticLabel {
                            message: "Incompatible state type for source: expected string"
                                .to_owned(),
                            span: *source_value.span(),
                        }],
                    })
                }
            })?;
        }

        if let Some(width_value) = type_declaration.fields.get("width") {
            expect_uint_literal("width", width_value, UIntVerifier::AcceptAny)?;
        }

        if let Some(height_value) = type_declaration.fields.get("height") {
            expect_uint_literal("height", height_value, UIntVerifier::AcceptAny)?;
        }

        if let Some(rounnding_value) = type_declaration.fields.get("rounding") {
            expect_uint_literal("rounding", rounnding_value, 0..u16::MAX as u64)?;
        }

        if let Some(field_value) = type_declaration.fields.get("fit_mode") {
            expect_string_literal("fit_mode", field_value, possible_fit_mode_values())?;
        }

        if let Some(field_value) = type_declaration.fields.get("resizing_method") {
            expect_string_literal(
                "resizing_method",
                field_value,
                possible_resizing_method_values(),
            )?;
        }

        if let Some(field_value) = type_declaration.fields.get("mipmap_mode") {
            expect_string_literal("mipmap_mode", field_value, posssible_mipmap_mode_values())?;
        }

        Ok(())
    }

    fn build(&self) {
        todo!()
    }
}

fn possible_fit_mode_values() -> &'static [&'static str] {
    &["contain", "fill", "cover", "scale-down"]
}

fn possible_resizing_method_values() -> &'static [&'static str] {
    &["nearest", "linear"]
}

fn posssible_mipmap_mode_values() -> &'static [&'static str] {
    &["none", "nearest", "linear"]
}

#[cfg(test)]
mod test {
    use crate::analysis::*;
    use crate::parser;
    use crate::plugin::*;

    #[test]
    fn validate_many_widgets_in_tree() {
        let parser = parser::grammar::InputParser::new();
        let build_context = BuildContext::default().with_default_plugins();
        let mut context = SemanticAnalyzerContext::new(&build_context);

        parser
            .parse(
                &mut vec![],
                "state ImageSrc = \"pathtofilepng\";
                    state BodyText = \"hello\";
                    state TitleText = \"Alex\";

                    alias BodyText = Text(
                        value = BodyText,
                        font = Font(size = 14, style = \"regular\"),
                        alignment = \"left\"
                    );

                    alias TitleText = Text(
                        value = TitleText,
                        font = Font(size = 16, style = \"bold\"),
                        wrap = false,
                        alignment = \"center\"
                    );

                    alias TextBox = FlexBox(
                        direction = \"vertical\",
                        border = Border(
                            size = 4,
                            radius = 5,
                            color = #000
                        ),
                        padding = Spacing(all = 18),
                        spacing = 10,
                        alignment = Alignment(
                            horizontal = \"start\",
                            vertical = \"center\"
                        )
                    ) {
                        TitleText() {}
                        BodyText() {}
                    };

                    AnimatedVisibility(
                        primary_animation = AnimationDefinition(kind = \"fade\", duration = \"250ms\"),
                        primary_spatial_change = SpatialChangeDefinition(duration = \"100ms\"),
                    ) {
                        ConstrainedBox(
                            min_width = 300,
                            max_width = 400,
                            min_height = 100,
                            max_height = 150,
                        ) {
                            FlexBox(
                                direction = \"horizontal\",
                                alignment = Alignment(
                                    horizontal = \"start\",
                                    vertical = \"center\",
                                )
                            ) {
                                Image(
                                    source = ImageSrc,
                                    width = 80,
                                ) {}

                                TextBox() {}
                            }
                        }
                    }",
            )
            .unwrap()
            .analyze(&mut context, SemanticScope::default())
            .unwrap()
            .validate(&context)
            .unwrap();
    }
}
