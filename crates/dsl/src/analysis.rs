use std::collections::HashMap;

use crate::ast::*;

// Goals of validation
//
// 1. Resovle aliases
// 2. Resolve states and references to it
// 3. Build paths to aliases, states (?)
//
// FieldValues must be resolved or ?

pub(crate) struct SemanitcAnalyzerContext {
    aliases: HashMap<String, Vec<DeclarationInfo<AliasDeclaration>>>,
    states: HashMap<String, Vec<DeclarationInfo<StateDeclaration>>>,
}

impl SemanitcAnalyzerContext {
    fn resolve_widget_declaration(
        &self,
        widget_declaration: &WidgetDeclaration,
    ) -> Result<WidgetDeclaration, AnalysisError> {
        todo!()
    }

    fn resolve_type_info(
        &self,
        type_info: &TypeInfo,
        current_path: &[PathElement],
    ) -> Result<TypeInfo, AnalysisError> {
        let type_info = if type_info.type_path.0.len() == 1 {
            if let Some(alias_declaration) = self
                .aliases
                .get(&type_info.type_path.0[0])
                .and_then(|declarations| Self::resolve_declaration(declarations, current_path))
            {
                let mut current_type_info: TypeInfo = match &alias_declaration.value.replacement {
                    AliasReplacement::Widget(_) => return Err(AnalysisError::IncompatibleType),
                    AliasReplacement::Type(type_declaration) => type_declaration.try_into()?,
                    AliasReplacement::Path(type_path) => {
                        let mut current_type_info = type_info.clone();
                        current_type_info.type_path = type_path.clone();
                        current_type_info
                    }
                };

                for (name, value) in &type_info.fields {
                    current_type_info.fields.insert(name.clone(), value.clone());
                }

                current_type_info
            } else {
                type_info.clone()
            }
        } else {
            type_info.clone()
        };

        // TODO: resolve field values

        todo!()
    }

    fn resolve_declaration<'a, T>(
        declarations: &'a [DeclarationInfo<T>],
        current_path: &[PathElement],
    ) -> Option<&'a DeclarationInfo<T>> {
        for info in declarations.iter().rev() {
            if info.path.len() > current_path.len() {
                continue;
            }

            if info
                .path
                .iter()
                .zip(current_path.iter())
                .any(|(left, right)| left.scope_identity != right.scope_identity)
            {
                continue;
            }

            return Some(info);
        }

        None
    }
}

struct DeclarationInfo<T> {
    path: Vec<PathElement>,
    value: T,
}

#[derive(PartialEq, Eq)]
pub(crate) struct PathElement {
    scope_identity: ScopeIdentity,
    position_before_next_scope: usize,
}

#[derive(PartialEq, Eq)]
struct ScopeIdentity {
    position: usize,
    kind: ScopeIdentityKind,
}

#[derive(PartialEq, Eq)]
enum ScopeIdentityKind {
    Global,
    Widget(TypePath),
}

pub(crate) trait SemanticAnalyzer {
    type Output;

    fn analyze(
        &self,
        context: &mut SemanitcAnalyzerContext,
        path: &[PathElement],
    ) -> Result<Self::Output, AnalysisError>;
}

// TODO: fill error type
pub(crate) enum AnalysisError {
    IncompatibleType,
    DuplicateFieldName,
}

impl SemanticAnalyzer for ScopeDeclaration {
    type Output = BuildScope;

    fn analyze(
        &self,
        context: &mut SemanitcAnalyzerContext,
        path: &[PathElement],
    ) -> Result<Self::Output, AnalysisError> {
        // TODO: is really needed to build a PathELement at current scope, not previous?
        let current_path_elem;
        match &self.kind {
            ScopeKind::Global => {
                current_path_elem = PathElement {
                    scope_identity: ScopeIdentity {
                        position: 0,
                        kind: ScopeIdentityKind::Global,
                    },
                    position_before_next_scope: 0,
                };
            }
            ScopeKind::Widget(widget_declaration) => {
                // TODO: resolve a widget declaration by alias name
                // if found, inherit fields
            }
        }

        for child in &self.children {
            match child {
                InScopeDeclaration::Alias(alias_declaration) => {
                    // TODO: verify that alias name is not equal to alias value (type path)
                    todo!();
                }
                InScopeDeclaration::State(state_declaration) => todo!(),
                InScopeDeclaration::Widget(scope_declaration) => todo!(),
            }
        }

        todo!()
    }
}

pub(crate) struct BuildScope {
    kind: BuildScopeKind,
    children: Vec<BuildScope>,
}

pub(crate) enum BuildScopeKind {
    Widget(WidgetDeclaration),
}

#[derive(Clone)]
struct TypeInfo {
    type_path: TypePath,
    fields: HashMap<String, FieldValue>,
}

impl TryFrom<&TypeDeclaration> for TypeInfo {
    type Error = AnalysisError;

    fn try_from(value: &TypeDeclaration) -> Result<Self, Self::Error> {
        let mut fields = HashMap::new();

        for field in &value.fields {
            let last_inserted = fields.insert(field.name.clone(), field.value.clone());

            if last_inserted.is_some() {
                return Err(AnalysisError::DuplicateFieldName);
            }
        }

        Ok(Self {
            type_path: value.type_path.clone(),
            fields,
        })
    }
}
