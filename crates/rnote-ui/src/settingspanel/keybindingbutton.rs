// Imports
use adw::{prelude::*, subclass::prelude::*};
use gtk4::{
    EventControllerKey, PropagationPhase, Widget, gdk, glib, glib::clone, glib::subclass::*,
};
use once_cell::sync::Lazy;
use rnote_compose::penevent::KeyBinding;
use std::cell::{Cell, RefCell};

mod imp {
    use super::*;
    use glib::translate::*;

    #[derive(Debug, Default)]
    pub(crate) struct RnKeybindingButton {
        pub(crate) capturing: Cell<bool>,
        pub(crate) binding: RefCell<Option<KeyBinding>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnKeybindingButton {
        const NAME: &'static str = "RnKeybindingButton";
        type Type = super::RnKeybindingButton;
        type ParentType = gtk4::Button;
        type Interfaces = ();

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("keybindingbutton");
        }
    }

    impl ObjectImpl for RnKeybindingButton {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            obj.set_tooltip_text(Some(
                "Click, then press a key combination. Esc cancels, Backspace clears.",
            ));

            let key_controller = EventControllerKey::new();
            key_controller.set_propagation_phase(PropagationPhase::Capture);
            key_controller.connect_key_pressed(clone!(
                #[weak(rename_to = widget)]
                obj,
                #[upgrade_or]
                glib::Propagation::Proceed,
                move |_controller, key, _keycode, state| {
                    let keyval: u32 = key.into_glib();
                    widget.handle_capture_key(keyval, state)
                }
            ));
            obj.add_controller(key_controller);

            obj.connect_clicked(clone!(
                #[weak(rename_to = widget)]
                obj,
                move |_| {
                    widget.start_capture();
                }
            ));
        }

        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("binding-changed").build()]);
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for RnKeybindingButton {}
    impl ButtonImpl for RnKeybindingButton {}
}

glib::wrapper! {
    pub(crate) struct RnKeybindingButton(ObjectSubclass<imp::RnKeybindingButton>)
        @extends gtk4::Button, Widget,
        @implements gtk4::Accessible, gtk4::Actionable, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnKeybindingButton {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl RnKeybindingButton {
    pub(crate) fn binding(&self) -> Option<KeyBinding> {
        *self.imp().binding.borrow()
    }

    /// Connects a callback that is invoked whenever the binding changed
    /// (including being cleared with backspace).
    pub(crate) fn connect_binding_changed<F>(&self, f: F)
    where
        F: Fn(&[glib::Value]) -> Option<glib::Value> + 'static,
    {
        self.connect_local("binding-changed", false, f);
    }

    /// Sets the binding and updates the label without emitting `binding-changed`.
    pub(crate) fn set_binding(&self, binding: Option<KeyBinding>) {
        *self.imp().binding.borrow_mut() = binding;
        self.update_label();
    }

    fn start_capture(&self) {
        self.imp().capturing.set(true);
        self.add_css_class("error");
        self.set_label("Press keys… (Esc to cancel, Backspace to clear)");
        self.grab_focus();
    }

    fn stop_capture(&self) {
        self.imp().capturing.set(false);
        self.remove_css_class("error");
        self.update_label();
    }

    fn update_label(&self) {
        match self.imp().binding.borrow().as_ref() {
            Some(binding) => {
                // SAFETY: both values are plain u32 glib conversions.
                let key: gdk::Key = unsafe { glib::translate::from_glib(binding.keyval) };
                let modifiers: gdk::ModifierType =
                    unsafe { glib::translate::from_glib(binding.modifiers) };
                let label = gtk4::accelerator_get_label(key, modifiers);
                if label.is_empty() {
                    self.set_label("Invalid key");
                } else {
                    self.set_label(&label);
                }
            }
            None => self.set_label("Set shortcut…"),
        }
    }

    /// Called from the capture-phase key controller while (and only while)
    /// capturing. Returns Stop for everything to shield the event from canvas
    /// shortcuts and app accelerators.
    fn handle_capture_key(&self, keyval: u32, state: gdk::ModifierType) -> glib::Propagation {
        if !self.imp().capturing.get() {
            return glib::Propagation::Proceed;
        }

        // SAFETY: plain u32 glib conversion.
        let key: gdk::Key = unsafe { glib::translate::from_glib(keyval) };
        let no_modifiers = !state.intersects(
            gdk::ModifierType::CONTROL_MASK
                | gdk::ModifierType::ALT_MASK
                | gdk::ModifierType::SHIFT_MASK,
        );

        // Escape without any modifiers cancels the capture.
        if no_modifiers && key == gdk::Key::Escape {
            self.stop_capture();
            return glib::Propagation::Stop;
        }

        // Backspace without any modifiers clears the binding.
        if no_modifiers && key == gdk::Key::BackSpace {
            *self.imp().binding.borrow_mut() = None;
            self.emit_by_name::<()>("binding-changed", &[]);
            self.stop_capture();
            return glib::Propagation::Stop;
        }

        // Modifier-only presses do not complete the binding.
        if matches!(
            key,
            gdk::Key::Control_L
                | gdk::Key::Control_R
                | gdk::Key::Shift_L
                | gdk::Key::Shift_R
                | gdk::Key::Alt_L
                | gdk::Key::Alt_R
                | gdk::Key::Meta_L
                | gdk::Key::Meta_R
        ) {
            return glib::Propagation::Stop;
        }

        let binding = KeyBinding {
            keyval,
            modifiers: state.bits(),
        }
        .normalized();
        *self.imp().binding.borrow_mut() = Some(binding);
        self.emit_by_name::<()>("binding-changed", &[]);
        self.stop_capture();

        glib::Propagation::Stop
    }
}
