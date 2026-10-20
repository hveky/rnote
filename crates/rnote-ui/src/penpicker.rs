// Imports
use crate::RnAppWindow;
use gtk4::{
    Box, Button, CompositeTemplate, EventControllerLegacy, TemplateChild, ToggleButton, Widget,
    gio,
    glib::{self, clone},
    prelude::*,
    subclass::prelude::*,
};
use rnote_engine::pens::{PenMode, PenStyle};

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penpicker.ui")]
    pub(crate) struct RnPenPicker {
        #[template_child]
        pub(crate) toolbox: TemplateChild<Box>,
        #[template_child]
        pub(crate) brush_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) shaper_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) typewriter_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) eraser_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) selector_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) tools_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) undo_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) redo_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) separator: TemplateChild<gtk4::Separator>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnPenPicker {
        const NAME: &'static str = "RnPenPicker";
        type Type = super::RnPenPicker;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnPenPicker {
        fn dispose(&self) {
            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for RnPenPicker {}
}

glib::wrapper! {
    pub(crate) struct RnPenPicker(ObjectSubclass<imp::RnPenPicker>)
        @extends Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnPenPicker {
    fn default() -> Self {
        Self::new()
    }
}

impl RnPenPicker {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn brush_toggle(&self) -> ToggleButton {
        self.imp().brush_toggle.get()
    }

    pub(crate) fn shaper_toggle(&self) -> ToggleButton {
        self.imp().shaper_toggle.get()
    }

    pub(crate) fn typewriter_toggle(&self) -> ToggleButton {
        self.imp().typewriter_toggle.get()
    }

    pub(crate) fn eraser_toggle(&self) -> ToggleButton {
        self.imp().eraser_toggle.get()
    }

    pub(crate) fn selector_toggle(&self) -> ToggleButton {
        self.imp().selector_toggle.get()
    }

    pub(crate) fn tools_toggle(&self) -> ToggleButton {
        self.imp().tools_toggle.get()
    }

    pub(crate) fn undo_button(&self) -> Button {
        self.imp().undo_button.get()
    }

    pub(crate) fn redo_button(&self) -> Button {
        self.imp().redo_button.get()
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        // controller to switch pen mode when hovering with a pen/eraser over the toolbox toolbar
        let pointer_controller_toolbox = EventControllerLegacy::builder()
            .name("pointer_controller_toolbar")
            .propagation_phase(gtk4::PropagationPhase::Bubble)
            .build();

        pointer_controller_toolbox.connect_event(clone!(
            #[weak]
            appwindow,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |_, event| {
                if let Some(canvas) = appwindow.active_tab_canvas()
                    && let Some(device) = event.device_tool()
                {
                    match device.tool_type() {
                        gtk4::gdk::DeviceToolType::Pen => {
                            let widget_flags = canvas.engine_mut().change_pen_mode(PenMode::Pen);
                            canvas.emit_handle_widget_flags(widget_flags);
                        }
                        gtk4::gdk::DeviceToolType::Eraser => {
                            let widget_flags = canvas.engine_mut().change_pen_mode(PenMode::Eraser);
                            canvas.emit_handle_widget_flags(widget_flags);
                        }
                        _ => {}
                    }
                }

                glib::Propagation::Proceed
            }
        ));

        imp.toolbox.add_controller(pointer_controller_toolbox);

        imp.brush_toggle.connect_toggled(clone!(
            #[weak]
            appwindow,
            move |brush_toggle| {
                if brush_toggle.is_active() {
                    appwindow.set_pen_style_with_lock(PenStyle::Brush);
                }
            }
        ));

        imp.shaper_toggle.connect_toggled(clone!(
            #[weak]
            appwindow,
            move |shaper_toggle| {
                if shaper_toggle.is_active() {
                    appwindow.set_pen_style_with_lock(PenStyle::Shaper);
                }
            }
        ));

        imp.typewriter_toggle.connect_toggled(clone!(
            #[weak]
            appwindow,
            move |typewriter_toggle| {
                if typewriter_toggle.is_active() {
                    appwindow.set_pen_style_with_lock(PenStyle::Typewriter);
                }
            }
        ));

        imp.eraser_toggle.get().connect_toggled(clone!(
            #[weak]
            appwindow,
            move |eraser_toggle| {
                if eraser_toggle.is_active() {
                    appwindow.set_pen_style_with_lock(PenStyle::Eraser);
                }
            }
        ));

        imp.selector_toggle.get().connect_toggled(clone!(
            #[weak]
            appwindow,
            move |selector_toggle| {
                if selector_toggle.is_active() {
                    appwindow.set_pen_style_with_lock(PenStyle::Selector);
                }
            }
        ));

        imp.tools_toggle.get().connect_toggled(clone!(
            #[weak]
            appwindow,
            move |tools_toggle| {
                if tools_toggle.is_active() {
                    appwindow.set_pen_style_with_lock(PenStyle::Tools);
                }
            }
        ));
    }

    /// Maps a settings id to its widget. The six pen toggles and the
    /// undo/redo buttons come from the template children.
    pub(crate) fn widget_for_id(&self, id: &str) -> Option<Widget> {
        let imp = self.imp();
        Some(match id {
            "brush" => imp.brush_toggle.get().upcast(),
            "shaper" => imp.shaper_toggle.get().upcast(),
            "typewriter" => imp.typewriter_toggle.get().upcast(),
            "eraser" => imp.eraser_toggle.get().upcast(),
            "selector" => imp.selector_toggle.get().upcast(),
            "tools" => imp.tools_toggle.get().upcast(),
            "undo" => imp.undo_button.get().upcast(),
            "redo" => imp.redo_button.get().upcast(),
            _ => return None,
        })
    }

    /// Applies the button layout: reorders the listed ids into the toolbox
    /// in exactly the saved order (and the separator between the pen toggles
    /// and undo/redo), hides the buttons that are not listed.
    pub(crate) fn apply_layout(&self, ids: &[String]) {
        let imp = self.imp();
        let toolbox = imp.toolbox.get();

        // Hide everything first, then show and order the listed ones.
        for id in PENPICKER_BUTTON_IDS {
            if let Some(widget) = self.widget_for_id(id) {
                widget.set_visible(false);
            }
        }

        let move_into_toolbox = |widget: &Widget| {
            if widget.parent().is_some() {
                widget.unparent();
            }
            toolbox.append(widget);
        };

        // Append the listed buttons in exactly the saved order, inserting
        // the separator right before the first visible undo/redo button.
        let mut any_pen_toggle_visible = false;
        let mut separator_placed = false;
        let separator = imp.separator.get();

        for id in ids {
            if (id == "undo" || id == "redo")
                && !separator_placed
                && any_pen_toggle_visible
            {
                separator.set_visible(true);
                move_into_toolbox(separator.upcast_ref());
                separator_placed = true;
            }

            let Some(widget) = self.widget_for_id(id) else {
                continue;
            };
            widget.set_visible(true);
            move_into_toolbox(&widget);

            match id.as_str() {
                "undo" | "redo" => {}
                _ => any_pen_toggle_visible = true,
            }
        }

        // Hidden when either side is empty (or all visible buttons are on
        // one side of it).
        if !separator_placed {
            separator.set_visible(false);
            move_into_toolbox(separator.upcast_ref());
        }
    }

    /// Loads the pen picker button layout from the app settings.
    pub(crate) fn load_from_settings(&self, settings: &gio::Settings) {
        let ids: Vec<String> = settings
            .strv("penpicker-buttons")
            .into_iter()
            .map(Into::into)
            .collect();
        self.apply_layout(&sanitize_penpicker_ids(ids));
    }
}

/// The fixed id set of all pen picker buttons.
pub(crate) const PENPICKER_BUTTON_IDS: [&str; 8] = [
    "brush",
    "shaper",
    "typewriter",
    "eraser",
    "selector",
    "tools",
    "undo",
    "redo",
];

/// Drops unknown ids and duplicates, keeping the first occurrence's order.
pub(crate) fn sanitize_penpicker_ids(ids: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    ids.into_iter()
        .filter(|id| PENPICKER_BUTTON_IDS.contains(&id.as_str()) && seen.insert(id.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::sanitize_penpicker_ids;

    #[test]
    fn sanitize_drops_unknown_and_duplicates() {
        let ids = vec![
            "brush".to_string(),
            "nonexistent".to_string(),
            "brush".to_string(),
            "undo".to_string(),
        ];
        assert_eq!(sanitize_penpicker_ids(ids), vec!["brush", "undo"]);
    }

    #[test]
    fn sanitize_keeps_default_order() {
        let ids: Vec<String> = super::PENPICKER_BUTTON_IDS
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(sanitize_penpicker_ids(ids.clone()), ids);
    }

    #[test]
    fn sanitize_allows_empty() {
        assert!(sanitize_penpicker_ids(vec![]).is_empty());
    }
}
