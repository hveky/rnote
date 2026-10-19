// Imports
use super::keybindingbutton::RnKeybindingButton;
use super::penshortcutmodels::{
    SHORTCUT_SHAPE_BUILDERS, ChangePenStyleIconFactory, ChangePenStyleListFactory,
    ChangePenStyleListModel, ShortcutShapeBuilderListModel, shape_builder_position,
};
use adw::{prelude::*, subclass::prelude::*};
use gtk4::{
    Button, CompositeTemplate, DropDown, ListBoxRow, Widget, glib, glib::clone, glib::subclass::*,
};
use num_traits::ToPrimitive;
use once_cell::sync::Lazy;
use rnote_compose::builders::ShapeBuilderType;
use rnote_compose::penevent::KeyBinding;
use rnote_engine::pens::PenStyle;
use rnote_engine::pens::shortcuts::{ShortcutAction, ShortcutMode};
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penshortcutkeyboardrow.ui")]
    pub(crate) struct RnKeyboardShortcutRow {
        pub(crate) binding: RefCell<Option<KeyBinding>>,
        pub(crate) action: RefCell<ShortcutAction>,
        pub(crate) changepenstyle_model: ChangePenStyleListModel,
        pub(crate) shapebuilder_model: ShortcutShapeBuilderListModel,

        #[template_child(id = "keybinding_button")]
        pub(crate) keybinding_button: TemplateChild<RnKeybindingButton>,
        #[template_child]
        pub(crate) penstyle_dropdown: TemplateChild<DropDown>,
        #[template_child]
        pub(crate) mode_dropdown: TemplateChild<DropDown>,
        #[template_child]
        pub(crate) shape_builder_dropdown: TemplateChild<DropDown>,
        #[template_child]
        pub(crate) delete_button: TemplateChild<Button>,
    }

    impl Default for RnKeyboardShortcutRow {
        fn default() -> Self {
            Self {
                binding: RefCell::new(None),
                action: RefCell::new(ShortcutAction::ChangePenStyle {
                    style: PenStyle::Brush,
                    mode: ShortcutMode::Permanent,
                }),
                changepenstyle_model: ChangePenStyleListModel::default(),
                shapebuilder_model: ShortcutShapeBuilderListModel::default(),

                keybinding_button: TemplateChild::default(),
                penstyle_dropdown: TemplateChild::default(),
                mode_dropdown: TemplateChild::default(),
                shape_builder_dropdown: TemplateChild::default(),
                delete_button: TemplateChild::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnKeyboardShortcutRow {
        const NAME: &'static str = "RnKeyboardShortcutRow";
        type Type = super::RnKeyboardShortcutRow;
        type ParentType = adw::ActionRow;
        type Interfaces = ();

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnKeyboardShortcutRow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            let list_factory = ChangePenStyleListFactory::default();
            let icon_factory = ChangePenStyleIconFactory::default();

            obj.imp()
                .penstyle_dropdown
                .get()
                .set_model(Some(&*obj.imp().changepenstyle_model));
            obj.imp()
                .penstyle_dropdown
                .get()
                .set_list_factory(Some(&*list_factory));
            obj.imp()
                .penstyle_dropdown
                .get()
                .set_factory(Some(&*icon_factory));
            obj.imp()
                .shape_builder_dropdown
                .get()
                .set_model(Some(&*obj.imp().shapebuilder_model));

            obj.imp()
                .keybinding_button
                .get()
                .connect_binding_changed(clone!(
                    #[weak(rename_to = row)]
                    obj,
                    #[upgrade_or]
                    None,
                    move |_| {
                        // Emit row-changed BEFORE overwriting the saved binding,
                        // so consumers read the previous binding as old_binding
                        // and can remove it from the config.
                        row.emit_by_name::<()>("row-changed", &[]);
                        *row.imp().binding.borrow_mut() =
                            row.imp().keybinding_button.get().binding();
                        None
                    }
                ));

            obj.imp()
                .penstyle_dropdown
                .get()
                .connect_selected_notify(clone!(
                    #[weak(rename_to = row)]
                    obj,
                    move |_| {
                        let new_style = row.pen_style();
                        match &mut *row.imp().action.borrow_mut() {
                            ShortcutAction::ChangePenStyle { style, .. }
                            | ShortcutAction::SwitchPenStyle { style, .. } => {
                                *style = new_style;
                            }
                        }
                        row.update_shape_builder_dropdown();
                        row.emit_by_name::<()>("row-changed", &[]);
                    }
                ));

            obj.imp()
                .mode_dropdown
                .get()
                .connect_selected_notify(clone!(
                    #[weak(rename_to = row)]
                    obj,
                    move |_| {
                        match &mut *row.imp().action.borrow_mut() {
                            ShortcutAction::ChangePenStyle { mode, .. }
                            | ShortcutAction::SwitchPenStyle { mode, .. } => {
                                *mode = row.shortcut_mode();
                            }
                        }
                        row.emit_by_name::<()>("row-changed", &[]);
                    }
                ));

            obj.imp()
                .shape_builder_dropdown
                .get()
                .connect_selected_notify(clone!(
                    #[weak(rename_to = row)]
                    obj,
                    move |_| {
                        // Read the selected builder without holding the
                        // refcell borrow: set_selected() in
                        // update_shape_builder_dropdown() can re-enter this
                        // handler while a borrow is active.
                        let position = row.imp().shape_builder_dropdown.get().selected();
                        let new_builder =
                            SHORTCUT_SHAPE_BUILDERS.get(position as usize).copied();

                        let action_is_switch = {
                            let action = row.imp().action.borrow();
                            matches!(&*action, ShortcutAction::SwitchPenStyle { .. })
                        };
                        if action_is_switch {
                            if let ShortcutAction::SwitchPenStyle { shape_builder, .. } =
                                &mut *row.imp().action.borrow_mut()
                            {
                                *shape_builder = new_builder;
                            }
                        }
                        row.emit_by_name::<()>("row-changed", &[]);
                    }
                ));
        }

        fn dispose(&self) {
            // `dispose_template` unparents the template children; do not
            // unparent them again manually here, that hits freed pointers.
            self.dispose_template();
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("row-changed").build()]);
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for RnKeyboardShortcutRow {}
    impl ListBoxRowImpl for RnKeyboardShortcutRow {}
    impl PreferencesRowImpl for RnKeyboardShortcutRow {}
    impl ActionRowImpl for RnKeyboardShortcutRow {}
}

glib::wrapper! {
    pub(crate) struct RnKeyboardShortcutRow(ObjectSubclass<imp::RnKeyboardShortcutRow>)
        @extends adw::ActionRow, adw::PreferencesRow, ListBoxRow, Widget,
        @implements gtk4::Accessible, gtk4::Actionable, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnKeyboardShortcutRow {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl RnKeyboardShortcutRow {
    #[allow(clippy::new_without_default)]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Connects a callback that is invoked whenever the binding, the pen style
    /// or the mode dropdown changed (i.e. the row was edited).
    pub(crate) fn connect_row_changed<F>(&self, f: F)
    where
        F: Fn(&[glib::Value]) -> Option<glib::Value> + 'static,
    {
        self.connect_local("row-changed", false, f);
    }

    pub(crate) fn keybinding_button(&self) -> RnKeybindingButton {
        self.imp().keybinding_button.get()
    }

    pub(crate) fn delete_button(&self) -> Button {
        self.imp().delete_button.get()
    }

    pub(crate) fn binding(&self) -> Option<KeyBinding> {
        *self.imp().binding.borrow()
    }

    pub(crate) fn set_binding(&self, binding: Option<KeyBinding>) {
        *self.imp().binding.borrow_mut() = binding;
        self.imp().keybinding_button.get().set_binding(binding);
    }

    pub(crate) fn action(&self) -> ShortcutAction {
        *self.imp().action.borrow()
    }

    pub(crate) fn set_action(&self, action: ShortcutAction) {
        *self.imp().action.borrow_mut() = action;
        match action {
            ShortcutAction::ChangePenStyle { style, mode } => {
                self.set_pen_style(style);
                self.set_shortcut_mode(mode);
            }
            ShortcutAction::SwitchPenStyle {
                style,
                shape_builder,
                mode,
            } => {
                self.set_pen_style(style);
                self.set_shortcut_mode(mode);
                if let Some(builder) = shape_builder {
                    self.set_shape_builder(builder);
                }
            }
        }
        self.update_shape_builder_dropdown();
    }

    /// Shows the shape-builder dropdown only for `SwitchPenStyle` actions
    /// targeting the shaper, and syncs its selection. Reads the values before
    /// calling `set_selected` — that call fires the notify handler, which
    /// borrows the action itself.
    fn update_shape_builder_dropdown(&self) {
        let (visible, position) = {
            let action = self.imp().action.borrow();
            match &*action {
                ShortcutAction::SwitchPenStyle {
                    style: PenStyle::Shaper,
                    shape_builder,
                    ..
                } => (true, shape_builder.and_then(|b| shape_builder_position(b))),
                _ => (false, None),
            }
        };

        self.imp().shape_builder_dropdown.get().set_visible(visible);
        if let Some(position) = position {
            self.imp().shape_builder_dropdown.get().set_selected(position);
        }
    }

    pub(crate) fn set_shape_builder(&self, builder: ShapeBuilderType) {
        if let Some(position) = shape_builder_position(builder) {
            self.imp().shape_builder_dropdown.get().set_selected(position);
        }
    }

    pub(crate) fn pen_style(&self) -> PenStyle {
        PenStyle::try_from(self.imp().penstyle_dropdown.get().selected()).unwrap()
    }

    pub(crate) fn set_pen_style(&self, style: PenStyle) {
        self.imp()
            .penstyle_dropdown
            .get()
            .set_selected(style.to_u32().unwrap());
    }

    pub(crate) fn shortcut_mode(&self) -> ShortcutMode {
        ShortcutMode::try_from(self.imp().mode_dropdown.get().selected()).unwrap()
    }

    pub(crate) fn set_shortcut_mode(&self, mode: ShortcutMode) {
        self.imp()
            .mode_dropdown
            .get()
            .set_selected(mode.to_u32().unwrap());
    }
}
