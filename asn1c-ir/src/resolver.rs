use crate::ir::*;
use crate::error::IrError;
use std::collections::HashMap;

pub struct Resolver {
    modules: HashMap<String, AsnModule>,
    current_module: Option<String>,
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            current_module: None,
        }
    }

    pub fn modules(&self) -> &HashMap<String, AsnModule> {
        &self.modules
    }

    pub fn add_module(&mut self, module: AsnModule) -> Result<(), IrError> {
        let name = module.name.clone();
        self.modules.insert(name.clone(), module);
        self.current_module = Some(name);
        Ok(())
    }

    pub fn resolve(&mut self) -> Result<(), IrError> {
        for module_name in self.modules.keys().cloned().collect::<Vec<_>>() {
            self.current_module = Some(module_name.clone());
            self.resolve_imports(&module_name)?;
            self.resolve_types(&module_name)?;
        }
        self.detect_circular_references()?;
        Ok(())
    }

    fn resolve_imports(&self, module_name: &str) -> Result<(), IrError> {
        let module = self.modules.get(module_name).ok_or_else(|| {
            IrError::UnknownModule(module_name.to_string())
        })?;

        for import in &module.imports {
            let imported_module = self.modules.get(&import.module).ok_or_else(|| {
                IrError::UnknownModule(import.module.clone())
            })?;

            for symbol in &import.symbols {
                match &imported_module.exports {
                    Exports::All => {}
                    Exports::Symbols(symbols) => {
                        if !symbols.contains(symbol) {
                            return Err(IrError::UnexportedSymbol(
                                symbol.clone(),
                                import.module.clone(),
                            ));
                        }
                    }
                    Exports::None => {
                        return Err(IrError::UnexportedSymbol(
                            symbol.clone(),
                            import.module.clone(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn resolve_types(&mut self, module_name: &str) -> Result<(), IrError> {
        let module = self.modules.get(module_name).ok_or_else(|| {
            IrError::UnknownModule(module_name.to_string())
        })?;

        let resolved_types: Vec<AsnType> = module
            .types
            .iter()
            .map(|t| self.resolve_type(&t.ty, module_name))
            .collect::<Result<Vec<_>, _>>()?;

        for (i, ty) in resolved_types.into_iter().enumerate() {
            self.modules.get_mut(module_name).unwrap().types[i].ty = ty;
        }

        Ok(())
    }

    fn is_complex_type(ty: &AsnType) -> bool {
        match ty {
            AsnType::Sequence { .. } | AsnType::Set { .. } | AsnType::Choice { .. } => true,
            AsnType::ConstrainedType { base, .. } => Self::is_complex_type(base),
            AsnType::Tagged { inner, .. } => Self::is_complex_type(inner),
            _ => false,
        }
    }

    fn resolve_type(
        &self,
        ty: &AsnType,
        module_name: &str,
    ) -> Result<AsnType, IrError> {
        match ty {
            AsnType::ReferencedType { module, name } => {
                let target_module = module.as_deref().unwrap_or(module_name);
                let target = self.modules.get(target_module).ok_or_else(|| {
                    IrError::UnknownModule(target_module.to_string())
                })?;

                let type_def = target.types.iter().find(|t| &t.name == name).ok_or_else(|| {
                    IrError::TypeNotFound(name.clone())
                })?;

                let resolved = self.resolve_type(&type_def.ty, target_module)?;

                if Self::is_complex_type(&resolved) {
                    Ok(AsnType::ReferencedType {
                        module: module.clone(),
                        name: name.clone(),
                    })
                } else {
                    Ok(resolved)
                }
            }
            AsnType::Sequence { fields, ext } => {
                let resolved_fields: Result<Vec<SequenceField>, IrError> = fields
                    .iter()
                    .map(|f| {
                        Ok(SequenceField {
                            name: f.name.clone(),
                            ty: self.resolve_type(&f.ty, module_name)?,
                            optional: f.optional,
                            default: f.default.clone(),
                        })
                    })
                    .collect();

                let resolved_ext: Option<Result<Vec<SequenceField>, IrError>> = ext.as_ref().map(|fields| {
                    fields
                        .iter()
                        .map(|f| {
                            Ok(SequenceField {
                                name: f.name.clone(),
                                ty: self.resolve_type(&f.ty, module_name)?,
                                optional: f.optional,
                                default: f.default.clone(),
                            })
                        })
                        .collect()
                });

                Ok(AsnType::Sequence {
                    fields: resolved_fields?,
                    ext: resolved_ext.transpose()?,
                })
            }
            AsnType::Set { fields, ext } => {
                let resolved_fields: Result<Vec<SequenceField>, IrError> = fields
                    .iter()
                    .map(|f| {
                        Ok(SequenceField {
                            name: f.name.clone(),
                            ty: self.resolve_type(&f.ty, module_name)?,
                            optional: f.optional,
                            default: f.default.clone(),
                        })
                    })
                    .collect();

                let resolved_ext: Option<Result<Vec<SequenceField>, IrError>> = ext.as_ref().map(|fields| {
                    fields
                        .iter()
                        .map(|f| {
                            Ok(SequenceField {
                                name: f.name.clone(),
                                ty: self.resolve_type(&f.ty, module_name)?,
                                optional: f.optional,
                                default: f.default.clone(),
                            })
                        })
                        .collect()
                });

                Ok(AsnType::Set {
                    fields: resolved_fields?,
                    ext: resolved_ext.transpose()?,
                })
            }
            AsnType::Choice { alternatives, ext } => {
                let resolved_alts: Result<Vec<ChoiceAlternative>, IrError> = alternatives
                    .iter()
                    .map(|a| {
                        Ok(ChoiceAlternative {
                            name: a.name.clone(),
                            ty: self.resolve_type(&a.ty, module_name)?,
                        })
                    })
                    .collect();

                let resolved_ext: Option<Result<Vec<ChoiceAlternative>, IrError>> = ext.as_ref().map(|alts| {
                    alts.iter()
                        .map(|a| {
                            Ok(ChoiceAlternative {
                                name: a.name.clone(),
                                ty: self.resolve_type(&a.ty, module_name)?,
                            })
                        })
                        .collect()
                });

                Ok(AsnType::Choice {
                    alternatives: resolved_alts?,
                    ext: resolved_ext.transpose()?,
                })
            }
            AsnType::SequenceOf { element_type } => {
                // Don't resolve element type - preserve ReferencedType name
                // so codegen can use the actual type alias name for encode/decode
                Ok(AsnType::SequenceOf {
                    element_type: element_type.clone(),
                })
            }
            AsnType::SetOf { element_type } => {
                Ok(AsnType::SetOf {
                    element_type: element_type.clone(),
                })
            }
            AsnType::Tagged { class, number, implicit, inner } => {
                Ok(AsnType::Tagged {
                    class: class.clone(),
                    number: *number,
                    implicit: *implicit,
                    inner: Box::new(self.resolve_type(inner, module_name)?),
                })
            }
            AsnType::ConstrainedType { base, constraints } => {
                Ok(AsnType::ConstrainedType {
                    base: Box::new(self.resolve_type(base, module_name)?),
                    constraints: constraints.clone(),
                })
            }
            _ => Ok(ty.clone()),
        }
    }

    fn detect_circular_references(&self) -> Result<(), IrError> {
        for module in self.modules.values() {
            for type_assignment in &module.types {
                self.check_type_for_cycles(&type_assignment.name, &type_assignment.ty, &[], &module.name)?;
            }
        }
        Ok(())
    }

    fn check_type_for_cycles(
        &self,
        original_name: &str,
        ty: &AsnType,
        chain: &[String],
        current_module: &str,
    ) -> Result<(), IrError> {
        match ty {
            AsnType::ReferencedType { module, name } => {
                if chain.contains(name) {
                    return Err(IrError::CircularReference(
                        chain.iter().chain(Some(name)).cloned().collect::<Vec<_>>().join(" -> "),
                    ));
                }
                if name == original_name {
                    return Err(IrError::CircularReference(name.clone()));
                }
                let target_module = module.as_deref().unwrap_or(current_module);
                if let Some(target) = self.modules.get(target_module) {
                    if let Some(type_def) = target.types.iter().find(|t| &t.name == name) {
                        let new_chain: Vec<String> = chain.iter()
                            .cloned()
                            .chain(Some(name.clone()))
                            .collect();
                        self.check_type_for_cycles(original_name, &type_def.ty, &new_chain, target_module)?;
                    }
                }
            }
            AsnType::Sequence { fields, ext } => {
                for field in fields {
                    self.check_type_for_cycles(original_name, &field.ty, chain, current_module)?;
                }
                if let Some(ext_fields) = ext {
                    for field in ext_fields {
                        self.check_type_for_cycles(original_name, &field.ty, chain, current_module)?;
                    }
                }
            }
            AsnType::Set { fields, ext } => {
                for field in fields {
                    self.check_type_for_cycles(original_name, &field.ty, chain, current_module)?;
                }
                if let Some(ext_fields) = ext {
                    for field in ext_fields {
                        self.check_type_for_cycles(original_name, &field.ty, chain, current_module)?;
                    }
                }
            }
            AsnType::Choice { alternatives, ext } => {
                for alt in alternatives {
                    self.check_type_for_cycles(original_name, &alt.ty, chain, current_module)?;
                }
                if let Some(ext_alts) = ext {
                    for alt in ext_alts {
                        self.check_type_for_cycles(original_name, &alt.ty, chain, current_module)?;
                    }
                }
            }
            AsnType::SequenceOf { element_type } => {
                self.check_type_for_cycles(original_name, element_type, chain, current_module)?;
            }
            AsnType::SetOf { element_type } => {
                self.check_type_for_cycles(original_name, element_type, chain, current_module)?;
            }
            AsnType::Tagged { inner, .. } => {
                self.check_type_for_cycles(original_name, inner, chain, current_module)?;
            }
            AsnType::ConstrainedType { base, .. } => {
                self.check_type_for_cycles(original_name, base, chain, current_module)?;
            }
            _ => {}
        }
        Ok(())
    }
}
