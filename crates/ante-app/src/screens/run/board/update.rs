use super::*;

impl RunBoard {
    pub fn update(
        &mut self,
        commands: &mut Commands,
        view: &RunView,
        atlases: &Atlases,
        session: &Session,
        text: &crate::services::assets::ui_text::UiText,
    ) {
        let trace = session.playback.as_ref();
        let preview = session.deck_hovered
            && trace.is_none()
            && session.navigation.screen == Page::Run
            && view.phase == Phase::Playing;
        let frame = BoardFrame {
            revision: view.revision,
            trace: trace.is_some(),
            speed: session.speed,
            preview,
        };
        commands
            .entity(self.error)
            .insert(Text::new(session.error.as_deref().unwrap_or_default()));
        if self.frame == Some(frame) {
            return;
        }
        self.frame = Some(frame);
        let title = match view.phase {
            Phase::BeforeBlind => "Choose your\nnext Blind",
            Phase::Shop if self.shop_sign != Entity::PLACEHOLDER => "",
            Phase::Shop => "Shop",
            _ => &view.blind,
        };
        let in_blind = view.phase == Phase::Playing;
        let title = if in_blind { "" } else { title };
        let rule = String::new();
        for (entity, text) in self.hud.into_iter().zip([
            title.to_owned(),
            rule,
            score_text(view.score),
            view.hands.to_string(),
            view.discards.to_string(),
            if view.in_debt {
                format!("-${}", view.money)
            } else {
                format!("${}", view.money)
            },
            view.ante.to_string(),
            view.round.to_string(),
        ]) {
            commands.entity(entity).insert(Text::new(text));
        }
        commands
            .entity(self.ante_end)
            .insert(Text::new(if view.endless_mode {
                String::new()
            } else {
                format!(" / {}", view.ante_end)
            }));
        if self.shop_sign != Entity::PLACEHOLDER {
            commands
                .entity(self.shop_sign)
                .insert(shop_sign_node(if view.phase == Phase::Shop {
                    Display::Flex
                } else {
                    Display::None
                }));
        }
        commands
            .entity(self.blind_box)
            .insert(blind_box_node(if in_blind {
                Display::Flex
            } else {
                Display::None
            }));
        if in_blind {
            if let Some(chip) = view.blind_chip.and_then(|id| atlases.blind_chip(id)) {
                commands.entity(self.blind_chip).insert(chip);
            }
            let [name, rule, target, reward] = self.blind_texts;
            commands.entity(name).insert(Text::new(view.blind.clone()));
            commands
                .entity(rule)
                .insert(Text::new(view.blind_rule.clone()));
            // `blind_chip_UI_scale`; only the size changes, the game font stays.
            commands.entity(target).insert((
                Text::new(score_text(view.target)),
                crate::app::presentation::TextSize(text_px(scale_number(
                    view.target as f64,
                    0.7,
                    100_000.0,
                ))),
            ));
            commands
                .entity(reward)
                .insert(Text::new("$".repeat(view.reward)));
        }
        commands.entity(self.hand_title).insert(Text::new(
            view.selected_hand.map_or_else(String::new, |hand| {
                format!("{}  lvl.{}", hand.name(), hand.level.level)
            }),
        ));
        let (chips, mult) = trace
            .and_then(|(_, playback)| playback.score_values())
            .unwrap_or_else(|| {
                view.selected_hand
                    .map_or((0, 0.0), |hand| (hand.level.chips, hand.level.mult as f64))
            });
        commands
            .entity(self.score_tiles[0])
            .insert(Text::new(score_text(chips)));
        commands
            .entity(self.score_tiles[1])
            .insert(Text::new(mult_text(mult)));
        if self.deck_shown != Some(view.deck_cards)
            && let Some(back) = self.deck_back.clone()
        {
            self.deck_shown = Some(view.deck_cards);
            let card = crate::components::tiles::card_px();
            let n = view.deck_cards;
            let top = deck_offset(n, 1) * TILE;
            commands
                .entity(self.deck_label)
                .insert(UiTransform::from_xy(px(top.x), px(top.y)));
            commands
                .entity(self.deck_pile)
                .despawn_children()
                .with_children(|pile| {
                    for k in drawn_backs(n) {
                        let offset = deck_offset(n, k) * TILE;
                        // Cards under the top one read greyer, like their stacked shadows.
                        let shade = if k == 1 { 1.0 } else { DECK_UNDER_SHADE };
                        pile.spawn((
                            ImageNode {
                                color: Color::srgb(shade, shade, shade),
                                ..back.clone()
                            },
                            Node {
                                position_type: PositionType::Absolute,
                                left: px((DECK_AREA.x - card.x) / 2.0 + offset.x),
                                top: px((DECK_AREA.y - card.y) / 2.0 + offset.y),
                                width: px(card.x),
                                height: px(card.y),
                                ..default()
                            },
                            Pickable::IGNORE,
                        ));
                    }
                });
        }
        commands.entity(self.deck_count).insert(Text::new(format!(
            "{}/{}",
            view.deck_cards, view.deck_total
        )));
        for (i, text) in [
            format!("{}/{}", view.held_jokers.len(), view.joker_slots),
            format!("{}/{}", view.consumables.len(), view.consumable_slots),
        ]
        .into_iter()
        .enumerate()
        {
            commands.entity(self.counts[i]).insert(Text::new(text));
            commands
                .entity(self.inventory[i])
                .despawn_children()
                .with_children(|zone| {
                    inventory_cards(zone, view, atlases, trace.is_some(), i == 0);
                });
        }
        let keys: Vec<_> = view.tags.iter().map(|tag| tag.content.id.0).collect();
        if self.tag_keys != keys {
            self.tag_keys = keys;
            commands
                .entity(self.tags)
                .despawn_children()
                .with_children(|stack| {
                    for tag in &view.tags {
                        tag_icon(stack, atlases, tag);
                    }
                });
        }
        // A steady hand keeps its action row: only the hand and the button states change.
        let steady = view.phase == Phase::Playing && trace.is_none() && !preview;
        if !steady {
            self.playing = None;
        }
        if let Some(slots) = self.playing.as_mut() {
            slots.refresh(commands, view, atlases);
            return;
        }
        let mut playing = None;
        commands
            .entity(self.center)
            .insert(center_layout(view.phase))
            .despawn_children()
            .with_children(|center| match view.phase {
                Phase::BeforeBlind => {
                    for (index, blind) in view.blinds.iter().enumerate() {
                        blind_panel(
                            center,
                            blind,
                            index,
                            atlases,
                            BlindPanel::Choice {
                                revision: view.revision,
                            },
                        );
                    }
                    if view.can_reroll_boss
                        || view
                            .vouchers
                            .iter()
                            .any(|v| matches!(v.id.0, "v_directors_cut" | "v_retcon"))
                    {
                        tile_optional(
                            center,
                            "Reroll Boss $10",
                            ORANGE,
                            (2.65, 0.65),
                            0.43,
                            view.can_reroll_boss
                                .then_some(Click::Play(Command::RerollBoss)),
                            false,
                        );
                    }
                }
                Phase::Playing => {
                    playing = playing_board(
                        center,
                        view,
                        atlases,
                        trace.is_some(),
                        session.speed,
                        preview,
                    );
                }
                Phase::Shop => {
                    shop_board(center, view, atlases, text);
                }
                Phase::Reward => {
                    round_eval_panel(center, view, atlases, session.stake, text);
                }
                Phase::Pack => pack_board(center, view, atlases),
                Phase::Won | Phase::Lost => {}
            });
        self.playing = playing;
    }
}
