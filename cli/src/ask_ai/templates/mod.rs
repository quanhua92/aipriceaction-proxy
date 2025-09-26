pub mod single_en;
pub mod single_vn;
pub mod multi_en;
pub mod multi_vn;
pub mod money_flow_en;
pub mod money_flow_vn;

use super::types::{AskAITemplate, Language};

pub fn get_single_ticker_templates(language: &Language) -> Vec<AskAITemplate> {
    match language {
        Language::English => single_en::get_single_ticker_templates_en(),
        Language::Vietnamese => single_vn::get_single_ticker_templates_vn(),
    }
}

pub fn get_multi_ticker_templates(language: &Language) -> Vec<AskAITemplate> {
    match language {
        Language::English => multi_en::get_multi_ticker_templates_en(),
        Language::Vietnamese => multi_vn::get_multi_ticker_templates_vn(),
    }
}

pub fn get_money_flow_templates(language: &Language) -> Vec<AskAITemplate> {
    match language {
        Language::English => money_flow_en::get_money_flow_templates_en(),
        Language::Vietnamese => money_flow_vn::get_money_flow_templates_vn(),
    }
}

pub fn get_template_by_id(id: &str, language: &Language) -> Option<AskAITemplate> {
    let all_templates = [
        get_single_ticker_templates(language),
        get_multi_ticker_templates(language),
        get_money_flow_templates(language),
    ].concat();

    all_templates.into_iter().find(|template| template.id == id)
}