use crate::check_code::verify_effect;
use crate::CodeVerifier;
use std::sync::Arc;
use syntax::code::{Effects, FinalizedEffects};
use syntax::operation_util::OperationGetter;
use syntax::r#struct::StructData;
use syntax::{Attribute, ParsingError, SimpleVariableManager};

pub async fn check_operator(
    code_verifier: &mut CodeVerifier<'_>,
    variables: &mut SimpleVariableManager,
    effect: Effects,
) -> Result<FinalizedEffects, ParsingError> {
    let operation;
    let mut values;
    if let Effects::Operation(new_operation, new_values) = effect {
        operation = new_operation;
        values = new_values;
    } else {
        unreachable!()
    }

    let error = ParsingError::new(
        String::default(),
        (0, 0),
        0,
        (0, 0),
        0,
        format!("Failed to find operation {} with {:?}", operation, values),
    );
    let mut outer_operation = None;
    // Check if it's two operations that should be combined, like a list ([])
    if values.len() > 0 {
        let mut reading_array = None;
        let mut last = values.pop().unwrap();
        if let Effects::CreateArray(mut effects) = last {
            if effects.len() > 0 {
                last = effects.pop().unwrap();
                reading_array = Some(effects);
            } else {
                last = Effects::CreateArray(vec![]);
            }
        }

        if let Effects::Operation(inner_operation, effects) = last {
            if operation.ends_with("{}") && inner_operation.starts_with("{}") {
                let combined = operation[0..operation.len() - 2].to_string() + &inner_operation;
                let new_operation =
                    if operation.starts_with("{}") && inner_operation.ends_with("{}") {
                        let mut output = vec![];
                        for i in 0..combined.len() - operation.len() - 2 {
                            let mut temp = combined.clone();
                            temp.truncate(operation.len() + i);
                            output.push(temp);
                        }
                        output
                    } else {
                        vec![combined.clone()]
                    };

                let getter = OperationGetter {
                    syntax: code_verifier.syntax.clone(),
                    operation: new_operation.clone(),
                    error: error.clone(),
                };

                if let Ok(found) = getter.await {
                    let new_operation = Attribute::find_attribute("operation", &found.attributes)
                        .unwrap()
                        .as_string_attribute()
                        .unwrap();

                    let mut inner_array = false;
                    if let Some(found) = reading_array {
                        values.push(Effects::CreateArray(found));
                        inner_array = true;
                    }
                    if new_operation.len() >= combined.len() {
                        if inner_array {
                            if let Effects::CreateArray(last) = values.last_mut().unwrap() {
                                for effect in effects {
                                    last.push(effect);
                                }
                            }
                        } else {
                            for effect in effects {
                                values.push(effect);
                            }
                        }
                        outer_operation = Some(found);
                    } else {
                        let new_inner = "{}".to_string()
                            + &combined[new_operation.replace("{+}", "{}").len()..];

                        let inner_data = OperationGetter {
                            syntax: code_verifier.syntax.clone(),
                            operation: vec![new_inner.clone()],
                            error: error.clone(),
                        }
                        .await?;

                        (outer_operation, values) = assign_with_priority(
                            new_operation.clone(),
                            &found,
                            values,
                            new_inner,
                            &inner_data,
                            effects,
                            inner_array,
                        );
                    }
                } else {
                    if let Some(mut found) = reading_array {
                        if let Effects::CreateArray(inner) = found.last_mut().unwrap() {
                            inner.push(Effects::Operation(inner_operation, effects));
                        } else {
                            panic!("Expected array!");
                        }
                    } else {
                        let outer_data = OperationGetter {
                            syntax: code_verifier.syntax.clone(),
                            operation: vec![operation.clone()],
                            error: error.clone(),
                        }
                        .await?;
                        let inner_data = OperationGetter {
                            syntax: code_verifier.syntax.clone(),
                            operation: vec![inner_operation.clone()],
                            error: error.clone(),
                        }
                        .await?;

                        (outer_operation, values) = assign_with_priority(
                            operation.clone(),
                            &outer_data,
                            values,
                            inner_operation,
                            &inner_data,
                            effects,
                            false,
                        );
                    }
                }
            } else {
                if let Some(mut found) = reading_array {
                    if let Effects::CreateArray(inner) = found.last_mut().unwrap() {
                        inner.push(Effects::Operation(inner_operation, effects));
                    } else {
                        panic!("Expected array!");
                    }
                } else {
                    values.push(Effects::Operation(inner_operation, effects));
                }
            }
        } else {
            if let Some(mut found) = reading_array {
                if let Effects::CreateArray(inner) = found.last_mut().unwrap() {
                    inner.push(last);
                } else {
                    panic!("Expected array!");
                }
            } else {
                values.push(last);
            }
        }
    }

    let operation = if let Some(found) = outer_operation {
        found
    } else {
        OperationGetter {
            syntax: code_verifier.syntax.clone(),
            operation: vec![operation],
            error,
        }
        .await?
    };

    if Attribute::find_attribute("operation", &operation.attributes)
        .unwrap()
        .as_string_attribute()
        .unwrap()
        .contains("{+}")
    {
        if let Effects::CreateArray(_) = values.first().unwrap() {
        } else {
            let effect = Effects::CreateArray(vec![values.remove(0)]);
            values.push(effect);
        }
    }

    let calling;
    if values.len() > 0 {
        calling = Box::new(values.remove(0));
    } else {
        calling = Box::new(Effects::NOP);
    }

    return verify_effect(
        code_verifier,
        variables,
        Effects::ImplementationCall(
            calling,
            operation.name.clone(),
            String::default(),
            values,
            None,
        ),
    )
    .await;
}

pub fn assign_with_priority(
    operation: String,
    found: &Arc<StructData>,
    mut values: Vec<Effects>,
    inner_operator: String,
    inner_data: &Arc<StructData>,
    mut inner_effects: Vec<Effects>,
    inner_array: bool,
) -> (Option<Arc<StructData>>, Vec<Effects>) {
    let op_priority = Attribute::find_attribute("priority", &found.attributes)
        .map(|inner| inner.as_int_attribute().unwrap_or(0))
        .unwrap_or(0);
    let op_parse_left = Attribute::find_attribute("parse_left", &found.attributes)
        .map(|inner| inner.as_bool_attribute().unwrap_or(false))
        .unwrap_or(false);
    let lhs_priority = Attribute::find_attribute("priority", &inner_data.attributes)
        .map(|inner| inner.as_int_attribute().unwrap_or(0))
        .unwrap_or(0);

    return if lhs_priority < op_priority || (!op_parse_left && lhs_priority == op_priority) {
        if inner_array {
            if let Effects::CreateArray(inner) = values.last_mut().unwrap() {
                inner.push(inner_effects.remove(0));
            } else {
                panic!("Assumed op args ended with an array when they didn't!")
            }
        } else {
            values.push(inner_effects.remove(0));
        }
        inner_effects.insert(0, Effects::Operation(operation, values));
        (Some(inner_data.clone()), inner_effects)
    } else {
        values.push(Effects::Operation(inner_operator, inner_effects));
        (Some(found.clone()), values)
    };
}
