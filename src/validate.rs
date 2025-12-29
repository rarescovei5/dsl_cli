use crate::{
    argument::CliArgument,
    error::{CliError, DefinitionError},
};

pub fn validate_arguments_definition(arguments: &[CliArgument<'_>]) -> Result<(), CliError> {
    // Disallow: [optional] <required>
    let mut saw_optional: (bool, String) = (false, String::new());
    let mut saw_variadic: (bool, String) = (false, String::new());

    for arg in arguments {
        match (&saw_optional, &arg.required) {
            ((true, _), true) => {
                return Err(CliError::DefinitionError(
                    DefinitionError::RequiredArgumentAfterOptionalArgument(
                        saw_optional.1,
                        arg.name.to_string(),
                    ),
                ));
            }
            (_, false) => {
                saw_optional = (true, arg.name.to_string());
            }
            (_, true) => {}
        }

        match (&saw_variadic, &arg.variadic) {
            ((true, _), _) => {
                return Err(CliError::DefinitionError(
                    DefinitionError::VariadicArgumentBeforeArguments(saw_variadic.1),
                ));
            }
            (_, true) => {
                saw_variadic = (true, arg.name.to_string());
            }
            (_, false) => {}
        }
    }

    Ok(())
}
