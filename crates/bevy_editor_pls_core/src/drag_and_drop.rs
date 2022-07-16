use bevy_inspector_egui::egui;

pub fn drag_source(
    ui: &mut egui::Ui,
    id: egui::Id,
    can_drag: bool,
    body: impl FnOnce(&mut egui::Ui),
) -> bool {
    if !can_drag {
        ui.scope(body);
        return false;
    }
    let is_being_dragged = ui.memory().is_being_dragged(id);

    if !is_being_dragged {
        let response = ui.scope(body).response;

        let response = ui.interact(response.rect, id, egui::Sense::click_and_drag());
        if response.dragged() && !response.clicked() {
            ui.output().cursor_icon = egui::CursorIcon::Grab;
        }
    } else {
        translate_ui_to_cursor(ui, id, body);
    }

    is_being_dragged
}

pub fn translate_ui_to_cursor(ui: &mut egui::Ui, id: egui::Id, body: impl FnOnce(&mut egui::Ui)) {
    let layer_id = egui::LayerId::new(egui::Order::Tooltip, id);
    let response = ui.with_layer_id(layer_id, body).response;

    if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
        let delta = pointer_pos - response.rect.center();
        ui.ctx().translate_layer(layer_id, delta);
    }
}

pub fn drop_target<R>(
    ui: &mut egui::Ui,
    can_accept_what_is_being_dragged: bool,
    body: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let is_being_dragged = ui.memory().is_anything_being_dragged();

    let margin = egui::Vec2::splat(4.0);

    let outer_rect_bounds = ui.available_rect_before_wrap();
    let inner_rect = outer_rect_bounds.shrink2(margin);
    let where_to_put_background = ui.painter().add(egui::Shape::Noop);

    let mut content_ui = ui.child_ui(inner_rect, *ui.layout());
    let ret = body(&mut content_ui);

    let outer_rect =
        egui::Rect::from_min_max(outer_rect_bounds.min, content_ui.min_rect().max + margin);
    let (rect, response) = ui.allocate_at_least(outer_rect.size(), egui::Sense::hover());

    if !is_being_dragged || !can_accept_what_is_being_dragged {
        return egui::InnerResponse::new(ret, response);
    }

    let style = if is_being_dragged && can_accept_what_is_being_dragged && response.hovered() {
        ui.visuals().widgets.active
    } else {
        ui.visuals().widgets.inactive
    };

    let mut fill = style.bg_fill;
    let mut stroke = style.bg_stroke;
    if is_being_dragged && !can_accept_what_is_being_dragged {
        // gray out:
        fill = egui::color::tint_color_towards(fill, ui.visuals().window_fill());
        stroke.color = egui::color::tint_color_towards(stroke.color, ui.visuals().window_fill());
    }

    ui.painter().set(
        where_to_put_background,
        egui::epaint::RectShape {
            rounding: style.rounding,
            fill,
            stroke,
            rect,
        },
    );

    egui::InnerResponse::new(ret, response)
}
