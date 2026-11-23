use super::*;
use serde::Deserialize;
use std::{collections::HashMap, sync::OnceLock};
#[derive(Deserialize)]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub glyph: String,
    pub family: String,
}
pub fn all() -> &'static [Entry] {
    static CATALOG: OnceLock<Vec<Entry>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let mut entries = vec![
            Entry {
                id: "brand:voicemeeter".into(),
                name: "Voicemeeter".into(),
                glyph: String::new(),
                family: String::new(),
            },
            Entry {
                id: "brand:chatgpt".into(),
                name: "ChatGPT".into(),
                glyph: String::new(),
                family: String::new(),
            },
        ];
        entries.extend(
            serde_json::from_str::<Vec<Entry>>(include_str!("../../../assets/icons/catalog.json"))
                .expect("validated bundled icon catalog"),
        );
        entries
    })
}
pub fn find(id: &str) -> Option<&'static Entry> {
    static LOOKUP: OnceLock<HashMap<&'static str, usize>> = OnceLock::new();
    LOOKUP
        .get_or_init(|| {
            all()
                .iter()
                .enumerate()
                .map(|(i, e)| (e.id.as_str(), i))
                .collect()
        })
        .get(id)
        .map(|&i| &all()[i])
}
pub fn refresh(ui: &Editor) {
    let query = ui.get_icon_query().to_lowercase();
    let library = ui.get_icon_library();
    let entries = all()
        .iter()
        .filter(|e| {
            (library == 0
                || (library == 1 && e.id.starts_with("bi:"))
                || (library == 2 && e.id.starts_with("md:"))
                || (library == 3 && e.id.starts_with("brand:")))
                && query
                    .split_whitespace()
                    .all(|word| e.name.to_lowercase().contains(word) || e.id.contains(word))
        })
        .collect::<Vec<_>>();
    let max_page = entries.len().saturating_sub(1) / 60;
    let page = (ui.get_icon_page().max(0) as usize).min(max_page);
    ui.set_icon_page(page as i32);
    ui.set_icon_count(entries.len() as i32);
    ui.set_icon_choices(ModelRc::new(VecModel::from(
        entries
            .into_iter()
            .skip(page * 60)
            .take(60)
            .map(|e| IconChoice {
                image: ui
                    .global::<BrandIcons>()
                    .invoke_image_for_id(e.id.clone().into()),
                id: e.id.clone().into(),
                label: e.name.clone().into(),
                glyph: e.glyph.clone().into(),
                family: e.family.clone().into(),
            })
            .collect::<Vec<_>>(),
    )));
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bundled_catalog_has_both_libraries_and_valid_glyphs() {
        assert!(all().len() > 4000);
        assert!(find("bi:alarm").is_some());
        assert!(find("md:home").is_some());
        assert_eq!(
            all().len(),
            all()
                .iter()
                .map(|e| &e.id)
                .collect::<std::collections::HashSet<_>>()
                .len()
        );
        assert!(find("brand:voicemeeter").is_some());
        assert!(find("brand:chatgpt").is_some());
        assert!(
            all()
                .iter()
                .all(|e| e.id.starts_with("brand:") || e.glyph.chars().count() == 1)
        );
    }
}
