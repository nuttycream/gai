use strum::VariantNames;

use crate::print::menu::MenuChosenOption;

use super::menu;
use super::renderer::Renderer;

#[derive(Debug, strum::VariantNames)]
pub enum ResponseAction {
    Apply,
    Edit,
    Regenerate,
    Exit,
}

pub(crate) fn response_actions(
    renderer: &Renderer
) -> anyhow::Result<ResponseAction> {
    let items = ResponseAction::VARIANTS;
    match menu::inline_menu(
        renderer,
        "What do you want to do?",
        items,
    )? {
        MenuChosenOption::Selected(0) => Ok(ResponseAction::Apply),

        MenuChosenOption::Selected(1) => {
            Ok(ResponseAction::Regenerate)
        }

        MenuChosenOption::Selected(2) => Ok(ResponseAction::Edit),

        _ => Ok(ResponseAction::Exit),
    }
}
