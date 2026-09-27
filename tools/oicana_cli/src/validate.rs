use crate::checks::{
    check_entrypoint, check_fallback_values, check_pdf_export, check_schemas, missing_fallbacks,
    unknown_keys, SchemaSelection,
};
use crate::target::TargetArgs;
use clap::Args;
use console::{style, Emoji};
use log::info;
use oicana::template::validate_native_template;

static CHECKMARK: Emoji<'_, '_> = Emoji("✔️", "");

#[derive(Debug, Args)]
pub struct ValidateArgs {
    #[clap(flatten)]
    target: TargetArgs,
    #[arg(long, help = "Fail the validation if any warnings are reported")]
    deny_warnings: bool,
}

#[rustfmt::skip]
pub const VALIDATE_AFTER_HELP: &str = color_print::cstr!("\
<s><u>Examples:</></>
  oicana validate templates/invoice
  oicana validate -a
  oicana validate templates -a
  oicana validate -a --deny-warnings
");

pub fn validate(args: ValidateArgs) -> anyhow::Result<()> {
    let templates = args.target.get_targets()?;

    let mut all_passed = true;
    let mut passed_count = 0;
    let mut warning_count = 0;
    let template_count = templates.len();

    for template in templates {
        let validation_result = validate_native_template(&template.path);
        match validation_result {
            Err(e) => {
                eprintln!("Template {:?}: {e}", template.path);
                all_passed = false;
            }
            Ok(manifest) => {
                info!("Template {:?}: manifest valid", template.path);

                let inputs = &manifest.tool.oicana.inputs;
                let schemas = check_schemas(&template.path, &manifest, SchemaSelection::All);
                let mut errors: Vec<String> = check_entrypoint(&template.path, &manifest)
                    .err()
                    .into_iter()
                    .collect();
                errors.extend(schemas.errors);
                errors.extend(check_fallback_values(
                    &template.path,
                    inputs,
                    &schemas.validators,
                ));
                errors.extend(check_pdf_export(&manifest));
                let mut warnings = missing_fallbacks(inputs);
                warnings.extend(unknown_keys(&manifest));

                warning_count += warnings.len();
                for warning in &warnings {
                    eprintln!(
                        "{}: Template {:?}: {warning}",
                        style("Warning").yellow().for_stderr(),
                        template.path
                    );
                }

                if errors.is_empty() {
                    info!("Template {:?}: all checks passed", template.path);
                    passed_count += 1;
                    println!(
                        "{CHECKMARK}  {} valid",
                        style(&manifest.package.name).bold(),
                    );
                } else {
                    all_passed = false;
                    for error in &errors {
                        eprintln!("Template {:?}: {error}", template.path);
                    }
                }
            }
        }
    }

    if !all_passed {
        anyhow::bail!("Validation failed for one or more templates.")
    }

    if args.deny_warnings && warning_count > 0 {
        anyhow::bail!("Validation reported warnings and --deny-warnings is set.")
    }

    println!(
        "\nValidated {} template{} successfully{}",
        passed_count,
        if template_count == 1 { "" } else { "s" },
        match warning_count {
            0 => String::new(),
            1 => " with 1 warning".to_owned(),
            count => format!(" with {count} warnings"),
        },
    );

    Ok(())
}
