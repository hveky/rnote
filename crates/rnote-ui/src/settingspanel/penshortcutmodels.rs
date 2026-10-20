// Imports
use gettextrs::gettext;
use gtk4::{Align, StringObject};
use gtk4::{
    Image, Label, ListItem, Orientation, SignalListItemFactory, StringList, glib::prelude::*,
    prelude::*,
};
use rnote_compose::builders::ShapeBuilderType;
use rnote_engine::pens::PenStyle;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

/// The shape builder types selectable for a `SwitchPenStyle` shortcut on the
/// shaper, in dropdown order.
pub(crate) const SHORTCUT_SHAPE_BUILDERS: [ShapeBuilderType; 4] = [
    ShapeBuilderType::Line,
    ShapeBuilderType::Arrow,
    ShapeBuilderType::Rectangle,
    ShapeBuilderType::Ellipse,
];

#[derive(Debug, Clone)]
pub(crate) struct ShortcutShapeBuilderListModel(StringList);

impl Default for ShortcutShapeBuilderListModel {
    fn default() -> Self {
        let names: Vec<String> = SHORTCUT_SHAPE_BUILDERS
            .iter()
            .map(|builder| shape_builder_label(*builder))
            .collect();
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        Self(StringList::new(&refs))
    }
}

impl Deref for ShortcutShapeBuilderListModel {
    type Target = StringList;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ShortcutShapeBuilderListModel {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// The single source of translated shape builder display names, shared by
/// the shaper sidebar picker and the shortcut settings dropdowns.
pub(crate) fn shape_builder_display_name(builder: ShapeBuilderType) -> String {
    match builder {
        ShapeBuilderType::Line => gettext("Line"),
        ShapeBuilderType::Arrow => gettext("Arrow"),
        ShapeBuilderType::Rectangle => gettext("Rectangle"),
        ShapeBuilderType::Grid => gettext("Grid"),
        ShapeBuilderType::CoordSystem2D => gettext("2D coordinate system"),
        ShapeBuilderType::CoordSystem3D => gettext("3D coordinate system"),
        ShapeBuilderType::QuadrantCoordSystem2D => {
            gettext("2D single quadrant coordinate system")
        }
        ShapeBuilderType::Ellipse => gettext("Ellipse"),
        ShapeBuilderType::FociEllipse => gettext("Ellipse with foci"),
        ShapeBuilderType::QuadBez => gettext("Quadratic bezier curve"),
        ShapeBuilderType::CubBez => gettext("Cubic bezier curve"),
        ShapeBuilderType::Polyline => gettext("Polyline"),
        ShapeBuilderType::Polygon => gettext("Polygon"),
    }
}

pub(crate) fn shape_builder_label(builder: ShapeBuilderType) -> String {
    shape_builder_display_name(builder)
}

/// Returns the dropdown position for a builder type, if it is selectable.
pub(crate) fn shape_builder_position(builder: ShapeBuilderType) -> Option<u32> {
    SHORTCUT_SHAPE_BUILDERS
        .iter()
        .position(|b| *b == builder)
        .map(|p| p as u32)
}

#[derive(Debug, Clone)]
pub(crate) struct ChangePenStyleListModel(StringList);

impl Default for ChangePenStyleListModel {
    fn default() -> Self {
        Self(StringList::new(&[
            &PenStyle::Brush.to_string(),
            &PenStyle::Shaper.to_string(),
            &PenStyle::Typewriter.to_string(),
            &PenStyle::Eraser.to_string(),
            &PenStyle::Selector.to_string(),
            &PenStyle::Tools.to_string(),
        ]))
    }
}

impl Deref for ChangePenStyleListModel {
    type Target = StringList;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ChangePenStyleListModel {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ChangePenStyleListFactory(SignalListItemFactory);

impl Default for ChangePenStyleListFactory {
    fn default() -> Self {
        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_factory, list_item| {
            let list_item = list_item.downcast_ref::<ListItem>().unwrap();
            let item_box = gtk4::Box::builder()
                .orientation(Orientation::Horizontal)
                .build();
            let image = Image::builder().margin_end(12).halign(Align::Start).build();
            let label = Label::builder()
                .label("")
                .hexpand(true)
                .halign(Align::Start)
                .build();

            item_box.prepend(&image);
            item_box.append(&label);
            list_item.set_child(Some(&item_box));
        });
        factory.connect_bind(move |_factory, list_item| {
            let list_item = list_item.downcast_ref::<ListItem>().unwrap();
            let pen_style = PenStyle::from_str(
                &list_item
                    .item()
                    .unwrap()
                    .downcast::<StringObject>()
                    .unwrap()
                    .string(),
            )
            .unwrap();
            let item_box = list_item.child().unwrap().downcast::<gtk4::Box>().unwrap();

            let mut child = item_box.first_child();
            while let Some(ref next_child) = child {
                if next_child.type_() == Label::static_type() {
                    let label = match pen_style {
                        PenStyle::Brush => gettext("Brush"),
                        PenStyle::Shaper => gettext("Shaper"),
                        PenStyle::Typewriter => gettext("Typewriter"),
                        PenStyle::Eraser => gettext("Eraser"),
                        PenStyle::Selector => gettext("Selector"),
                        PenStyle::Tools => gettext("Tools"),
                    };
                    next_child
                        .downcast_ref::<Label>()
                        .unwrap()
                        .set_label(&label);
                } else if next_child.type_() == Image::static_type() {
                    next_child
                        .downcast_ref::<Image>()
                        .unwrap()
                        .set_icon_name(Some(pen_style.icon_name().as_str()));
                }

                child = next_child.next_sibling();
            }
        });
        Self(factory)
    }
}

impl Deref for ChangePenStyleListFactory {
    type Target = SignalListItemFactory;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ChangePenStyleListFactory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ChangePenStyleIconFactory(SignalListItemFactory);

impl Default for ChangePenStyleIconFactory {
    fn default() -> Self {
        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_factory, list_item| {
            let list_item = list_item.downcast_ref::<ListItem>().unwrap();

            let image = Image::builder().build();
            list_item.set_child(Some(&image));
        });
        factory.connect_bind(move |_factory, list_item| {
            let list_item = list_item.downcast_ref::<ListItem>().unwrap();
            let pen_style = PenStyle::from_str(
                &list_item
                    .item()
                    .unwrap()
                    .downcast::<StringObject>()
                    .unwrap()
                    .string(),
            )
            .unwrap();
            let image = list_item.child().unwrap().downcast::<Image>().unwrap();
            image
                .downcast_ref::<Image>()
                .unwrap()
                .set_icon_name(Some(pen_style.icon_name().as_str()));
        });
        Self(factory)
    }
}

impl Deref for ChangePenStyleIconFactory {
    type Target = SignalListItemFactory;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ChangePenStyleIconFactory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
