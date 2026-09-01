use anyhow::Result;
use engine_core::{
    Resource,
    ecs::{
        query::{filter::With, query::Query},
        systems::param::{Res, ResMut},
    },
    egui::{
        self, Align2, Color32, CornerRadius, FontId, Frame, Rect, RichText, Sense, Stroke,
        StrokeKind, Vec2, Window, pos2,
    },
    input::InputManager,
    log_error,
    rendering::egui::context::EguiContext,
    update,
};
use game_data::components::item::ItemDef;

use crate::ui::containers::Container;
use crate::{GameState, components::Player};

const UI_TEXT_SIZE: f32 = 28.0;
const INVENTORY_WIDTH: f32 = 560.0;
const INVENTORY_HEIGHT: f32 = 480.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemTab {
    All,
    Weapon,
    Apparel,
    Magic,
    Misc,
}

impl ItemTab {
    pub const ALL: [ItemTab; 5] = [
        ItemTab::All,
        ItemTab::Weapon,
        ItemTab::Apparel,
        ItemTab::Magic,
        ItemTab::Misc,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ItemTab::All => "All",
            ItemTab::Weapon => "Weapon",
            ItemTab::Apparel => "Apparel",
            ItemTab::Magic => "Magic",
            ItemTab::Misc => "Misc",
        }
    }

    pub fn from_category(category: &str) -> ItemTab {
        match category.to_ascii_lowercase().as_str() {
            "weapon" | "weapons" | "sword" => ItemTab::Weapon,
            "apparel" | "armour" | "clothing" => ItemTab::Apparel,
            "magic" | "spell" | "scroll" => ItemTab::Magic,
            _ => ItemTab::Misc,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InventoryItem {
    pub def: ItemDef,
    pub count: u32,
    pub icon: Option<egui::TextureId>,
}

impl InventoryItem {
    pub fn from_def(def: ItemDef) -> Self {
        Self {
            def,
            count: 1,
            icon: None,
        }
    }

    pub fn display_name(&self) -> &str {
        &self.def.name
    }

    pub fn category(&self) -> ItemTab {
        self.def
            .categories
            .first()
            .map(|c| ItemTab::from_category(c))
            .unwrap_or(ItemTab::Misc)
    }
}

#[derive(Resource)]
pub struct InventoryUi {
    pub open: bool,
    pub selected_tab: ItemTab,
    pub weight: f32,
    pub weight_max: f32,
    pub armour: f32,
    icon_handles: Vec<egui::TextureHandle>,
}

impl std::fmt::Debug for InventoryUi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InventoryUi")
            .field("open", &self.open)
            .field("selected_tab", &self.selected_tab)
            .field("weight", &self.weight)
            .field("weight_max", &self.weight_max)
            .field("armour", &self.armour)
            .finish()
    }
}

impl Default for InventoryUi {
    fn default() -> Self {
        Self {
            open: true,
            selected_tab: ItemTab::All,
            weight: 28.0,
            weight_max: 50.0,
            armour: 12.0,
            icon_handles: Vec::new(),
        }
    }
}

fn accent() -> Color32 {
    Color32::from_rgb(120, 90, 40)
}

fn accent_text() -> Color32 {
    Color32::from_rgb(228, 186, 94)
}

fn item_color(category: ItemTab) -> Color32 {
    match category {
        ItemTab::Weapon => Color32::from_rgb(110, 120, 140),
        ItemTab::Apparel => Color32::from_rgb(120, 100, 90),
        ItemTab::Magic => Color32::from_rgb(100, 90, 140),
        ItemTab::Misc => Color32::from_rgb(110, 120, 90),
        ItemTab::All => Color32::from_rgb(120, 110, 90),
    }
}

/// Loads the item's icon as an egui colour image.
///
/// If the item def references a `sprite:<name>` icon it is always loaded from
/// `res/icons/<name>.png`. Otherwise a colored placeholder is used.
fn load_item_icon(item: &mut InventoryItem) -> egui::ColorImage {
    const ICON_SIZE: usize = 40;

    let Some(sprite) = item.def.sprite_icon() else {
        return egui::ColorImage::filled([ICON_SIZE, ICON_SIZE], item_color(item.category()));
    };

    let path = format!("{}/res/icons/{}.png", env!("CARGO_MANIFEST_DIR"), sprite);
    match image::open(&path) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw())
        }
        Err(err) => {
            log_error!(reason: "failed to load icon", "{}: {err}", path);
            egui::ColorImage::filled([ICON_SIZE, ICON_SIZE], item_color(item.category()))
        }
    }
}

fn item_slot(ui: &mut egui::Ui, item: &InventoryItem) {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(40.0), Sense::click());

    let painter = ui.painter();
    let bg = if response.hovered() {
        Color32::from_rgb(60, 90, 90)
    } else {
        item_color(item.category())
    };
    painter.rect_filled(rect, CornerRadius::same(2), bg);

    if let Some(icon) = item.icon {
        painter.image(
            icon,
            rect.shrink(3.0),
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    } else {
        let letter = item
            .display_name()
            .chars()
            .next()
            .unwrap_or('?')
            .to_string();
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            letter,
            FontId::proportional(UI_TEXT_SIZE),
            accent_text(),
        );
    }

    painter.rect_stroke(
        rect,
        CornerRadius::same(2),
        Stroke::new(1.0, accent()),
        StrokeKind::Inside,
    );

    if item.count > 1 {
        painter.text(
            rect.right_bottom(),
            Align2::RIGHT_BOTTOM,
            item.count.to_string(),
            FontId::proportional(UI_TEXT_SIZE),
            Color32::WHITE,
        );
    }

    response.on_hover_text(
        RichText::new(item.display_name())
            .size(UI_TEXT_SIZE)
            .color(accent_text()),
    );
}

fn weight_bar(ui: &mut egui::Ui, weight: f32, weight_max: f32) {
    let size = Vec2::new(140.0, 34.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());

    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(2), Color32::from_rgb(45, 20, 20));

    let ratio = (weight / weight_max).clamp(0.0, 1.0);
    let fill = Rect::from_min_size(rect.min, Vec2::new(rect.width() * ratio, rect.height()));
    painter.rect_filled(fill, CornerRadius::same(2), Color32::from_rgb(60, 60, 200));
    painter.rect_stroke(
        rect,
        CornerRadius::same(2),
        Stroke::new(1.0, accent()),
        StrokeKind::Inside,
    );

    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        format!("{weight:.0}/{weight_max:.0}"),
        FontId::proportional(UI_TEXT_SIZE),
        Color32::WHITE,
    );
}

fn paperdoll(ui: &mut egui::Ui, height: f32) {
    const BASE_W: f32 = 110.0;
    const BASE_H: f32 = 200.0;
    let width = height * (BASE_W / BASE_H);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let s = height / BASE_H;

    let painter = ui.painter();
    let bg = Color32::from_rgb(25, 20, 15);
    painter.rect_filled(rect, CornerRadius::same(3), bg);
    painter.rect_stroke(
        rect,
        CornerRadius::same(3),
        Stroke::new(1.0, accent()),
        StrokeKind::Inside,
    );

    let stroke = if response.hovered() {
        Stroke::new(2.0, Color32::from_rgb(60, 90, 90))
    } else {
        Stroke::new(1.0, accent())
    };
    painter.rect_stroke(rect, CornerRadius::same(3), stroke, StrokeKind::Inside);

    let body = Color32::from_rgb(70, 55, 40);
    let center_x = rect.center().x;
    let top = rect.top();

    painter.circle_filled(pos2(center_x, top + 34.0 * s), 20.0 * s, body);
    painter.rect_filled(
        Rect::from_center_size(
            pos2(center_x, top + 100.0 * s),
            Vec2::new(36.0 * s, 62.0 * s),
        ),
        CornerRadius::same((6.0 * s) as u8),
        body,
    );
    painter.rect_filled(
        Rect::from_center_size(
            pos2(center_x - 24.0 * s, top + 88.0 * s),
            Vec2::new(10.0 * s, 48.0 * s),
        ),
        CornerRadius::same((4.0 * s) as u8),
        body,
    );
    painter.rect_filled(
        Rect::from_center_size(
            pos2(center_x + 24.0 * s, top + 88.0 * s),
            Vec2::new(10.0 * s, 48.0 * s),
        ),
        CornerRadius::same((4.0 * s) as u8),
        body,
    );
    painter.rect_filled(
        Rect::from_center_size(
            pos2(center_x - 11.0 * s, top + 168.0 * s),
            Vec2::new(14.0 * s, 48.0 * s),
        ),
        CornerRadius::same((4.0 * s) as u8),
        body,
    );
    painter.rect_filled(
        Rect::from_center_size(
            pos2(center_x + 11.0 * s, top + 168.0 * s),
            Vec2::new(14.0 * s, 48.0 * s),
        ),
        CornerRadius::same((4.0 * s) as u8),
        body,
    );
}

#[update]
pub fn inventory_ui(
    context: ResMut<EguiContext>,
    game_state: Res<GameState>,
    input: Res<InputManager>,
    mut inventory: ResMut<InventoryUi>,
    players: Query<&mut Container, With<Player>>,
) -> Result<()> {
    if input.just_pressed("Inventory") {
        inventory.open = !inventory.open;
    }
    if !inventory.open || game_state.is_main_menu() {
        return Ok(());
    }

    let Some(container) = players.iter().next() else {
        return Ok(());
    };

    if container.items.iter().any(|i| i.icon.is_none()) {
        let handles: Vec<egui::TextureHandle> = container
            .items
            .iter_mut()
            .enumerate()
            .filter_map(|(i, item)| {
                if item.icon.is_some() {
                    return None;
                }
                let image = load_item_icon(item);
                let handle = context.0.load_texture(
                    format!("inventory_item_{i}"),
                    image,
                    egui::TextureOptions::NEAREST,
                );
                item.icon = Some(handle.id());
                Some(handle)
            })
            .collect();
        inventory.icon_handles.extend(handles);
    }

    let active = inventory.selected_tab;
    let mut pending_tab: Option<ItemTab> = None;

    Window::new("Inventory")
        .resizable(true)
        .movable(true)
        .collapsible(false)
        .default_size(Vec2::new(INVENTORY_WIDTH, INVENTORY_HEIGHT))
        .min_size(Vec2::new(360.0, INVENTORY_HEIGHT * 0.4))
        .frame(
            Frame::NONE
                .fill(Color32::from_rgb(30, 24, 18))
                .stroke(Stroke::new(2.0, accent()))
                .corner_radius(CornerRadius::same(4))
                .inner_margin(4.0),
        )
        .show(&context.0, |ui| {
            ui.style_mut().interaction.selectable_labels = false;
            ui.style_mut().interaction.multi_widget_text_select = false;
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);

            const SLOT: f32 = 40.0;
            const GAP: f32 = 8.0;

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Weight")
                        .size(UI_TEXT_SIZE)
                        .color(Color32::from_rgb(60, 60, 200)),
                );
                weight_bar(ui, inventory.weight, inventory.weight_max);
                ui.add_space(12.0);
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                for tab in ItemTab::ALL {
                    if ui
                        .selectable_label(
                            active == tab,
                            RichText::new(tab.label()).size(UI_TEXT_SIZE),
                        )
                        .clicked()
                    {
                        pending_tab = Some(tab);
                    }
                }
            });

            ui.separator();

            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    paperdoll(ui, 200.0);
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(format!("Armour {}", inventory.armour))
                            .size(UI_TEXT_SIZE)
                            .color(accent_text()),
                    );
                });

                ui.add_space(16.0);

                let filtered: Vec<&InventoryItem> = container
                    .items
                    .iter()
                    .filter(|i| active == ItemTab::All || i.category() == active)
                    .collect();

                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let grid_height = ui.available_height();

                        let max_rows = (((grid_height + GAP) / (SLOT + GAP)).floor() as usize)
                            .saturating_sub(1)
                            .max(1);
                        let columns = filtered.len().div_ceil(max_rows).max(1);

                        ui.horizontal_top(|ui| {
                            for col in 0..columns {
                                ui.vertical(|ui| {
                                    for (row, item) in filtered
                                        .iter()
                                        .skip(col * max_rows)
                                        .take(max_rows)
                                        .enumerate()
                                    {
                                        item_slot(ui, item);
                                        if row < max_rows - 1 {
                                            ui.add_space(GAP);
                                        }
                                    }
                                });
                                ui.add_space(GAP);
                            }
                        });
                    });
            });
        });

    if let Some(tab) = pending_tab {
        inventory.selected_tab = tab;
    }

    Ok(())
}
