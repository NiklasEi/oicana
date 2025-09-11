use crate::target::TargetArgs;
use clap::Args;
use log::info;
use oicana_template::validate_native_template;

#[derive(Debug, Args)]
pub struct ValidateArgs {
    #[clap(flatten)]
    target: TargetArgs,
}

#[rustfmt::skip]
pub const VALIDATE_AFTER_HELP: &str = color_print::cstr!("\
<s><u>Examples:</></>
  oicana validate templates/invoice
  oicana validate -a
  oicana validate templates -a
");

pub fn validate(args: ValidateArgs) -> anyhow::Result<()> {
    let templates = args.target.get_targets()?;

    for template in templates {
        let validation_result = validate_native_template(template.path);
        info!("{validation_result:?}");
    }

    Ok(())
}
