use super::layout::page_layout;
use super::*;
use crate::components::{
    card_area::CardArea,
    card_row::card_row,
    tiles::{CARD_HEIGHT, CARD_WIDTH, option_cycle},
};

/// A card page: the page's card areas in one black box (padding 0.1, emboss 0.05), then
/// the page cycle, as in `create_UIBox_your_collection_jokers` and its siblings.
pub(super) fn card_page(
    ui: &mut ChildSpawnerCommands,
    page: usize,
    profile: &Profile,
    atlases: &Atlases,
    group: ContentGroup,
    catalog: &[Content],
) {
    let Some(layout) = page_layout(group) else {
        return;
    };
    let pages = layout.pages(catalog.len());
    let page = page.min(pages - 1);
    let card = Vec2::new(CARD_WIDTH, CARD_HEIGHT) * layout.card_scale;
    generic_options(ui, Click::Back, |content| {
        unsaved_note(content, profile);
        content
            .spawn((
                Node {
                    min_width: px(2.5 * TILE),
                    padding: UiRect::all(px(0.1 * TILE)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                PixelBox::new(BLACK).emboss(0.05),
            ))
            .with_children(|rows| {
                let mut start = page * layout.per_page();
                for spec in &layout.rows {
                    let end = (start + spec.count).min(catalog.len());
                    let items = &catalog[start.min(end)..end];
                    start += spec.count;
                    let area = CardArea::new(spec.kind, spec.width * CARD_W, CARD_W, spec.limit);
                    rows.spawn(Node {
                        padding: UiRect::all(px(layout.row_padding * TILE)),
                        ..default()
                    })
                    .with_children(|row| {
                        card_row(
                            row,
                            area,
                            spec.height * CARD_H,
                            card,
                            items.len(),
                            |entity, i| {
                                let item = &items[i];
                                let (discovered, locked) = shown(profile, group, item);
                                tooltip::attach(
                                    entity,
                                    collection_tip(item, group, discovered, locked),
                                );
                                entity.with_children(|art| {
                                    collection_card_art(
                                        art,
                                        atlases,
                                        group,
                                        item.id.0,
                                        (discovered, locked),
                                        card,
                                    );
                                });
                            },
                        );
                    });
                }
            });
        if layout.cycle {
            let previous = Click::Collection((page + pages - 1) % pages);
            let next = Click::Collection((page + 1) % pages);
            let turns = pages > 1;
            option_cycle(
                content,
                &format!("Page {}/{}", page + 1, pages),
                4.5,
                turns.then_some(previous),
                turns.then_some(next),
            );
        } else if group == ContentGroup::Planets {
            // An empty row with 0.7 padding keeps the page as tall as the others.
            content.spawn(Node {
                height: px(1.4 * TILE),
                ..default()
            });
        }
        if matches!(
            group,
            ContentGroup::Enhancements | ContentGroup::Seals | ContentGroup::Editions
        ) {
            label(
                content,
                "A playing card holds at most one enhancement, edition and seal",
                text_px(0.32),
                MUTED,
            );
        }
    });
}
